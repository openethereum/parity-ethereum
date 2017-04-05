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

//! Eth RPC interface for the light client.

// TODO: remove when complete.
#![allow(unused_imports, unused_variables)]

use std::sync::Arc;

use jsonrpc_core::Error;
use jsonrpc_macros::Trailing;

use light::cache::Cache as LightDataCache;
use light::client::Client as LightClient;
use light::{cht, TransactionQueue};
use light::on_demand::{request, OnDemand};

use ethcore::account_provider::{AccountProvider, DappId};
use ethcore::basic_account::BasicAccount;
use ethcore::encoded;
use ethcore::executed::{Executed, ExecutionError};
use ethcore::ids::BlockId;
use ethcore::transaction::{Action, SignedTransaction, Transaction as EthTransaction};
use ethsync::LightSync;
use rlp::UntrustedRlp;
use util::sha3::{SHA3_NULL_RLP, SHA3_EMPTY_LIST_RLP};
use util::{RwLock, Mutex, Uint, U256};

use futures::{future, Future, BoxFuture, IntoFuture};
use futures::sync::oneshot;

use v1::helpers::{CallRequest as CRequest, errors, limit_logs, dispatch};
use v1::helpers::block_import::is_major_importing;
use v1::helpers::light_fetch::LightFetch;
use v1::traits::Eth;
use v1::types::{
	RichBlock, Block, BlockTransactions, BlockNumber, Bytes, SyncStatus, SyncInfo,
	Transaction, CallRequest, Index, Filter, Log, Receipt, Work,
	H64 as RpcH64, H256 as RpcH256, H160 as RpcH160, U256 as RpcU256,
};
use v1::metadata::Metadata;

use util::Address;

/// Light client `ETH` RPC.
pub struct EthClient {
	sync: Arc<LightSync>,
	client: Arc<LightClient>,
	on_demand: Arc<OnDemand>,
	transaction_queue: Arc<RwLock<TransactionQueue>>,
	accounts: Arc<AccountProvider>,
	cache: Arc<Mutex<LightDataCache>>,
}


impl EthClient {
	/// Create a new `EthClient` with a handle to the light sync instance, client,
	/// and on-demand request service, which is assumed to be attached as a handler.
	pub fn new(
		sync: Arc<LightSync>,
		client: Arc<LightClient>,
		on_demand: Arc<OnDemand>,
		transaction_queue: Arc<RwLock<TransactionQueue>>,
		accounts: Arc<AccountProvider>,
		cache: Arc<Mutex<LightDataCache>>,
	) -> Self {
		EthClient {
			sync: sync,
			client: client,
			on_demand: on_demand,
			transaction_queue: transaction_queue,
			accounts: accounts,
			cache: cache,
		}
	}

	/// Create a light data fetcher instance.
	fn fetcher(&self) -> LightFetch {
		LightFetch {
			client: self.client.clone(),
			on_demand: self.on_demand.clone(),
			sync: self.sync.clone(),
			cache: self.cache.clone(),

		}
	}
}

impl Eth for EthClient {
	type Metadata = Metadata;

	fn protocol_version(&self) -> Result<String, Error> {
		Ok(format!("{}", ::light::net::MAX_PROTOCOL_VERSION))
	}

	fn syncing(&self) -> Result<SyncStatus, Error> {
		if self.sync.is_major_importing() {
			let chain_info = self.client.chain_info();
			let current_block = U256::from(chain_info.best_block_number);
			let highest_block = self.sync.highest_block().map(U256::from)
				.unwrap_or_else(|| current_block.clone());

			Ok(SyncStatus::Info(SyncInfo {
				starting_block: U256::from(self.sync.start_block()).into(),
				current_block: current_block.into(),
				highest_block: highest_block.into(),
				warp_chunks_amount: None,
				warp_chunks_processed: None,
			}))
		} else {
			Ok(SyncStatus::None)
		}
	}

	fn author(&self, _meta: Self::Metadata) -> BoxFuture<RpcH160, Error> {
		future::ok(Default::default()).boxed()
	}

	fn is_mining(&self) -> Result<bool, Error> {
		Ok(false)
	}

	fn hashrate(&self) -> Result<RpcU256, Error> {
		Ok(Default::default())
	}

	fn gas_price(&self) -> Result<RpcU256, Error> {
		Ok(Default::default())
	}

	fn accounts(&self, meta: Metadata) -> BoxFuture<Vec<RpcH160>, Error> {
		let dapp: DappId = meta.dapp_id().into();

		let accounts = self.accounts
			.note_dapp_used(dapp.clone())
			.and_then(|_| self.accounts.dapp_addresses(dapp))
			.map_err(|e| errors::account("Could not fetch accounts.", e))
			.map(|accs| accs.into_iter().map(Into::<RpcH160>::into).collect());

		future::done(accounts).boxed()
	}

	fn block_number(&self) -> Result<RpcU256, Error> {
		Ok(self.client.chain_info().best_block_number.into())
	}

	fn balance(&self, address: RpcH160, num: Trailing<BlockNumber>) -> BoxFuture<RpcU256, Error> {
		self.fetcher().account(address.into(), num.0.into())
			.map(|acc| acc.map_or(0.into(), |a| a.balance).into()).boxed()
	}

	fn storage_at(&self, _address: RpcH160, _key: RpcU256, _num: Trailing<BlockNumber>) -> BoxFuture<RpcH256, Error> {
		future::err(errors::unimplemented(None)).boxed()
	}

	fn block_by_hash(&self, hash: RpcH256, include_txs: bool) -> BoxFuture<Option<RichBlock>, Error> {
		future::err(errors::unimplemented(None)).boxed()
	}

	fn block_by_number(&self, num: BlockNumber, include_txs: bool) -> BoxFuture<Option<RichBlock>, Error> {
		future::err(errors::unimplemented(None)).boxed()
	}

	fn transaction_count(&self, address: RpcH160, num: Trailing<BlockNumber>) -> BoxFuture<RpcU256, Error> {
		self.fetcher().account(address.into(), num.0.into())
			.map(|acc| acc.map_or(0.into(), |a| a.nonce).into()).boxed()
	}

	fn block_transaction_count_by_hash(&self, hash: RpcH256) -> BoxFuture<Option<RpcU256>, Error> {
		let (sync, on_demand) = (self.sync.clone(), self.on_demand.clone());

		self.fetcher().header(BlockId::Hash(hash.into())).and_then(move |hdr| {
			let hdr = match hdr {
				None => return future::ok(None).boxed(),
				Some(hdr) => hdr,
			};

			if hdr.transactions_root() == SHA3_NULL_RLP {
				future::ok(Some(U256::from(0).into())).boxed()
			} else {
				sync.with_context(|ctx| on_demand.block(ctx, request::Body::new(hdr)))
					.map(|x| x.map(|b| Some(U256::from(b.transactions_count()).into())))
					.map(|x| x.map_err(errors::on_demand_cancel).boxed())
					.unwrap_or_else(|| future::err(errors::network_disabled()).boxed())
			}
		}).boxed()
	}

	fn block_transaction_count_by_number(&self, num: BlockNumber) -> BoxFuture<Option<RpcU256>, Error> {
		let (sync, on_demand) = (self.sync.clone(), self.on_demand.clone());

		self.fetcher().header(num.into()).and_then(move |hdr| {
			let hdr = match hdr {
				None => return future::ok(None).boxed(),
				Some(hdr) => hdr,
			};

			if hdr.transactions_root() == SHA3_NULL_RLP {
				future::ok(Some(U256::from(0).into())).boxed()
			} else {
				sync.with_context(|ctx| on_demand.block(ctx, request::Body::new(hdr)))
					.map(|x| x.map(|b| Some(U256::from(b.transactions_count()).into())))
					.map(|x| x.map_err(errors::on_demand_cancel).boxed())
					.unwrap_or_else(|| future::err(errors::network_disabled()).boxed())
			}
		}).boxed()
	}

	fn block_uncles_count_by_hash(&self, hash: RpcH256) -> BoxFuture<Option<RpcU256>, Error> {
		let (sync, on_demand) = (self.sync.clone(), self.on_demand.clone());

		self.fetcher().header(BlockId::Hash(hash.into())).and_then(move |hdr| {
			let hdr = match hdr {
				None => return future::ok(None).boxed(),
				Some(hdr) => hdr,
			};

			if hdr.uncles_hash() == SHA3_EMPTY_LIST_RLP {
				future::ok(Some(U256::from(0).into())).boxed()
			} else {
				sync.with_context(|ctx| on_demand.block(ctx, request::Body::new(hdr)))
					.map(|x| x.map(|b| Some(U256::from(b.uncles_count()).into())))
					.map(|x| x.map_err(errors::on_demand_cancel).boxed())
					.unwrap_or_else(|| future::err(errors::network_disabled()).boxed())
			}
		}).boxed()
	}

	fn block_uncles_count_by_number(&self, num: BlockNumber) -> BoxFuture<Option<RpcU256>, Error> {
		let (sync, on_demand) = (self.sync.clone(), self.on_demand.clone());

		self.fetcher().header(num.into()).and_then(move |hdr| {
			let hdr = match hdr {
				None => return future::ok(None).boxed(),
				Some(hdr) => hdr,
			};

			if hdr.uncles_hash() == SHA3_EMPTY_LIST_RLP {
				future::ok(Some(U256::from(0).into())).boxed()
			} else {
				sync.with_context(|ctx| on_demand.block(ctx, request::Body::new(hdr)))
					.map(|x| x.map(|b| Some(U256::from(b.uncles_count()).into())))
					.map(|x| x.map_err(errors::on_demand_cancel).boxed())
					.unwrap_or_else(|| future::err(errors::network_disabled()).boxed())
			}
		}).boxed()
	}

	fn code_at(&self, address: RpcH160, num: Trailing<BlockNumber>) -> BoxFuture<Bytes, Error> {
		future::err(errors::unimplemented(None)).boxed()
	}

	fn send_raw_transaction(&self, raw: Bytes) -> Result<RpcH256, Error> {
		let best_header = self.client.best_block_header().decode();

		UntrustedRlp::new(&raw.into_vec()).as_val()
			.map_err(errors::from_rlp_error)
			.and_then(|tx| {
				self.client.engine().verify_transaction_basic(&tx, &best_header)
					.map_err(errors::from_transaction_error)?;

				let signed = SignedTransaction::new(tx).map_err(errors::from_transaction_error)?;
				let hash = signed.hash();

				self.transaction_queue.write().import(signed.into())
					.map(|_| hash)
					.map_err(Into::into)
					.map_err(errors::from_transaction_error)
			})
			.map(Into::into)
	}

	fn submit_transaction(&self, raw: Bytes) -> Result<RpcH256, Error> {
		self.send_raw_transaction(raw)
	}

	fn call(&self, req: CallRequest, num: Trailing<BlockNumber>) -> BoxFuture<Bytes, Error> {
		self.fetcher().proved_execution(req, num).and_then(|res| {
			match res {
				Ok(exec) => Ok(exec.output.into()),
				Err(e) => Err(errors::execution(e)),
			}
		}).boxed()
	}

	fn estimate_gas(&self, req: CallRequest, num: Trailing<BlockNumber>) -> BoxFuture<RpcU256, Error> {
		// TODO: binary chop for more accurate estimates.
		self.fetcher().proved_execution(req, num).and_then(|res| {
			match res {
				Ok(exec) => Ok((exec.refunded + exec.gas_used).into()),
				Err(e) => Err(errors::execution(e)),
			}
		}).boxed()
	}

	fn transaction_by_hash(&self, hash: RpcH256) -> Result<Option<Transaction>, Error> {
		Err(errors::unimplemented(None))
	}

	fn transaction_by_block_hash_and_index(&self, hash: RpcH256, idx: Index) -> Result<Option<Transaction>, Error> {
		Err(errors::unimplemented(None))
	}

	fn transaction_by_block_number_and_index(&self, num: BlockNumber, idx: Index) -> Result<Option<Transaction>, Error> {
		Err(errors::unimplemented(None))
	}

	fn transaction_receipt(&self, hash: RpcH256) -> Result<Option<Receipt>, Error> {
		Err(errors::unimplemented(None))
	}

	fn uncle_by_block_hash_and_index(&self, hash: RpcH256, idx: Index) -> Result<Option<RichBlock>, Error> {
		Err(errors::unimplemented(None))
	}

	fn uncle_by_block_number_and_index(&self, num: BlockNumber, idx: Index) -> Result<Option<RichBlock>, Error> {
		Err(errors::unimplemented(None))
	}

	fn compilers(&self) -> Result<Vec<String>, Error> {
		Err(errors::deprecated("Compilation functionality is deprecated.".to_string()))

	}

	fn compile_lll(&self, _: String) -> Result<Bytes, Error> {
		Err(errors::deprecated("Compilation of LLL via RPC is deprecated".to_string()))
	}

	fn compile_serpent(&self, _: String) -> Result<Bytes, Error> {
		Err(errors::deprecated("Compilation of Serpent via RPC is deprecated".to_string()))
	}

	fn compile_solidity(&self, _: String) -> Result<Bytes, Error> {
		Err(errors::deprecated("Compilation of Solidity via RPC is deprecated".to_string()))
	}

	fn logs(&self, _filter: Filter) -> Result<Vec<Log>, Error> {
		Err(errors::unimplemented(None))
	}

	fn work(&self, _timeout: Trailing<u64>) -> Result<Work, Error> {
		Err(errors::unimplemented(None))
	}

	fn submit_work(&self, _nonce: RpcH64, _pow_hash: RpcH256, _mix_hash: RpcH256) -> Result<bool, Error> {
		Err(errors::unimplemented(None))
	}

	fn submit_hashrate(&self, _rate: RpcU256, _id: RpcH256) -> Result<bool, Error> {
		Err(errors::unimplemented(None))
	}
}
