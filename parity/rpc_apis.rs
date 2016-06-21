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

use std::collections::BTreeMap;
use std::str::FromStr;
use std::sync::Arc;

use die::*;
use ethsync::EthSync;
use ethcore::miner::{Miner, ExternalMiner};
use ethcore::client::Client;
use util::RotatingLogger;
use ethcore::account_provider::AccountProvider;
use util::network_settings::NetworkSettings;
use util::network::NetworkService;

#[cfg(feature="rpc")]
pub use ethcore_rpc::ConfirmationsQueue;
#[cfg(not(feature="rpc"))]
#[derive(Default)]
pub struct ConfirmationsQueue;

#[cfg(feature="rpc")]
use ethcore_rpc::Extendable;

pub enum Api {
	Web3,
	Net,
	Eth,
	Personal,
	Signer,
	Ethcore,
	EthcoreSet,
	Traces,
	Rpc,
}

pub enum ApiError {
	UnknownApi(String)
}

pub enum ApiSet {
	SafeContext,
	UnsafeContext,
	List(Vec<Api>),
}

impl FromStr for Api {
	type Err = ApiError;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		use self::Api::*;

		match s {
			"web3" => Ok(Web3),
			"net" => Ok(Net),
			"eth" => Ok(Eth),
			"personal" => Ok(Personal),
			"signer" => Ok(Signer),
			"ethcore" => Ok(Ethcore),
			"ethcore_set" => Ok(EthcoreSet),
			"traces" => Ok(Traces),
			"rpc" => Ok(Rpc),
			e => Err(ApiError::UnknownApi(e.into())),
		}
	}
}

pub struct Dependencies {
	pub signer_port: Option<u16>,
	pub signer_queue: Arc<ConfirmationsQueue>,
	pub client: Arc<Client>,
	pub sync: Arc<EthSync>,
	pub secret_store: Arc<AccountProvider>,
	pub miner: Arc<Miner>,
	pub external_miner: Arc<ExternalMiner>,
	pub logger: Arc<RotatingLogger>,
	pub settings: Arc<NetworkSettings>,
	pub allow_pending_receipt_query: bool,
	pub net_service: Arc<NetworkService<::ethcore::service::SyncMessage>>,
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
			Api::Ethcore => ("ethcore", "1.0"),
			Api::EthcoreSet => ("ethcore_set", "1.0"),
			Api::Traces => ("traces", "1.0"),
			Api::Rpc => ("rpc", "1.0"),
		};
		modules.insert(name.into(), version.into());
	}
	modules
}

pub fn from_str(apis: Vec<&str>) -> Vec<Api> {
	apis.into_iter()
		.map(Api::from_str)
		.collect::<Result<Vec<Api>, ApiError>>()
		.unwrap_or_else(|e| match e {
			ApiError::UnknownApi(s) => die!("Unknown RPC API specified: {}", s),
		})
}

fn list_apis(apis: ApiSet) -> Vec<Api> {
	match apis {
		ApiSet::List(apis) => apis,
		ApiSet::UnsafeContext => {
			vec![Api::Web3, Api::Net, Api::Eth, Api::Personal, Api::Ethcore, Api::Traces, Api::Rpc]
		},
		_ => {
			vec![Api::Web3, Api::Net, Api::Eth, Api::Personal, Api::Signer, Api::Ethcore, Api::Traces, Api::Rpc]
		},
	}
}

pub fn setup_rpc<T: Extendable>(server: T, deps: Arc<Dependencies>, apis: ApiSet) -> T {
	use ethcore_rpc::v1::*;

	let apis = list_apis(apis);
	for api in &apis {
		match *api {
			Api::Web3 => {
				server.add_delegate(Web3Client::new().to_delegate());
			},
			Api::Net => {
				server.add_delegate(NetClient::new(&deps.sync).to_delegate());
			},
			Api::Eth => {
				server.add_delegate(EthClient::new(&deps.client, &deps.sync, &deps.secret_store, &deps.miner, &deps.external_miner, deps.allow_pending_receipt_query).to_delegate());
				server.add_delegate(EthFilterClient::new(&deps.client, &deps.miner).to_delegate());

				if deps.signer_port.is_some() {
					server.add_delegate(EthSigningQueueClient::new(&deps.signer_queue, &deps.miner).to_delegate());
				} else {
					server.add_delegate(EthSigningUnsafeClient::new(&deps.client, &deps.secret_store, &deps.miner).to_delegate());
				}
			},
			Api::Personal => {
				server.add_delegate(PersonalClient::new(&deps.secret_store, &deps.client, &deps.miner, deps.signer_port).to_delegate());
			},
			Api::Signer => {
				server.add_delegate(SignerClient::new(&deps.secret_store, &deps.client, &deps.miner, &deps.signer_queue).to_delegate());
			},
			Api::Ethcore => {
				server.add_delegate(EthcoreClient::new(&deps.client, &deps.miner, deps.logger.clone(), deps.settings.clone()).to_delegate())
			},
			Api::EthcoreSet => {
				server.add_delegate(EthcoreSetClient::new(&deps.miner, &deps.net_service).to_delegate())
			},
			Api::Traces => {
				server.add_delegate(TracesClient::new(&deps.client, &deps.miner).to_delegate())
			},
			Api::Rpc => {
				let modules = to_modules(&apis);
				server.add_delegate(RpcClient::new(modules).to_delegate());
			}
		}
	}
	server
}
