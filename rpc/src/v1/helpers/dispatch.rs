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

use util::{Address, H256, U256, Uint, Bytes};
use util::bytes::ToPretty;
use ethkey::Signature;
use ethcore::miner::MinerService;
use ethcore::client::MiningBlockChainClient;
use ethcore::transaction::{Action, SignedTransaction, Transaction};
use ethcore::account_provider::AccountProvider;
use jsonrpc_core::{Error, Value, to_value};
use v1::helpers::TransactionRequest;
use v1::types::{H256 as RpcH256, H520 as RpcH520, Bytes as RpcBytes};
use v1::helpers::errors;

pub const DEFAULT_MAC: [u8; 2] = [0, 0];

pub fn dispatch_transaction<C, M>(client: &C, miner: &M, signed_transaction: SignedTransaction) -> Result<RpcH256, Error>
	where C: MiningBlockChainClient, M: MinerService {
	let hash = RpcH256::from(signed_transaction.hash());

	miner.import_own_transaction(client, signed_transaction)
		.map_err(errors::from_transaction_error)
		.map(|_| hash)
}

fn signature(accounts: &AccountProvider, address: Address, password: Option<String>, hash: H256) -> Result<Signature, Error> {
	accounts.sign(address, password.clone(), hash).map_err(|e| match password {
		Some(_) => errors::from_password_error(e),
		None => errors::from_signing_error(e),
	})
}

pub fn sign(accounts: &AccountProvider, address: Address, password: Option<String>, hash: H256) -> Result<Value, Error> {
	signature(accounts, address, password, hash)
		.map(RpcH520::from)
		.map(to_value)
}

pub fn decrypt(accounts: &AccountProvider, address: Address, password: Option<String>, msg: Bytes) -> Result<Value, Error> {
	accounts.decrypt(address, password.clone(), &DEFAULT_MAC, &msg)
		.map_err(|e| match password {
			Some(_) => errors::from_password_error(e),
			None => errors::from_signing_error(e),
		})
		.map(RpcBytes::from)
		.map(to_value)
}

pub fn sign_and_dispatch<C, M>(client: &C, miner: &M, accounts: &AccountProvider, request: TransactionRequest, password: Option<String>) -> Result<Value, Error>
	where C: MiningBlockChainClient, M: MinerService {

	let address = request.from;
	let signed_transaction = {
		let t = prepare_transaction(client, miner, request);
		let hash = t.hash();
		let signature = try!(signature(accounts, address, password, hash));
		t.with_signature(signature)
	};

	trace!(target: "miner", "send_transaction: dispatching tx: {}", ::rlp::encode(&signed_transaction).to_vec().pretty());
	dispatch_transaction(&*client, &*miner, signed_transaction).map(to_value)
}

fn prepare_transaction<C, M>(client: &C, miner: &M, request: TransactionRequest) -> Transaction where C: MiningBlockChainClient, M: MinerService {
	Transaction {
		nonce: request.nonce
			.or_else(|| miner
				.last_nonce(&request.from)
				.map(|nonce| nonce + U256::one()))
			.unwrap_or_else(|| client.latest_nonce(&request.from)),

		action: request.to.map_or(Action::Create, Action::Call),
		gas: request.gas.unwrap_or_else(|| miner.sensible_gas_limit()),
		gas_price: request.gas_price.unwrap_or_else(|| default_gas_price(client, miner)),
		value: request.value.unwrap_or_else(U256::zero),
		data: request.data.map_or_else(Vec::new, |b| b.to_vec()),
	}
}

pub fn default_gas_price<C, M>(client: &C, miner: &M) -> U256 where C: MiningBlockChainClient, M: MinerService {
	client
		.gas_price_statistics(100, 8)
		.map(|x| x[4])
		.unwrap_or_else(|_| miner.sensible_gas_price())
}
