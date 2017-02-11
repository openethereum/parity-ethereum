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

use std::cmp::PartialEq;
use std::collections::BTreeMap;
use std::collections::HashSet;
use std::str::FromStr;
use std::sync::Arc;

pub use ethcore_rpc::SignerService;

use ethcore::account_provider::AccountProvider;
use ethcore::client::Client;
use ethcore::miner::{Miner, ExternalMiner};
use ethcore::snapshot::SnapshotService;
use ethcore_rpc::{Metadata, NetworkSettings};
use ethcore_rpc::informant::{Middleware, RpcStats, ClientNotifier};
use ethcore_rpc::dispatch::FullDispatcher;
use ethsync::{ManageNetwork, SyncProvider};
use hash_fetch::fetch::Client as FetchClient;
use jsonrpc_core::{MetaIoHandler};
use updater::Updater;
use util::RotatingLogger;

#[derive(Debug, PartialEq, Clone, Eq, Hash)]
pub enum Api {
	/// Web3 (Safe)
	Web3,
	/// Net (Safe)
	Net,
	/// Eth (Safe)
	Eth,
	/// Geth-compatible "personal" API (DEPRECATED; only used in `--geth` mode.)
	Personal,
	/// Signer - Confirm transactions in Signer (UNSAFE: Passwords, List of transactions)
	Signer,
	/// Parity - Custom extensions (Safe)
	Parity,
	/// Parity Accounts extensions (UNSAFE: Passwords, Side Effects (new account))
	ParityAccounts,
	/// Parity - Set methods (UNSAFE: Side Effects affecting node operation)
	ParitySet,
	/// Traces (Safe)
	Traces,
	/// Rpc (Safe)
	Rpc,
}

impl FromStr for Api {
	type Err = String;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		use self::Api::*;

		match s {
			"web3" => Ok(Web3),
			"net" => Ok(Net),
			"eth" => Ok(Eth),
			"personal" => Ok(Personal),
			"signer" => Ok(Signer),
			"parity" => Ok(Parity),
			"parity_accounts" => Ok(ParityAccounts),
			"parity_set" => Ok(ParitySet),
			"traces" => Ok(Traces),
			"rpc" => Ok(Rpc),
			api => Err(format!("Unknown api: {}", api))
		}
	}
}

#[derive(Debug)]
pub enum ApiSet {
	SafeContext,
	UnsafeContext,
	IpcContext,
	List(HashSet<Api>),
}

impl Default for ApiSet {
	fn default() -> Self {
		ApiSet::UnsafeContext
	}
}

impl PartialEq for ApiSet {
	fn eq(&self, other: &Self) -> bool {
		self.list_apis() == other.list_apis()
	}
}

impl FromStr for ApiSet {
	type Err = String;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		s.split(',')
			.map(Api::from_str)
			.collect::<Result<_, _>>()
			.map(ApiSet::List)
	}
}

pub struct Dependencies {
	pub signer_service: Arc<SignerService>,
	pub client: Arc<Client>,
	pub snapshot: Arc<SnapshotService>,
	pub sync: Arc<SyncProvider>,
	pub net: Arc<ManageNetwork>,
	pub secret_store: Arc<AccountProvider>,
	pub miner: Arc<Miner>,
	pub external_miner: Arc<ExternalMiner>,
	pub logger: Arc<RotatingLogger>,
	pub settings: Arc<NetworkSettings>,
	pub net_service: Arc<ManageNetwork>,
	pub updater: Arc<Updater>,
	pub geth_compatibility: bool,
	pub dapps_interface: Option<String>,
	pub dapps_port: Option<u16>,
	pub fetch: FetchClient,
}

fn to_modules(apis: &[Api]) -> BTreeMap<String, String> {
	let mut modules = BTreeMap::new();
	for api in apis {
		let (name, version) = match *api {
			Api::Web3 => ("web3", "1.0"),
			Api::Net => ("net", "1.0"),
			Api::Eth => ("eth", "1.0"),
			Api::Personal => ("personal", "1.0"),
			Api::Signer => ("signer", "1.0"),
			Api::Parity => ("parity", "1.0"),
			Api::ParityAccounts => ("parity_accounts", "1.0"),
			Api::ParitySet => ("parity_set", "1.0"),
			Api::Traces => ("traces", "1.0"),
			Api::Rpc => ("rpc", "1.0"),
		};
		modules.insert(name.into(), version.into());
	}
	modules
}

impl ApiSet {
	pub fn list_apis(&self) -> HashSet<Api> {
		let mut safe_list = vec![Api::Web3, Api::Net, Api::Eth, Api::Parity, Api::Traces, Api::Rpc]
			.into_iter().collect();
		match *self {
			ApiSet::List(ref apis) => apis.clone(),
			ApiSet::UnsafeContext => safe_list,
			ApiSet::IpcContext => {
				safe_list.insert(Api::ParityAccounts);
				safe_list
			},
			ApiSet::SafeContext => {
				safe_list.insert(Api::ParityAccounts);
				safe_list.insert(Api::ParitySet);
				safe_list.insert(Api::Signer);
				safe_list
			},
		}
	}
}

macro_rules! add_signing_methods {
	($namespace:ident, $handler:expr, $deps:expr) => {
		{
			let handler = &mut $handler;
			let deps = &$deps;
			let dispatcher = FullDispatcher::new(Arc::downgrade(&deps.client), Arc::downgrade(&deps.miner));
			if deps.signer_service.is_enabled() {
				handler.extend_with($namespace::to_delegate(SigningQueueClient::new(&deps.signer_service, dispatcher, &deps.secret_store)))
			} else {
				handler.extend_with($namespace::to_delegate(SigningUnsafeClient::new(&deps.secret_store, dispatcher)))
			}
		}
	}
}

pub fn setup_rpc(stats: Arc<RpcStats>, deps: Arc<Dependencies>, apis: ApiSet) -> MetaIoHandler<Metadata, Middleware> {
	use ethcore_rpc::v1::*;

	let mut handler = MetaIoHandler::with_middleware(Middleware::new(stats, ClientNotifier {
		client: deps.client.clone(),
	}));

	// it's turned into vector, cause ont of the cases requires &[]
	let apis = apis.list_apis().into_iter().collect::<Vec<_>>();
	let dispatcher = FullDispatcher::new(Arc::downgrade(&deps.client), Arc::downgrade(&deps.miner));

	for api in &apis {
		match *api {
			Api::Web3 => {
				handler.extend_with(Web3Client::new().to_delegate());
			},
			Api::Net => {
				handler.extend_with(NetClient::new(&deps.sync).to_delegate());
			},
			Api::Eth => {
				let client = EthClient::new(
					&deps.client,
					&deps.snapshot,
					&deps.sync,
					&deps.secret_store,
					&deps.miner,
					&deps.external_miner,
					EthClientOptions {
						allow_pending_receipt_query: !deps.geth_compatibility,
						send_block_number_in_get_work: !deps.geth_compatibility,
					}
				);
				handler.extend_with(client.to_delegate());

				let filter_client = EthFilterClient::new(&deps.client, &deps.miner);
				handler.extend_with(filter_client.to_delegate());

				add_signing_methods!(EthSigning, handler, deps);
			},
			Api::Personal => {
				handler.extend_with(PersonalClient::new(&deps.secret_store, dispatcher.clone(), deps.geth_compatibility).to_delegate());
			},
			Api::Signer => {
				handler.extend_with(SignerClient::new(&deps.secret_store, dispatcher.clone(), &deps.signer_service).to_delegate());
			},
			Api::Parity => {
				let signer = match deps.signer_service.is_enabled() {
					true => Some(deps.signer_service.clone()),
					false => None,
				};
				handler.extend_with(ParityClient::new(
					&deps.client,
					&deps.miner,
					&deps.sync,
					&deps.updater,
					&deps.net_service,
					&deps.secret_store,
					deps.logger.clone(),
					deps.settings.clone(),
					signer,
					deps.dapps_interface.clone(),
					deps.dapps_port,
				).to_delegate());

				add_signing_methods!(EthSigning, handler, deps);
				add_signing_methods!(ParitySigning, handler, deps);
			},
			Api::ParityAccounts => {
				handler.extend_with(ParityAccountsClient::new(&deps.secret_store).to_delegate());
			},
			Api::ParitySet => {
				handler.extend_with(ParitySetClient::new(
					&deps.client,
					&deps.miner,
					&deps.updater,
					&deps.net_service,
					deps.fetch.clone(),
				).to_delegate())
			},
			Api::Traces => {
				handler.extend_with(TracesClient::new(&deps.client, &deps.miner).to_delegate())
			},
			Api::Rpc => {
				let modules = to_modules(&apis);
				handler.extend_with(RpcClient::new(modules).to_delegate());
			}
		}
	}
	handler
}

#[cfg(test)]
mod test {
	use super::{Api, ApiSet};

	#[test]
	fn test_api_parsing() {
		assert_eq!(Api::Web3, "web3".parse().unwrap());
		assert_eq!(Api::Net, "net".parse().unwrap());
		assert_eq!(Api::Eth, "eth".parse().unwrap());
		assert_eq!(Api::Personal, "personal".parse().unwrap());
		assert_eq!(Api::Signer, "signer".parse().unwrap());
		assert_eq!(Api::Parity, "parity".parse().unwrap());
		assert_eq!(Api::ParityAccounts, "parity_accounts".parse().unwrap());
		assert_eq!(Api::ParitySet, "parity_set".parse().unwrap());
		assert_eq!(Api::Traces, "traces".parse().unwrap());
		assert_eq!(Api::Rpc, "rpc".parse().unwrap());
		assert!("rp".parse::<Api>().is_err());
	}

	#[test]
	fn test_api_set_default() {
		assert_eq!(ApiSet::UnsafeContext, ApiSet::default());
	}

	#[test]
	fn test_api_set_parsing() {
		assert_eq!(ApiSet::List(vec![Api::Web3, Api::Eth].into_iter().collect()), "web3,eth".parse().unwrap());
	}

	#[test]
	fn test_api_set_unsafe_context() {
		let expected = vec![
			// make sure this list contains only SAFE methods
			Api::Web3, Api::Net, Api::Eth, Api::Parity, Api::Traces, Api::Rpc
		].into_iter().collect();
		assert_eq!(ApiSet::UnsafeContext.list_apis(), expected);
	}

	#[test]
	fn test_api_set_ipc_context() {
		let expected = vec![
			// safe
			Api::Web3, Api::Net, Api::Eth, Api::Parity, Api::Traces, Api::Rpc,
			// semi-safe
			Api::ParityAccounts
		].into_iter().collect();
		assert_eq!(ApiSet::IpcContext.list_apis(), expected);
	}

	#[test]
	fn test_api_set_safe_context() {
		let expected = vec![
			// safe
			Api::Web3, Api::Net, Api::Eth, Api::Parity, Api::Traces, Api::Rpc,
			// semi-safe
			Api::ParityAccounts,
			// Unsafe
			Api::ParitySet, Api::Signer,
		].into_iter().collect();
		assert_eq!(ApiSet::SafeContext.list_apis(), expected);
	}
}
