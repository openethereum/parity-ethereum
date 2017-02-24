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
use std::sync::{Arc, Weak};
use std::str::FromStr;
use std::collections::{BTreeMap, HashSet};
use futures::{future, Future, BoxFuture};

use util::{RotatingLogger, Address};
use util::misc::version_data;

use crypto::ecies;
use ethkey::{Brain, Generator};
use ethstore::random_phrase;
use ethsync::{SyncProvider, ManageNetwork};
use ethcore::miner::MinerService;
use ethcore::client::{MiningBlockChainClient};
use ethcore::mode::Mode;
use ethcore::account_provider::AccountProvider;
use updater::{Service as UpdateService};

use jsonrpc_core::Error;
use jsonrpc_macros::Trailing;
use v1::helpers::{errors, SigningQueue, SignerService, NetworkSettings};
use v1::helpers::dispatch::DEFAULT_MAC;
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

/// Parity implementation.
pub struct ParityClient<C, M, S: ?Sized, U> where
	C: MiningBlockChainClient,
	M: MinerService,
	S: SyncProvider,
	U: UpdateService,
{
	client: Weak<C>,
	miner: Weak<M>,
	sync: Weak<S>,
	updater: Weak<U>,
	net: Weak<ManageNetwork>,
	accounts: Weak<AccountProvider>,
	logger: Arc<RotatingLogger>,
	settings: Arc<NetworkSettings>,
	signer: Option<Arc<SignerService>>,
	dapps_interface: Option<String>,
	dapps_port: Option<u16>,
}

impl<C, M, S: ?Sized, U> ParityClient<C, M, S, U> where
	C: MiningBlockChainClient,
	M: MinerService,
	S: SyncProvider,
	U: UpdateService,
{
	/// Creates new `ParityClient`.
	pub fn new(
		client: &Arc<C>,
		miner: &Arc<M>,
		sync: &Arc<S>,
		updater: &Arc<U>,
		net: &Arc<ManageNetwork>,
		store: &Arc<AccountProvider>,
		logger: Arc<RotatingLogger>,
		settings: Arc<NetworkSettings>,
		signer: Option<Arc<SignerService>>,
		dapps_interface: Option<String>,
		dapps_port: Option<u16>,
	) -> Self {
		ParityClient {
			client: Arc::downgrade(client),
			miner: Arc::downgrade(miner),
			sync: Arc::downgrade(sync),
			updater: Arc::downgrade(updater),
			net: Arc::downgrade(net),
			accounts: Arc::downgrade(store),
			logger: logger,
			settings: settings,
			signer: signer,
			dapps_interface: dapps_interface,
			dapps_port: dapps_port,
		}
	}
}

impl<C, M, S: ?Sized, U> Parity for ParityClient<C, M, S, U> where
	M: MinerService + 'static,
	C: MiningBlockChainClient + 'static,
	S: SyncProvider + 'static,
	U: UpdateService + 'static,
{
	type Metadata = Metadata;

	fn accounts_info(&self, dapp: Trailing<DappId>) -> Result<BTreeMap<H160, AccountInfo>, Error> {
		let dapp = dapp.0;

		let store = take_weak!(self.accounts);
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
		let store = take_weak!(self.accounts);
		let info = store.hardware_accounts_info().map_err(|e| errors::account("Could not fetch account info.", e))?;
		Ok(info
			.into_iter()
			.map(|(a, v)| (H160::from(a), HwAccountInfo { name: v.name, manufacturer: v.meta }))
			.collect()
		)
	}

	fn default_account(&self, meta: Self::Metadata) -> BoxFuture<H160, Error> {
		let dapp_id = meta.dapp_id();
		future::ok(
			take_weakf!(self.accounts)
				.dapp_default_address(dapp_id.into())
				.map(Into::into)
				.ok()
				.unwrap_or_default()
		).boxed()
	}

	fn transactions_limit(&self) -> Result<usize, Error> {
		Ok(take_weak!(self.miner).transactions_limit())
	}

	fn min_gas_price(&self) -> Result<U256, Error> {
		Ok(U256::from(take_weak!(self.miner).minimal_gas_price()))
	}

	fn extra_data(&self) -> Result<Bytes, Error> {
		Ok(Bytes::new(take_weak!(self.miner).extra_data()))
	}

	fn gas_floor_target(&self) -> Result<U256, Error> {
		Ok(U256::from(take_weak!(self.miner).gas_floor_target()))
	}

	fn gas_ceil_target(&self) -> Result<U256, Error> {
		Ok(U256::from(take_weak!(self.miner).gas_ceil_target()))
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
		let sync = take_weak!(self.sync);
		let sync_status = sync.status();
		let net_config = take_weak!(self.net).network_config();
		let peers = sync.peers().into_iter().map(Into::into).collect();

		Ok(Peers {
			active: sync_status.num_active_peers,
			connected: sync_status.num_peers,
			max: sync_status.current_max_peers(net_config.min_peers, net_config.max_peers),
			peers: peers
		})
	}

	fn net_port(&self) -> Result<u16, Error> {
		Ok(self.settings.network_port)
	}

	fn node_name(&self) -> Result<String, Error> {
		Ok(self.settings.name.clone())
	}

	fn registry_address(&self) -> Result<Option<H160>, Error> {
		Ok(
			take_weak!(self.client)
				.additional_params()
				.get("registrar")
				.and_then(|s| Address::from_str(s).ok())
				.map(|s| H160::from(s))
		)
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
		future::done(take_weakf!(self.client)
			.gas_price_corpus(100)
			.histogram(10)
			.ok_or_else(errors::not_enough_data)
			.map(Into::into)
		).boxed()
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

	fn list_accounts(&self, count: u64, after: Option<H160>, block_number: Trailing<BlockNumber>) -> Result<Option<Vec<H160>>, Error> {
		Ok(take_weak!(self.client)
			.list_accounts(block_number.0.into(), after.map(Into::into).as_ref(), count)
			.map(|a| a.into_iter().map(Into::into).collect()))
	}

	fn list_storage_keys(&self, address: H160, count: u64, after: Option<H256>, block_number: Trailing<BlockNumber>) -> Result<Option<Vec<H256>>, Error> {
		Ok(take_weak!(self.client)
			.list_storage(block_number.0.into(), &address.into(), after.map(Into::into).as_ref(), count)
			.map(|a| a.into_iter().map(Into::into).collect()))
	}

	fn encrypt_message(&self, key: H512, phrase: Bytes) -> Result<Bytes, Error> {
		ecies::encrypt(&key.into(), &DEFAULT_MAC, &phrase.0)
			.map_err(errors::encryption_error)
			.map(Into::into)
	}

	fn pending_transactions(&self) -> Result<Vec<Transaction>, Error> {
		Ok(take_weak!(self.miner).pending_transactions().into_iter().map(Into::into).collect::<Vec<_>>())
	}

	fn future_transactions(&self) -> Result<Vec<Transaction>, Error> {
		Ok(take_weak!(self.miner).future_transactions().into_iter().map(Into::into).collect::<Vec<_>>())
	}

	fn pending_transactions_stats(&self) -> Result<BTreeMap<H256, TransactionStats>, Error> {
		let stats = take_weak!(self.sync).transactions_stats();
		Ok(stats.into_iter()
		   .map(|(hash, stats)| (hash.into(), stats.into()))
		   .collect()
		)
	}

	fn local_transactions(&self) -> Result<BTreeMap<H256, LocalTransactionStatus>, Error> {
		let transactions = take_weak!(self.miner).local_transactions();
		Ok(transactions
		   .into_iter()
		   .map(|(hash, status)| (hash.into(), status.into()))
		   .collect()
		)
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
		let address: Address = address.into();
		let miner = take_weakf!(self.miner);
		let client = take_weakf!(self.client);

		future::ok(miner.last_nonce(&address)
			.map(|n| n + 1.into())
			.unwrap_or_else(|| client.latest_nonce(&address))
			.into()
		).boxed()
	}

	fn mode(&self) -> Result<String, Error> {
		Ok(match take_weak!(self.client).mode() {
			Mode::Off => "offline",
			Mode::Dark(..) => "dark",
			Mode::Passive(..) => "passive",
			Mode::Active => "active",
		}.into())
	}

	fn enode(&self) -> Result<String, Error> {
		take_weak!(self.sync).enode().ok_or_else(errors::network_disabled)
	}

	fn consensus_capability(&self) -> Result<ConsensusCapability, Error> {
		let updater = take_weak!(self.updater);
		Ok(updater.capability().into())
	}

	fn version_info(&self) -> Result<VersionInfo, Error> {
		let updater = take_weak!(self.updater);
		Ok(updater.version_info().into())
	}

	fn releases_info(&self) -> Result<Option<OperationsInfo>, Error> {
		let updater = take_weak!(self.updater);
		Ok(updater.info().map(Into::into))
	}

	fn chain_status(&self) -> Result<ChainStatus, Error> {
		let chain_info = take_weak!(self.client).chain_info();

		let gap = chain_info.ancient_block_number.map(|x| U256::from(x + 1))
			.and_then(|first| chain_info.first_block_number.map(|last| (first, U256::from(last))));

		Ok(ChainStatus {
			block_gap: gap.map(|(x, y)| (x.into(), y.into())),
		})
	}
}
