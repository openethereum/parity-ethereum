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

//! Parity-specific rpc implementation.
use std::sync::Arc;
use std::collections::{BTreeMap, HashSet};
use futures::{future, Future, BoxFuture};

use util::RotatingLogger;
use util::misc::version_data;

use crypto::ecies;
use ethkey::{Brain, Generator};
use ethstore::random_phrase;
use ethsync::LightSyncProvider;
use ethcore::account_provider::AccountProvider;

use jsonrpc_core::Error;
use jsonrpc_macros::Trailing;
use v1::helpers::{errors, SigningQueue, SignerService, NetworkSettings};
use v1::helpers::dispatch::{LightDispatcher, DEFAULT_MAC};
use v1::metadata::Metadata;
use v1::traits::Parity;
use v1::types::{
	Bytes, U256, H160, H256, H512,
	Peers, Transaction, RpcSettings, Histogram,
	TransactionStats, LocalTransactionStatus,
	BlockNumber, ConsensusCapability, VersionInfo,
	OperationsInfo, DappId, ChainStatus,
	AccountInfo, HwAccountInfo
};

/// Parity implementation for light client.
pub struct ParityClient {
	light_dispatch: Arc<LightDispatcher>,
	accounts: Arc<AccountProvider>,
	logger: Arc<RotatingLogger>,
	settings: Arc<NetworkSettings>,
	signer: Option<Arc<SignerService>>,
	dapps_interface: Option<String>,
	dapps_port: Option<u16>,
}

impl ParityClient {
	/// Creates new `ParityClient`.
	pub fn new(
		light_dispatch: Arc<LightDispatcher>,
		accounts: Arc<AccountProvider>,
		logger: Arc<RotatingLogger>,
		settings: Arc<NetworkSettings>,
		signer: Option<Arc<SignerService>>,
		dapps_interface: Option<String>,
		dapps_port: Option<u16>,
	) -> Self {
		ParityClient {
			light_dispatch: light_dispatch,
			accounts: accounts,
			logger: logger,
			settings: settings,
			signer: signer,
			dapps_interface: dapps_interface,
			dapps_port: dapps_port,
		}
	}
}

impl Parity for ParityClient {
	type Metadata = Metadata;

	fn accounts_info(&self, dapp: Trailing<DappId>) -> Result<BTreeMap<H160, AccountInfo>, Error> {
		let dapp = dapp.0;

		let store = &self.accounts;
		let dapp_accounts = store
			.note_dapp_used(dapp.clone().into())
			.and_then(|_| store.dapp_addresses(dapp.into()))
			.map_err(|e| errors::internal("Could not fetch accounts.", e))?
			.into_iter().collect::<HashSet<_>>();

		let info = store.accounts_info().map_err(|e| errors::account("Could not fetch account info.", e))?;
		let other = store.addresses_info();

		Ok(info
			.into_iter()
			.chain(other.into_iter())
			.filter(|&(ref a, _)| dapp_accounts.contains(a))
			.map(|(a, v)| (H160::from(a), AccountInfo { name: v.name }))
			.collect()
		)
	}

	fn hardware_accounts_info(&self) -> Result<BTreeMap<H160, HwAccountInfo>, Error> {
		let store = &self.accounts;
		let info = store.hardware_accounts_info().map_err(|e| errors::account("Could not fetch account info.", e))?;
		Ok(info
			.into_iter()
			.map(|(a, v)| (H160::from(a), HwAccountInfo { name: v.name, manufacturer: v.meta }))
			.collect()
		)
	}

	fn default_account(&self, meta: Self::Metadata) -> BoxFuture<H160, Error> {
		let dapp_id = meta.dapp_id();
		future::ok(self.accounts
			.dapp_addresses(dapp_id.into())
			.ok()
			.and_then(|accounts| accounts.get(0).cloned())
			.map(|acc| acc.into())
			.unwrap_or_default()
		).boxed()
	}

	fn transactions_limit(&self) -> Result<usize, Error> {
		Ok(usize::max_value())
	}

	fn min_gas_price(&self) -> Result<U256, Error> {
		Ok(U256::default())
	}

	fn extra_data(&self) -> Result<Bytes, Error> {
		Ok(Bytes::default())
	}

	fn gas_floor_target(&self) -> Result<U256, Error> {
		Ok(U256::default())
	}

	fn gas_ceil_target(&self) -> Result<U256, Error> {
		Ok(U256::default())
	}

	fn dev_logs(&self) -> Result<Vec<String>, Error> {
		let logs = self.logger.logs();
		Ok(logs.as_slice().to_owned())
	}

	fn dev_logs_levels(&self) -> Result<String, Error> {
		Ok(self.logger.levels().to_owned())
	}

	fn net_chain(&self) -> Result<String, Error> {
		Ok(self.settings.chain.clone())
	}

	fn net_peers(&self) -> Result<Peers, Error> {
		let peers = self.light_dispatch.sync.peers().into_iter().map(Into::into).collect();
		let peer_numbers = self.light_dispatch.sync.peer_numbers();

		Ok(Peers {
			active: peer_numbers.active,
			connected: peer_numbers.connected,
			max: peer_numbers.max as u32,
			peers: peers,
		})
	}

	fn net_port(&self) -> Result<u16, Error> {
		Ok(self.settings.network_port)
	}

	fn node_name(&self) -> Result<String, Error> {
		Ok(self.settings.name.clone())
	}

	fn registry_address(&self) -> Result<Option<H160>, Error> {
		Err(errors::light_unimplemented(None))
	}

	fn rpc_settings(&self) -> Result<RpcSettings, Error> {
		Ok(RpcSettings {
			enabled: self.settings.rpc_enabled,
			interface: self.settings.rpc_interface.clone(),
			port: self.settings.rpc_port as u64,
		})
	}

	fn default_extra_data(&self) -> Result<Bytes, Error> {
		Ok(Bytes::new(version_data()))
	}

	fn gas_price_histogram(&self) -> BoxFuture<Histogram, Error> {
		self.light_dispatch.gas_price_corpus()
			.and_then(|corpus| corpus.histogram(10).ok_or_else(errors::not_enough_data))
			.map(Into::into)
			.boxed()
	}

	fn unsigned_transactions_count(&self) -> Result<usize, Error> {
		match self.signer {
			None => Err(errors::signer_disabled()),
			Some(ref signer) => Ok(signer.len()),
		}
	}

	fn generate_secret_phrase(&self) -> Result<String, Error> {
		Ok(random_phrase(12))
	}

	fn phrase_to_address(&self, phrase: String) -> Result<H160, Error> {
		Ok(Brain::new(phrase).generate().unwrap().address().into())
	}

	fn list_accounts(&self, _: u64, _: Option<H160>, _: Trailing<BlockNumber>) -> Result<Option<Vec<H160>>, Error> {
		Err(errors::light_unimplemented(None))
	}

	fn list_storage_keys(&self, _: H160, _: u64, _: Option<H256>, _: Trailing<BlockNumber>) -> Result<Option<Vec<H256>>, Error> {
		Err(errors::light_unimplemented(None))
	}

	fn encrypt_message(&self, key: H512, phrase: Bytes) -> Result<Bytes, Error> {
		ecies::encrypt(&key.into(), &DEFAULT_MAC, &phrase.0)
			.map_err(errors::encryption_error)
			.map(Into::into)
	}

	fn pending_transactions(&self) -> Result<Vec<Transaction>, Error> {
		let txq = self.light_dispatch.transaction_queue.read();
		let chain_info = self.light_dispatch.client.chain_info();
		Ok(
			txq.ready_transactions(chain_info.best_block_number, chain_info.best_block_timestamp)
				.into_iter()
				.map(Into::into)
				.collect::<Vec<_>>()
		)
	}

	fn future_transactions(&self) -> Result<Vec<Transaction>, Error> {
		let txq = self.light_dispatch.transaction_queue.read();
		let chain_info = self.light_dispatch.client.chain_info();
		Ok(
			txq.future_transactions(chain_info.best_block_number, chain_info.best_block_timestamp)
				.into_iter()
				.map(Into::into)
				.collect::<Vec<_>>()
		)
	}

	fn pending_transactions_stats(&self) -> Result<BTreeMap<H256, TransactionStats>, Error> {
		let stats = self.light_dispatch.sync.transactions_stats();
		Ok(stats.into_iter()
		   .map(|(hash, stats)| (hash.into(), stats.into()))
		   .collect()
		)
	}

	fn local_transactions(&self) -> Result<BTreeMap<H256, LocalTransactionStatus>, Error> {
		let mut map = BTreeMap::new();
		let chain_info = self.light_dispatch.client.chain_info();
		let (best_num, best_tm) = (chain_info.best_block_number, chain_info.best_block_timestamp);
		let txq = self.light_dispatch.transaction_queue.read();

		for pending in txq.ready_transactions(best_num, best_tm) {
			map.insert(pending.hash().into(), LocalTransactionStatus::Pending);
		}

		for future in txq.future_transactions(best_num, best_tm) {
			map.insert(future.hash().into(), LocalTransactionStatus::Future);
		}

		// TODO: other types?

		Ok(map)
	}

	fn signer_port(&self) -> Result<u16, Error> {
		self.signer
			.clone()
			.and_then(|signer| signer.address())
			.map(|address| address.1)
			.ok_or_else(|| errors::signer_disabled())
	}

	fn dapps_port(&self) -> Result<u16, Error> {
		self.dapps_port
			.ok_or_else(|| errors::dapps_disabled())
	}

	fn dapps_interface(&self) -> Result<String, Error> {
		self.dapps_interface.clone()
			.ok_or_else(|| errors::dapps_disabled())
	}

	fn next_nonce(&self, address: H160) -> BoxFuture<U256, Error> {
		self.light_dispatch.next_nonce(address.into()).map(Into::into).boxed()
	}

	fn mode(&self) -> Result<String, Error> {
		Err(errors::light_unimplemented(None))
	}

	fn enode(&self) -> Result<String, Error> {
		self.light_dispatch.sync.enode().ok_or_else(errors::network_disabled)
	}

	fn consensus_capability(&self) -> Result<ConsensusCapability, Error> {
		Err(errors::light_unimplemented(None))
	}

	fn version_info(&self) -> Result<VersionInfo, Error> {
		Err(errors::light_unimplemented(None))
	}

	fn releases_info(&self) -> Result<Option<OperationsInfo>, Error> {
		Err(errors::light_unimplemented(None))
	}

	fn chain_status(&self) -> Result<ChainStatus, Error> {
		let chain_info = self.light_dispatch.client.chain_info();

		let gap = chain_info.ancient_block_number.map(|x| U256::from(x + 1))
			.and_then(|first| chain_info.first_block_number.map(|last| (first, U256::from(last))));

		Ok(ChainStatus {
			block_gap: gap.map(|(x, y)| (x.into(), y.into())),
		})
	}
}
