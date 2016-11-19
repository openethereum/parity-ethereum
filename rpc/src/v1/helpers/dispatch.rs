// Copyright 2015, 2016 Ethcore (UK) Ltd.
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

use rlp;
use util::{Address, H256, U256, Uint, Bytes};
use util::bytes::ToPretty;

use ethkey::Signature;
use ethcore::miner::MinerService;
use ethcore::client::MiningBlockChainClient;
use ethcore::transaction::{Action, SignedTransaction, Transaction};
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

pub fn execute<C, M>(client: &C, miner: &M, accounts: &AccountProvider, payload: ConfirmationPayload, pass: Option<String>) -> Result<ConfirmationResponse, Error>
	where C: MiningBlockChainClient, M: MinerService
{
	match payload {
		ConfirmationPayload::SendTransaction(request) => {
			sign_and_dispatch(client, miner, accounts, request, pass)
				.map(RpcH256::from)
				.map(ConfirmationResponse::SendTransaction)
		},
		ConfirmationPayload::SignTransaction(request) => {
			sign_no_dispatch(client, miner, accounts, request, pass)
				.map(RpcRichRawTransaction::from)
				.map(ConfirmationResponse::SignTransaction)
		},
		ConfirmationPayload::Signature(address, hash) => {
			signature(accounts, address, hash, pass)
				.map(RpcH520::from)
				.map(ConfirmationResponse::Signature)
		},
		ConfirmationPayload::Decrypt(address, data) => {
			decrypt(accounts, address, data, pass)
				.map(RpcBytes)
				.map(ConfirmationResponse::Decrypt)
		},
	}
}

fn signature(accounts: &AccountProvider, address: Address, hash: H256, password: Option<String>) -> Result<Signature, Error> {
	accounts.sign(address, password.clone(), hash).map_err(|e| match password {
		Some(_) => errors::from_password_error(e),
		None => errors::from_signing_error(e),
	})
}

fn decrypt(accounts: &AccountProvider, address: Address, msg: Bytes, password: Option<String>) -> Result<Bytes, Error> {
	accounts.decrypt(address, password.clone(), &DEFAULT_MAC, &msg)
		.map_err(|e| match password {
			Some(_) => errors::from_password_error(e),
			None => errors::from_signing_error(e),
		})
}

pub fn dispatch_transaction<C, M>(client: &C, miner: &M, signed_transaction: SignedTransaction) -> Result<H256, Error>
	where C: MiningBlockChainClient, M: MinerService {
	let hash = signed_transaction.hash();

	miner.import_own_transaction(client, signed_transaction)
		.map_err(errors::from_transaction_error)
		.map(|_| hash)
}

pub fn sign_no_dispatch<C, M>(client: &C, miner: &M, accounts: &AccountProvider, filled: FilledTransactionRequest, password: Option<String>) -> Result<SignedTransaction, Error>
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
		let signature = try!(signature(accounts, address, hash, password));
		t.with_signature(signature, network_id)
	};
	Ok(signed_transaction)
}

pub fn sign_and_dispatch<C, M>(client: &C, miner: &M, accounts: &AccountProvider, filled: FilledTransactionRequest, password: Option<String>) -> Result<H256, Error>
	where C: MiningBlockChainClient, M: MinerService
{

	let network_id = client.signing_network_id();
	let signed_transaction = try!(sign_no_dispatch(client, miner, accounts, filled, password));

	trace!(target: "miner", "send_transaction: dispatching tx: {} for network ID {:?}", rlp::encode(&signed_transaction).to_vec().pretty(), network_id);
	dispatch_transaction(&*client, &*miner, signed_transaction)
}

pub fn fill_optional_fields<C, M>(request: TransactionRequest, client: &C, miner: &M) -> FilledTransactionRequest
	where C: MiningBlockChainClient, M: MinerService
{
	FilledTransactionRequest {
		from: request.from,
		to: request.to,
		nonce: request.nonce,
		gas_price: request.gas_price.unwrap_or_else(|| default_gas_price(client, miner)),
		gas: request.gas.unwrap_or_else(|| miner.sensible_gas_limit()),
		value: request.value.unwrap_or_else(|| 0.into()),
		data: request.data.unwrap_or_else(Vec::new),
	}
}

pub fn default_gas_price<C, M>(client: &C, miner: &M) -> U256
	where C: MiningBlockChainClient, M: MinerService
{
	client.gas_price_median(100).unwrap_or_else(|| miner.sensible_gas_price())
}

pub fn from_rpc<C, M>(payload: RpcConfirmationPayload, client: &C, miner: &M) -> ConfirmationPayload
	where C: MiningBlockChainClient, M: MinerService {

	match payload {
		RpcConfirmationPayload::SendTransaction(request) => {
			ConfirmationPayload::SendTransaction(fill_optional_fields(request.into(), client, miner))
		},
		RpcConfirmationPayload::SignTransaction(request) => {
			ConfirmationPayload::SignTransaction(fill_optional_fields(request.into(), client, miner))
		},
		RpcConfirmationPayload::Decrypt(RpcDecryptRequest { address, msg }) => {
			ConfirmationPayload::Decrypt(address.into(), msg.into())
		},
		RpcConfirmationPayload::Signature(RpcSignRequest { address, hash }) => {
			ConfirmationPayload::Signature(address.into(), hash.into())
		},
	}
}
