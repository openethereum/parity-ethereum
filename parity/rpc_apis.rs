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
use std::sync::{Arc, Weak};

pub use parity_rpc::signer::SignerService;
pub use parity_rpc::dapps::{DappsService, LocalDapp};

use ethcore::account_provider::AccountProvider;
use ethcore::client::Client;
use ethcore::miner::{Miner, ExternalMiner};
use ethcore::snapshot::SnapshotService;
use ethcore_logger::RotatingLogger;
use ethsync::{ManageNetwork, SyncProvider, LightSync};
use hash_fetch::fetch::Client as FetchClient;
use jsonrpc_core::{self as core, MetaIoHandler};
use light::{TransactionQueue as LightTransactionQueue, Cache as LightDataCache};
use node_health::NodeHealth;
use parity_reactor;
use parity_rpc::dispatch::{FullDispatcher, LightDispatcher};
use parity_rpc::informant::{ActivityNotifier, ClientNotifier};
use parity_rpc::{Metadata, NetworkSettings};
use updater::Updater;
use parking_lot::{Mutex, RwLock};

#[derive(Debug, PartialEq, Clone, Eq, Hash)]
pub enum Api {
	/// Web3 (Safe)
	Web3,
	/// Net (Safe)
	Net,
	/// Eth (Safe)
	Eth,
	/// Eth Pub-Sub (Safe)
	EthPubSub,
	/// Geth-compatible "personal" API (DEPRECATED; only used in `--geth` mode.)
	Personal,
	/// Signer - Confirm transactions in Signer (UNSAFE: Passwords, List of transactions)
	Signer,
	/// Parity - Custom extensions (Safe)
	Parity,
	/// Parity PubSub - Generic Publish-Subscriber (Safety depends on other APIs exposed).
	ParityPubSub,
	/// Parity Accounts extensions (UNSAFE: Passwords, Side Effects (new account))
	ParityAccounts,
	/// Parity - Set methods (UNSAFE: Side Effects affecting node operation)
	ParitySet,
	/// Traces (Safe)
	Traces,
	/// Rpc (Safe)
	Rpc,
	/// SecretStore (Safe)
	SecretStore,
	/// Whisper (Safe)
	// TODO: _if_ someone guesses someone else's key or filter IDs they can remove
	// BUT these are all ephemeral so it seems fine.
	Whisper,
	/// Whisper Pub-Sub (Safe but same concerns as above).
	WhisperPubSub,
}

impl FromStr for Api {
	type Err = String;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		use self::Api::*;

		match s {
			"web3" => Ok(Web3),
			"net" => Ok(Net),
			"eth" => Ok(Eth),
			"pubsub" => Ok(EthPubSub),
			"personal" => Ok(Personal),
			"signer" => Ok(Signer),
			"parity" => Ok(Parity),
			"parity_pubsub" => Ok(ParityPubSub),
			"parity_accounts" => Ok(ParityAccounts),
			"parity_set" => Ok(ParitySet),
			"traces" => Ok(Traces),
			"rpc" => Ok(Rpc),
			"secretstore" => Ok(SecretStore),
			"shh" => Ok(Whisper),
			"shh_pubsub" => Ok(WhisperPubSub),
			api => Err(format!("Unknown api: {}", api))
		}
	}
}

#[derive(Debug, Clone)]
pub enum ApiSet {
	// Safe context (like token-protected WS interface)
	SafeContext,
	// Unsafe context (like jsonrpc over http)
	UnsafeContext,
	// Public context (like public jsonrpc over http)
	PublicContext,
	// All possible APIs
	All,
	// Local "unsafe" context and accounts access
	IpcContext,
	// APIs for Parity Generic Pub-Sub
	PubSub,
	// Fixed list of APis
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
		let mut apis = HashSet::new();

		for api in s.split(',') {
			match api {
				"all" => {
					apis.extend(ApiSet::All.list_apis());
				},
				"safe" => {
					// Safe APIs are those that are safe even in UnsafeContext.
					apis.extend(ApiSet::UnsafeContext.list_apis());
				},
				// Remove the API
				api if api.starts_with("-") => {
					let api = api[1..].parse()?;
					apis.remove(&api);
				},
				api => {
					let api = api.parse()?;
					apis.insert(api);
				},
			}
		}

		Ok(ApiSet::List(apis))
	}
}

fn to_modules(apis: &HashSet<Api>) -> BTreeMap<String, String> {
	let mut modules = BTreeMap::new();
	for api in apis {
		let (name, version) = match *api {
			Api::Web3 => ("web3", "1.0"),
			Api::Net => ("net", "1.0"),
			Api::Eth => ("eth", "1.0"),
			Api::EthPubSub => ("pubsub", "1.0"),
			Api::Personal => ("personal", "1.0"),
			Api::Signer => ("signer", "1.0"),
			Api::Parity => ("parity", "1.0"),
			Api::ParityAccounts => ("parity_accounts", "1.0"),
			Api::ParityPubSub => ("parity_pubsub", "1.0"),
			Api::ParitySet => ("parity_set", "1.0"),
			Api::Traces => ("traces", "1.0"),
			Api::Rpc => ("rpc", "1.0"),
			Api::SecretStore => ("secretstore", "1.0"),
			Api::Whisper => ("shh", "1.0"),
			Api::WhisperPubSub => ("shh_pubsub", "1.0"),
		};
		modules.insert(name.into(), version.into());
	}
	modules
}

/// RPC dependencies can be used to initialize RPC endpoints from APIs.
pub trait Dependencies {
	type Notifier: ActivityNotifier;

	/// Create the activity notifier.
	fn activity_notifier(&self) -> Self::Notifier;

	/// Extend the given I/O handler with endpoints for each API.
	fn extend_with_set<S>(
		&self,
		handler: &mut MetaIoHandler<Metadata, S>,
		apis: &HashSet<Api>,
	) where S: core::Middleware<Metadata>;
}

/// RPC dependencies for a full node.
pub struct FullDependencies {
	pub signer_service: Arc<SignerService>,
	pub client: Arc<Client>,
	pub snapshot: Arc<SnapshotService>,
	pub sync: Arc<SyncProvider>,
	pub net: Arc<ManageNetwork>,
	pub secret_store: Option<Arc<AccountProvider>>,
	pub miner: Arc<Miner>,
	pub external_miner: Arc<ExternalMiner>,
	pub logger: Arc<RotatingLogger>,
	pub settings: Arc<NetworkSettings>,
	pub net_service: Arc<ManageNetwork>,
	pub updater: Arc<Updater>,
	pub health: NodeHealth,
	pub geth_compatibility: bool,
	pub dapps_service: Option<Arc<DappsService>>,
	pub dapps_address: Option<(String, u16)>,
	pub ws_address: Option<(String, u16)>,
	pub fetch: FetchClient,
	pub remote: parity_reactor::Remote,
	pub whisper_rpc: Option<::whisper::RpcFactory>,
}

impl FullDependencies {
	fn extend_api<S>(
		&self,
		handler: &mut MetaIoHandler<Metadata, S>,
		apis: &HashSet<Api>,
		for_generic_pubsub: bool,
	) where S: core::Middleware<Metadata> {
		use parity_rpc::v1::*;

		macro_rules! add_signing_methods {
			($namespace:ident, $handler:expr, $deps:expr) => {
				{
					let deps = &$deps;
					let dispatcher = FullDispatcher::new(deps.client.clone(), deps.miner.clone());
					if deps.signer_service.is_enabled() {
						$handler.extend_with($namespace::to_delegate(SigningQueueClient::new(&deps.signer_service, dispatcher, &deps.secret_store)))
					} else {
						$handler.extend_with($namespace::to_delegate(SigningUnsafeClient::new(&deps.secret_store, dispatcher)))
					}
				}
			}
		}

		let dispatcher = FullDispatcher::new(self.client.clone(), self.miner.clone());
		for api in apis {
			match *api {
				Api::Web3 => {
					handler.extend_with(Web3Client::new().to_delegate());
				},
				Api::Net => {
					handler.extend_with(NetClient::new(&self.sync).to_delegate());
				},
				Api::Eth => {
					let client = EthClient::new(
						&self.client,
						&self.snapshot,
						&self.sync,
						&self.secret_store,
						&self.miner,
						&self.external_miner,
						EthClientOptions {
							pending_nonce_from_queue: self.geth_compatibility,
							allow_pending_receipt_query: !self.geth_compatibility,
							send_block_number_in_get_work: !self.geth_compatibility,
						}
					);
					handler.extend_with(client.to_delegate());

					if !for_generic_pubsub {
						let filter_client = EthFilterClient::new(self.client.clone(), self.miner.clone());
						handler.extend_with(filter_client.to_delegate());

						add_signing_methods!(EthSigning, handler, self);
					}
				},
				Api::EthPubSub => {
					if !for_generic_pubsub {
						let client = EthPubSubClient::new(self.client.clone(), self.remote.clone());
						self.client.add_notify(client.handler());
						handler.extend_with(client.to_delegate());
					}
				},
				Api::Personal => {
					handler.extend_with(PersonalClient::new(&self.secret_store, dispatcher.clone(), self.geth_compatibility).to_delegate());
				},
				Api::Signer => {
					handler.extend_with(SignerClient::new(&self.secret_store, dispatcher.clone(), &self.signer_service, self.remote.clone()).to_delegate());
				},
				Api::Parity => {
					let signer = match self.signer_service.is_enabled() {
						true => Some(self.signer_service.clone()),
						false => None,
					};
					handler.extend_with(ParityClient::new(
						self.client.clone(),
						self.miner.clone(),
						self.sync.clone(),
						self.updater.clone(),
						self.net_service.clone(),
						self.health.clone(),
						self.secret_store.clone(),
						self.logger.clone(),
						self.settings.clone(),
						signer,
						self.dapps_address.clone(),
						self.ws_address.clone(),
					).to_delegate());

					if !for_generic_pubsub {
						add_signing_methods!(ParitySigning, handler, self);
					}
				},
				Api::ParityPubSub => {
					if !for_generic_pubsub {
						let mut rpc = MetaIoHandler::default();
						let apis = ApiSet::List(apis.clone()).retain(ApiSet::PubSub).list_apis();
						self.extend_api(&mut rpc, &apis, true);
						handler.extend_with(PubSubClient::new(rpc, self.remote.clone()).to_delegate());
					}
				},
				Api::ParityAccounts => {
					handler.extend_with(ParityAccountsClient::new(&self.secret_store).to_delegate());
				},
				Api::ParitySet => {
					handler.extend_with(ParitySetClient::new(
						&self.client,
						&self.miner,
						&self.updater,
						&self.net_service,
						self.dapps_service.clone(),
						self.fetch.clone(),
					).to_delegate())
				},
				Api::Traces => {
					handler.extend_with(TracesClient::new(&self.client, &self.miner).to_delegate())
				},
				Api::Rpc => {
					let modules = to_modules(&apis);
					handler.extend_with(RpcClient::new(modules).to_delegate());
				},
				Api::SecretStore => {
					handler.extend_with(SecretStoreClient::new(&self.secret_store).to_delegate());
				},
				Api::Whisper => {
					if let Some(ref whisper_rpc) = self.whisper_rpc {
						let whisper = whisper_rpc.make_handler();
						handler.extend_with(::parity_whisper::rpc::Whisper::to_delegate(whisper));
					}
				}
				Api::WhisperPubSub => {
					if !for_generic_pubsub {
						if let Some(ref whisper_rpc) = self.whisper_rpc {
							let whisper = whisper_rpc.make_handler();
							handler.extend_with(
								::parity_whisper::rpc::WhisperPubSub::to_delegate(whisper)
							);
						}
					}
				}
			}
		}
	}
}

impl Dependencies for FullDependencies {
	type Notifier = ClientNotifier;

	fn activity_notifier(&self) -> ClientNotifier {
		ClientNotifier {
			client: self.client.clone(),
		}
	}

	fn extend_with_set<S>(
		&self,
		handler: &mut MetaIoHandler<Metadata, S>,
		apis: &HashSet<Api>,
	) where S: core::Middleware<Metadata> {
		self.extend_api(handler, apis, false)
	}
}

/// Light client notifier. Doesn't do anything yet, but might in the future.
pub struct LightClientNotifier;

impl ActivityNotifier for LightClientNotifier {
	fn active(&self) {}
}

/// RPC dependencies for a light client.
pub struct LightDependencies {
	pub signer_service: Arc<SignerService>,
	pub client: Arc<::light::client::Client>,
	pub sync: Arc<LightSync>,
	pub net: Arc<ManageNetwork>,
	pub secret_store: Arc<AccountProvider>,
	pub logger: Arc<RotatingLogger>,
	pub settings: Arc<NetworkSettings>,
	pub health: NodeHealth,
	pub on_demand: Arc<::light::on_demand::OnDemand>,
	pub cache: Arc<Mutex<LightDataCache>>,
	pub transaction_queue: Arc<RwLock<LightTransactionQueue>>,
	pub dapps_service: Option<Arc<DappsService>>,
	pub dapps_address: Option<(String, u16)>,
	pub ws_address: Option<(String, u16)>,
	pub fetch: FetchClient,
	pub geth_compatibility: bool,
	pub remote: parity_reactor::Remote,
	pub whisper_rpc: Option<::whisper::RpcFactory>,
}

impl LightDependencies {
	fn extend_api<T: core::Middleware<Metadata>>(
		&self,
		handler: &mut MetaIoHandler<Metadata, T>,
		apis: &HashSet<Api>,
		for_generic_pubsub: bool,
	) {
		use parity_rpc::v1::*;

		let dispatcher = LightDispatcher::new(
			self.sync.clone(),
			self.client.clone(),
			self.on_demand.clone(),
			self.cache.clone(),
			self.transaction_queue.clone(),
		);

		macro_rules! add_signing_methods {
			($namespace:ident, $handler:expr, $deps:expr) => {
				{
					let deps = &$deps;
					let dispatcher = dispatcher.clone();
					let secret_store = Some(deps.secret_store.clone());
					if deps.signer_service.is_enabled() {
						$handler.extend_with($namespace::to_delegate(
							SigningQueueClient::new(&deps.signer_service, dispatcher, &secret_store)
						))
					} else {
						$handler.extend_with(
							$namespace::to_delegate(SigningUnsafeClient::new(&secret_store, dispatcher))
						)
					}
				}
			}
		}

		for api in apis {
			match *api {
				Api::Web3 => {
					handler.extend_with(Web3Client::new().to_delegate());
				},
				Api::Net => {
					handler.extend_with(light::NetClient::new(self.sync.clone()).to_delegate());
				},
				Api::Eth => {
					let client = light::EthClient::new(
						self.sync.clone(),
						self.client.clone(),
						self.on_demand.clone(),
						self.transaction_queue.clone(),
						self.secret_store.clone(),
						self.cache.clone(),
					);
					handler.extend_with(Eth::to_delegate(client.clone()));

					if !for_generic_pubsub {
						handler.extend_with(EthFilter::to_delegate(client));
						add_signing_methods!(EthSigning, handler, self);
					}
				},
				Api::EthPubSub => {
					let client = EthPubSubClient::light(
						self.client.clone(),
						self.on_demand.clone(),
						self.sync.clone(),
						self.cache.clone(),
						self.remote.clone(),
					);
					self.client.add_listener(
						Arc::downgrade(&client.handler()) as Weak<::light::client::LightChainNotify>
					);
					handler.extend_with(EthPubSub::to_delegate(client));
				},
				Api::Personal => {
					let secret_store = Some(self.secret_store.clone());
					handler.extend_with(PersonalClient::new(&secret_store, dispatcher.clone(), self.geth_compatibility).to_delegate());
				},
				Api::Signer => {
					let secret_store = Some(self.secret_store.clone());
					handler.extend_with(SignerClient::new(&secret_store, dispatcher.clone(), &self.signer_service, self.remote.clone()).to_delegate());
				},
				Api::Parity => {
					let signer = match self.signer_service.is_enabled() {
						true => Some(self.signer_service.clone()),
						false => None,
					};
					handler.extend_with(light::ParityClient::new(
						self.client.clone(),
						Arc::new(dispatcher.clone()),
						self.secret_store.clone(),
						self.logger.clone(),
						self.settings.clone(),
						self.health.clone(),
						signer,
						self.dapps_address.clone(),
						self.ws_address.clone(),
					).to_delegate());

					if !for_generic_pubsub {
						add_signing_methods!(ParitySigning, handler, self);
					}
				},
				Api::ParityPubSub => {
					if !for_generic_pubsub {
						let mut rpc = MetaIoHandler::default();
						let apis = ApiSet::List(apis.clone()).retain(ApiSet::PubSub).list_apis();
						self.extend_api(&mut rpc, &apis, true);
						handler.extend_with(PubSubClient::new(rpc, self.remote.clone()).to_delegate());
					}
				},
				Api::ParityAccounts => {
					let secret_store = Some(self.secret_store.clone());
					handler.extend_with(ParityAccountsClient::new(&secret_store).to_delegate());
				},
				Api::ParitySet => {
					handler.extend_with(light::ParitySetClient::new(
						self.sync.clone(),
						self.dapps_service.clone(),
						self.fetch.clone(),
					).to_delegate())
				},
				Api::Traces => {
					handler.extend_with(light::TracesClient.to_delegate())
				},
				Api::Rpc => {
					let modules = to_modules(&apis);
					handler.extend_with(RpcClient::new(modules).to_delegate());
				},
				Api::SecretStore => {
					let secret_store = Some(self.secret_store.clone());
					handler.extend_with(SecretStoreClient::new(&secret_store).to_delegate());
				},
				Api::Whisper => {
					if let Some(ref whisper_rpc) = self.whisper_rpc {
						let whisper = whisper_rpc.make_handler();
						handler.extend_with(::parity_whisper::rpc::Whisper::to_delegate(whisper));
					}
				}
				Api::WhisperPubSub => {
					if let Some(ref whisper_rpc) = self.whisper_rpc {
						let whisper = whisper_rpc.make_handler();
						handler.extend_with(::parity_whisper::rpc::WhisperPubSub::to_delegate(whisper));
					}
				}
			}
		}
	}
}

impl Dependencies for LightDependencies {
	type Notifier = LightClientNotifier;

	fn activity_notifier(&self) -> Self::Notifier { LightClientNotifier }

	fn extend_with_set<S>(
		&self,
		handler: &mut MetaIoHandler<Metadata, S>,
		apis: &HashSet<Api>,
	) where S: core::Middleware<Metadata> {
		self.extend_api(handler, apis, false)
	}
}

impl ApiSet {
	/// Retains only APIs in given set.
	pub fn retain(self, set: Self) -> Self {
		ApiSet::List(&self.list_apis() & &set.list_apis())
	}

	pub fn list_apis(&self) -> HashSet<Api> {
		let mut public_list = [
			Api::Web3,
			Api::Net,
			Api::Eth,
			Api::EthPubSub,
			Api::Parity,
			Api::Rpc,
			Api::SecretStore,
			Api::Whisper,
			Api::WhisperPubSub,
		].into_iter().cloned().collect();

		match *self {
			ApiSet::List(ref apis) => apis.clone(),
			ApiSet::PublicContext => public_list,
			ApiSet::UnsafeContext => {
				public_list.insert(Api::Traces);
				public_list.insert(Api::ParityPubSub);
				public_list
			},
			ApiSet::IpcContext => {
				public_list.insert(Api::Traces);
				public_list.insert(Api::ParityPubSub);
				public_list.insert(Api::ParityAccounts);
				public_list
			},
			ApiSet::SafeContext => {
				public_list.insert(Api::Traces);
				public_list.insert(Api::ParityPubSub);
				public_list.insert(Api::ParityAccounts);
				public_list.insert(Api::ParitySet);
				public_list.insert(Api::Signer);
				public_list
			},
			ApiSet::All => {
				public_list.insert(Api::Traces);
				public_list.insert(Api::ParityPubSub);
				public_list.insert(Api::ParityAccounts);
				public_list.insert(Api::ParitySet);
				public_list.insert(Api::Signer);
				public_list.insert(Api::Personal);
				public_list
			},
			ApiSet::PubSub => [
				Api::Eth,
				Api::Parity,
				Api::ParityAccounts,
				Api::ParitySet,
				Api::Traces,
			].into_iter().cloned().collect()
		}
	}
}

#[cfg(test)]
mod test {
	use super::{Api, ApiSet};

	#[test]
	fn test_api_parsing() {
		assert_eq!(Api::Web3, "web3".parse().unwrap());
		assert_eq!(Api::Net, "net".parse().unwrap());
		assert_eq!(Api::Eth, "eth".parse().unwrap());
		assert_eq!(Api::EthPubSub, "pubsub".parse().unwrap());
		assert_eq!(Api::Personal, "personal".parse().unwrap());
		assert_eq!(Api::Signer, "signer".parse().unwrap());
		assert_eq!(Api::Parity, "parity".parse().unwrap());
		assert_eq!(Api::ParityAccounts, "parity_accounts".parse().unwrap());
		assert_eq!(Api::ParitySet, "parity_set".parse().unwrap());
		assert_eq!(Api::Traces, "traces".parse().unwrap());
		assert_eq!(Api::Rpc, "rpc".parse().unwrap());
		assert_eq!(Api::SecretStore, "secretstore".parse().unwrap());
		assert_eq!(Api::Whisper, "shh".parse().unwrap());
		assert_eq!(Api::WhisperPubSub, "shh_pubsub".parse().unwrap());
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
			Api::Web3, Api::Net, Api::Eth, Api::EthPubSub, Api::Parity, Api::ParityPubSub, Api::Traces, Api::Rpc, Api::SecretStore, Api::Whisper, Api::WhisperPubSub,
		].into_iter().collect();
		assert_eq!(ApiSet::UnsafeContext.list_apis(), expected);
	}

	#[test]
	fn test_api_set_ipc_context() {
		let expected = vec![
			// safe
			Api::Web3, Api::Net, Api::Eth, Api::EthPubSub, Api::Parity, Api::ParityPubSub, Api::Traces, Api::Rpc, Api::SecretStore, Api::Whisper, Api::WhisperPubSub,
			// semi-safe
			Api::ParityAccounts
		].into_iter().collect();
		assert_eq!(ApiSet::IpcContext.list_apis(), expected);
	}

	#[test]
	fn test_api_set_safe_context() {
		let expected = vec![
			// safe
			Api::Web3, Api::Net, Api::Eth, Api::EthPubSub, Api::Parity, Api::ParityPubSub, Api::Traces, Api::Rpc, Api::SecretStore, Api::Whisper, Api::WhisperPubSub,
			// semi-safe
			Api::ParityAccounts,
			// Unsafe
			Api::ParitySet, Api::Signer,
		].into_iter().collect();
		assert_eq!(ApiSet::SafeContext.list_apis(), expected);
	}

	#[test]
	fn test_all_apis() {
		assert_eq!("all".parse::<ApiSet>().unwrap(), ApiSet::List(vec![
			Api::Web3, Api::Net, Api::Eth, Api::EthPubSub, Api::Parity, Api::ParityPubSub, Api::Traces, Api::Rpc, Api::SecretStore, Api::Whisper, Api::WhisperPubSub,
			Api::ParityAccounts,
			Api::ParitySet, Api::Signer,
			Api::Personal
		].into_iter().collect()));
	}

	#[test]
	fn test_all_without_personal_apis() {
		assert_eq!("personal,all,-personal".parse::<ApiSet>().unwrap(), ApiSet::List(vec![
			Api::Web3, Api::Net, Api::Eth, Api::EthPubSub, Api::Parity, Api::ParityPubSub, Api::Traces, Api::Rpc, Api::SecretStore, Api::Whisper, Api::WhisperPubSub,
			Api::ParityAccounts,
			Api::ParitySet, Api::Signer,
		].into_iter().collect()));
	}

	#[test]
	fn test_safe_parsing() {
		assert_eq!("safe".parse::<ApiSet>().unwrap(), ApiSet::List(vec![
			Api::Web3, Api::Net, Api::Eth, Api::EthPubSub, Api::Parity, Api::ParityPubSub, Api::Traces, Api::Rpc, Api::SecretStore, Api::Whisper, Api::WhisperPubSub,
		].into_iter().collect()));
	}
}
