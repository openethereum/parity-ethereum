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

//! Ethereum rpc interface implementation.

macro_rules! take_weak {
	($weak: expr) => {
		match $weak.upgrade() {
			Some(arc) => arc,
			None => return Err(Error::internal_error())
		}
	}
}

macro_rules! rpc_unimplemented {
	() => (Err(Error::internal_error()))
}

mod web3;
mod eth;
mod net;
mod personal;
mod ethcore;
mod traces;
mod rpc;

pub use self::web3::Web3Client;
pub use self::eth::{EthClient, EthFilterClient};
pub use self::net::NetClient;
pub use self::personal::PersonalClient;
pub use self::ethcore::EthcoreClient;
pub use self::traces::TracesClient;
pub use self::rpc::RpcClient;

use v1::types::TransactionRequest;
use std::sync::Weak;
use ethminer::{AccountDetails, MinerService};
use ethcore::client::BlockChainClient;
use ethcore::transaction::{Action, SignedTransaction, Transaction};
use util::numbers::*;
use util::rlp::encode;
use util::bytes::ToPretty;
use jsonrpc_core::{Error, to_value, Value};

fn dispatch_transaction<C, M>(client: &C, miner: &M, signed_transaction: SignedTransaction) -> Result<Value, Error>
	where C: BlockChainClient, M: MinerService {
	let hash = signed_transaction.hash();

	let import = miner.import_own_transaction(client, signed_transaction, |a: &Address| {
		AccountDetails {
			nonce: client.latest_nonce(&a),
			balance: client.latest_balance(&a),
		}
	});

	to_value(&import.map(|_| hash).unwrap_or(H256::zero()))
}

fn sign_and_dispatch<C, M>(client: &Weak<C>, miner: &Weak<M>, request: TransactionRequest, secret: H256) -> Result<Value, Error>
	where C: BlockChainClient, M: MinerService {
	let client = take_weak!(client);
	let miner = take_weak!(miner);

	let signed_transaction = {
		Transaction {
			nonce: request.nonce
				.or_else(|| miner
						 .last_nonce(&request.from)
						 .map(|nonce| nonce + U256::one()))
				.unwrap_or_else(|| client.latest_nonce(&request.from)),

			action: request.to.map_or(Action::Create, Action::Call),
			gas: request.gas.unwrap_or_else(|| miner.sensible_gas_limit()),
			gas_price: request.gas_price.unwrap_or_else(|| miner.sensible_gas_price()),
			value: request.value.unwrap_or_else(U256::zero),
			data: request.data.map_or_else(Vec::new, |b| b.to_vec()),
		}.sign(&secret)
	};

	trace!(target: "miner", "send_transaction: dispatching tx: {}", encode(&signed_transaction).to_vec().pretty());
	dispatch_transaction(&*client, &*miner, signed_transaction)
}