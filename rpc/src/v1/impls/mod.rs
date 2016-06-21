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
mod eth_filter;
mod eth_signing;
mod net;
mod personal;
mod personal_signer;
mod ethcore;
mod ethcore_set;
mod traces;
mod rpc;

pub use self::web3::Web3Client;
pub use self::eth::EthClient;
pub use self::eth_filter::EthFilterClient;
pub use self::eth_signing::{EthSigningUnsafeClient, EthSigningQueueClient};
pub use self::net::NetClient;
pub use self::personal::PersonalClient;
pub use self::personal_signer::SignerClient;
pub use self::ethcore::EthcoreClient;
pub use self::ethcore_set::EthcoreSetClient;
pub use self::traces::TracesClient;
pub use self::rpc::RpcClient;

use v1::types::TransactionRequest;
use ethcore::error::Error as EthcoreError;
use ethcore::miner::{AccountDetails, MinerService};
use ethcore::client::MiningBlockChainClient;
use ethcore::transaction::{Action, SignedTransaction, Transaction};
use ethcore::account_provider::{AccountProvider, Error as AccountError};
use util::numbers::*;
use util::rlp::encode;
use util::bytes::ToPretty;
use jsonrpc_core::{Error, ErrorCode, Value, to_value};

mod error_codes {
	// NOTE [ToDr] Codes from [-32099, -32000]
	pub const UNSUPPORTED_REQUEST_CODE: i64 = -32000;
	pub const NO_WORK_CODE: i64 = -32001;
	pub const UNKNOWN_ERROR: i64 = -32002;
	pub const TRANSACTION_ERROR: i64 = -32010;
	pub const ACCOUNT_LOCKED: i64 = -32020;
	pub const SIGNER_DISABLED: i64 = -32030;
}

fn dispatch_transaction<C, M>(client: &C, miner: &M, signed_transaction: SignedTransaction) -> Result<Value, Error>
	where C: MiningBlockChainClient, M: MinerService {
	let hash = signed_transaction.hash();

	let import = miner.import_own_transaction(client, signed_transaction, |a: &Address| {
		AccountDetails {
			nonce: client.latest_nonce(&a),
			balance: client.latest_balance(&a),
		}
	});

	import
		.map_err(transaction_error)
		.and_then(|_| to_value(&hash))
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
		gas_price: request.gas_price.unwrap_or_else(|| miner.sensible_gas_price()),
		value: request.value.unwrap_or_else(U256::zero),
		data: request.data.map_or_else(Vec::new, |b| b.to_vec()),
	}
}

fn unlock_sign_and_dispatch<C, M>(client: &C, miner: &M, request: TransactionRequest, account_provider: &AccountProvider, address: Address, password: String) -> Result<Value, Error>
	where C: MiningBlockChainClient, M: MinerService {

	let signed_transaction = {
		let t = prepare_transaction(client, miner, request);
		let hash = t.hash();
		let signature = try!(account_provider.sign_with_password(address, password, hash).map_err(signing_error));
		t.with_signature(signature)
	};

	trace!(target: "miner", "send_transaction: dispatching tx: {}", encode(&signed_transaction).to_vec().pretty());
	dispatch_transaction(&*client, &*miner, signed_transaction)
}

fn sign_and_dispatch<C, M>(client: &C, miner: &M, request: TransactionRequest, account_provider: &AccountProvider, address: Address) -> Result<Value, Error>
	where C: MiningBlockChainClient, M: MinerService {

	let signed_transaction = {
		let t = prepare_transaction(client, miner, request);
		let hash = t.hash();
		let signature = try!(account_provider.sign(address, hash).map_err(signing_error));
		t.with_signature(signature)
	};

	trace!(target: "miner", "send_transaction: dispatching tx: {}", encode(&signed_transaction).to_vec().pretty());
	dispatch_transaction(&*client, &*miner, signed_transaction)
}

fn signing_error(error: AccountError) -> Error {
	Error {
		code: ErrorCode::ServerError(error_codes::ACCOUNT_LOCKED),
		message: "Your account is locked. Unlock the account via CLI, personal_unlockAccount or use Trusted Signer.".into(),
		data: Some(Value::String(format!("{:?}", error))),
	}
}

fn transaction_error(error: EthcoreError) -> Error {
	use ethcore::error::TransactionError::*;

	if let EthcoreError::Transaction(e) = error {
		let msg = match e {
			AlreadyImported => "Transaction with the same hash was already imported.".into(),
			Old => "Transaction nonce is too low. Try incrementing the nonce.".into(),
			TooCheapToReplace => {
				"Transaction fee is too low. There is another transaction with same nonce in the queue. Try increasing the fee or incrementing the nonce.".into()
			},
			LimitReached => {
				"There is too many transactions in the queue. Your transaction was dropped due to limit. Try increasing the fee.".into()
			},
			InsufficientGasPrice { minimal, got } => {
				format!("Transaction fee is to low. It does not satisfy your node's minimal fee (minimal: {}, got: {}). Try increasing the fee.", minimal, got)
			},
			InsufficientBalance { balance, cost } => {
				format!("Insufficient funds. Account you try to send transaction from does not have enough funds. Required {} and got: {}.", cost, balance)
			},
			GasLimitExceeded { limit, got } => {
				format!("Transaction cost exceeds current gas limit. Limit: {}, got: {}. Try decreasing supplied gas.", limit, got)
			},
			InvalidGasLimit(_) => "Supplied gas is beyond limit.".into(),
			DAORescue => "Transaction removes funds from a DAO.".into(),
		};
		Error {
			code: ErrorCode::ServerError(error_codes::TRANSACTION_ERROR),
			message: msg,
			data: None,
		}
	} else {
		Error {
			code: ErrorCode::ServerError(error_codes::UNKNOWN_ERROR),
			message: "Unknown error when sending transaction.".into(),
			data: Some(Value::String(format!("{:?}", error))),
		}
	}
}
