// Copyright 2015-2017 Parity Technologies (UK) Ltd.
// This file is part of Parity.

// Parity is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity.  If not, see <http://www.gnu.org/licenses/>.

use std::fmt::Debug;
use std::ops::Deref;
use rlp;
use util::{Address, H520, H256, U256, Uint, Bytes};
use util::bytes::ToPretty;
use util::sha3::Hashable;

use ethkey::Signature;
use ethcore::miner::MinerService;
use ethcore::client::MiningBlockChainClient;
use ethcore::transaction::{Action, SignedTransaction, PendingTransaction, Transaction};
use ethcore::account_provider::AccountProvider;

use jsonrpc_core::Error;
use v1::helpers::{errors, TransactionRequest, FilledTransactionRequest, ConfirmationPayload};
use v1::types::{
	H256 as RpcH256, H520 as RpcH520, Bytes as RpcBytes,
	RichRawTransaction as RpcRichRawTransaction,
	ConfirmationPayload as RpcConfirmationPayload,
	ConfirmationResponse,
	SignRequest as RpcSignRequest,
	DecryptRequest as RpcDecryptRequest,
};

pub const DEFAULT_MAC: [u8; 2] = [0, 0];

type AccountToken = String;

#[derive(Debug, Clone, PartialEq)]
pub enum SignWith {
	Nothing,
	Password(String),
	Token(AccountToken),
}

#[derive(Debug)]
pub enum WithToken<T: Debug> {
	No(T),
	Yes(T, AccountToken),
}

impl<T: Debug> Deref for WithToken<T> {
	type Target = T;

	fn deref(&self) -> &Self::Target {
		match *self {
			WithToken::No(ref v) => v,
			WithToken::Yes(ref v, _) => v,
		}
	}
}

impl<T: Debug> WithToken<T> {
	pub fn map<S, F>(self, f: F) -> WithToken<S> where
		S: Debug,
		F: FnOnce(T) -> S,
	{
		match self {
			WithToken::No(v) => WithToken::No(f(v)),
			WithToken::Yes(v, token) => WithToken::Yes(f(v), token),
		}
	}

	pub fn into_value(self) -> T {
		match self {
			WithToken::No(v) => v,
			WithToken::Yes(v, _) => v,
		}
	}
}

impl<T: Debug> From<(T, AccountToken)> for WithToken<T> {
	fn from(tuple: (T, AccountToken)) -> Self {
		WithToken::Yes(tuple.0, tuple.1)
	}
}

pub fn execute<C, M>(client: &C, miner: &M, accounts: &AccountProvider, payload: ConfirmationPayload, pass: SignWith) -> Result<WithToken<ConfirmationResponse>, Error>
	where C: MiningBlockChainClient, M: MinerService
{
	match payload {
		ConfirmationPayload::SendTransaction(request) => {
			sign_and_dispatch(client, miner, accounts, request, pass)
				.map(|result| result
					.map(RpcH256::from)
					.map(ConfirmationResponse::SendTransaction)
				)
		},
		ConfirmationPayload::SignTransaction(request) => {
			sign_no_dispatch(client, miner, accounts, request, pass)
				.map(|result| result
					.map(RpcRichRawTransaction::from)
					.map(ConfirmationResponse::SignTransaction)
				)
		},
		ConfirmationPayload::Signature(address, data) => {
			signature(accounts, address, data.sha3(), pass)
				.map(|result| result
					.map(|rsv| {
						let mut vrs = [0u8; 65];
						let rsv = rsv.as_ref();
						vrs[0] = rsv[64] + 27;
						vrs[1..33].copy_from_slice(&rsv[0..32]);
						vrs[33..65].copy_from_slice(&rsv[32..64]);
						H520(vrs)
					})
					.map(RpcH520::from)
					.map(ConfirmationResponse::Signature)
				)
		},
		ConfirmationPayload::Decrypt(address, data) => {
			decrypt(accounts, address, data, pass)
				.map(|result| result
					.map(RpcBytes)
					.map(ConfirmationResponse::Decrypt)
				)
		},
	}
}

fn signature(accounts: &AccountProvider, address: Address, hash: H256, password: SignWith) -> Result<WithToken<Signature>, Error> {
	match password.clone() {
		SignWith::Nothing => accounts.sign(address, None, hash).map(WithToken::No),
		SignWith::Password(pass) => accounts.sign(address, Some(pass), hash).map(WithToken::No),
		SignWith::Token(token) => accounts.sign_with_token(address, token, hash).map(Into::into),
	}.map_err(|e| match password {
		SignWith::Nothing => errors::from_signing_error(e),
		_ => errors::from_password_error(e),
	})
}

fn decrypt(accounts: &AccountProvider, address: Address, msg: Bytes, password: SignWith) -> Result<WithToken<Bytes>, Error> {
	match password.clone() {
		SignWith::Nothing => accounts.decrypt(address, None, &DEFAULT_MAC, &msg).map(WithToken::No),
		SignWith::Password(pass) => accounts.decrypt(address, Some(pass), &DEFAULT_MAC, &msg).map(WithToken::No),
		SignWith::Token(token) => accounts.decrypt_with_token(address, token, &DEFAULT_MAC, &msg).map(Into::into),
	}.map_err(|e| match password {
		SignWith::Nothing => errors::from_signing_error(e),
		_ => errors::from_password_error(e),
	})
}

pub fn dispatch_transaction<C, M>(client: &C, miner: &M, signed_transaction: PendingTransaction) -> Result<H256, Error>
	where C: MiningBlockChainClient, M: MinerService {
	let hash = signed_transaction.transaction.hash();

	miner.import_own_transaction(client, signed_transaction)
		.map_err(errors::from_transaction_error)
		.map(|_| hash)
}

pub fn sign_no_dispatch<C, M>(client: &C, miner: &M, accounts: &AccountProvider, filled: FilledTransactionRequest, password: SignWith) -> Result<WithToken<SignedTransaction>, Error>
	where C: MiningBlockChainClient, M: MinerService {

	let network_id = client.signing_network_id();
	let address = filled.from;
	let signed_transaction = {
		let t = Transaction {
			nonce: filled.nonce
				.or_else(|| miner
					.last_nonce(&filled.from)
					.map(|nonce| nonce + U256::one()))
				.unwrap_or_else(|| client.latest_nonce(&filled.from)),

			action: filled.to.map_or(Action::Create, Action::Call),
			gas: filled.gas,
			gas_price: filled.gas_price,
			value: filled.value,
			data: filled.data,
		};

		let hash = t.hash(network_id);
		let signature = signature(accounts, address, hash, password)?;
		signature.map(|sig| {
			SignedTransaction::new(t.with_signature(sig, network_id))
				.expect("Transaction was signed by AccountsProvider; it never produces invalid signatures; qed")
		})
	};
	Ok(signed_transaction)
}

pub fn sign_and_dispatch<C, M>(client: &C, miner: &M, accounts: &AccountProvider, filled: FilledTransactionRequest, password: SignWith) -> Result<WithToken<H256>, Error>
	where C: MiningBlockChainClient, M: MinerService
{

	let network_id = client.signing_network_id();
	let condition = filled.condition.clone();
	let signed_transaction = sign_no_dispatch(client, miner, accounts, filled, password)?;

	let (signed_transaction, token) = match signed_transaction {
		WithToken::No(signed_transaction) => (signed_transaction, None),
		WithToken::Yes(signed_transaction, token) => (signed_transaction, Some(token)),
	};

	trace!(target: "miner", "send_transaction: dispatching tx: {} for network ID {:?}", rlp::encode(&signed_transaction).to_vec().pretty(), network_id);
	let pending_transaction = PendingTransaction::new(signed_transaction, condition.map(Into::into));
	dispatch_transaction(&*client, &*miner, pending_transaction).map(|hash| {
		match token {
			Some(ref token) => WithToken::Yes(hash, token.clone()),
			None => WithToken::No(hash),
		}
	})
}

pub fn fill_optional_fields<C, M>(request: TransactionRequest, default_sender: Address, client: &C, miner: &M) -> FilledTransactionRequest
	where C: MiningBlockChainClient, M: MinerService
{
	FilledTransactionRequest {
		from: request.from.unwrap_or(default_sender),
		used_default_from: request.from.is_none(),
		to: request.to,
		nonce: request.nonce,
		gas_price: request.gas_price.unwrap_or_else(|| default_gas_price(client, miner)),
		gas: request.gas.unwrap_or_else(|| miner.sensible_gas_limit()),
		value: request.value.unwrap_or_else(|| 0.into()),
		data: request.data.unwrap_or_else(Vec::new),
		condition: request.condition,
	}
}

pub fn default_gas_price<C, M>(client: &C, miner: &M) -> U256
	where C: MiningBlockChainClient, M: MinerService
{
	client.gas_price_median(100).unwrap_or_else(|| miner.sensible_gas_price())
}

pub fn from_rpc<C, M>(payload: RpcConfirmationPayload, default_account: Address, client: &C, miner: &M) -> ConfirmationPayload
	where C: MiningBlockChainClient, M: MinerService {

	match payload {
		RpcConfirmationPayload::SendTransaction(request) => {
			ConfirmationPayload::SendTransaction(fill_optional_fields(request.into(), default_account, client, miner))
		},
		RpcConfirmationPayload::SignTransaction(request) => {
			ConfirmationPayload::SignTransaction(fill_optional_fields(request.into(), default_account, client, miner))
		},
		RpcConfirmationPayload::Decrypt(RpcDecryptRequest { address, msg }) => {
			ConfirmationPayload::Decrypt(address.into(), msg.into())
		},
		RpcConfirmationPayload::Signature(RpcSignRequest { address, data }) => {
			ConfirmationPayload::Signature(address.into(), data.into())
		},
	}
}
