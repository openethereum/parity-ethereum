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

use ethsync::{EthSync, ManageNetwork};
use ethcore::miner::{Miner, ExternalMiner};
use ethcore::client::Client;
use util::RotatingLogger;
use ethcore::account_provider::AccountProvider;
use util::network_settings::NetworkSettings;

pub use ethcore_rpc::ConfirmationsQueue;

use ethcore_rpc::Extendable;

#[derive(Debug, PartialEq)]
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
			"ethcore" => Ok(Ethcore),
			"ethcore_set" => Ok(EthcoreSet),
			"traces" => Ok(Traces),
			"rpc" => Ok(Rpc),
			api => Err(format!("Unknown api: {}", api))
		}
	}
}

#[derive(Debug, PartialEq)]
pub enum ApiSet {
	SafeContext,
	UnsafeContext,
	List(Vec<Api>),
}

impl Default for ApiSet {
	fn default() -> Self {
		ApiSet::UnsafeContext
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
	pub net_service: Arc<ManageNetwork>,
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

impl ApiSet {
	pub fn list_apis(self) -> Vec<Api> {
		match self {
			ApiSet::List(apis) => apis,
			ApiSet::UnsafeContext => {
				vec![Api::Web3, Api::Net, Api::Eth, Api::Personal, Api::Ethcore, Api::Traces, Api::Rpc]
			},
			_ => {
				vec![Api::Web3, Api::Net, Api::Eth, Api::Personal, Api::Signer, Api::Ethcore, Api::Traces, Api::Rpc]
			},
		}
	}
}

pub fn setup_rpc<T: Extendable>(server: T, deps: Arc<Dependencies>, apis: ApiSet) -> T {
	use ethcore_rpc::v1::*;

	let apis = apis.list_apis();
	for api in &apis {
		match *api {
			Api::Web3 => {
				server.add_delegate(Web3Client::new().to_delegate());
			},
			Api::Net => {
				server.add_delegate(NetClient::new(&deps.sync).to_delegate());
			},
			Api::Eth => {
				let client = EthClient::new(
					&deps.client,
					&deps.sync,
					&deps.secret_store,
					&deps.miner,
					&deps.external_miner,
					deps.allow_pending_receipt_query
				);
				server.add_delegate(client.to_delegate());

				let filter_client = EthFilterClient::new(&deps.client, &deps.miner);
				server.add_delegate(filter_client.to_delegate());

				if deps.signer_port.is_some() {
					server.add_delegate(EthSigningQueueClient::new(&deps.signer_queue, &deps.client, &deps.miner, &deps.secret_store).to_delegate());
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
				let queue = deps.signer_port.map(|_| deps.signer_queue.clone());
				server.add_delegate(EthcoreClient::new(&deps.client, &deps.miner, deps.logger.clone(), deps.settings.clone(), queue).to_delegate())
			},
			Api::EthcoreSet => {
				server.add_delegate(EthcoreSetClient::new(&deps.client, &deps.miner, &deps.net_service).to_delegate())
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
		assert_eq!(Api::Ethcore, "ethcore".parse().unwrap());
		assert_eq!(Api::EthcoreSet, "ethcore_set".parse().unwrap());
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
		assert_eq!(ApiSet::List(vec![Api::Web3, Api::Eth]), "web3,eth".parse().unwrap());
	}

	#[test]
	fn test_api_set_unsafe_context() {
		let mut web3 = 0;
		let mut net = 0;
		let mut eth = 0;
		let mut personal = 0;
		let mut signer = 0;
		let mut ethcore = 0;
		let mut ethcore_set = 0;
		let mut traces = 0;
		let mut rpc = 0;

		for i in &ApiSet::UnsafeContext.list_apis() {
			match *i {
				Api::Web3 => web3 += 1,
				Api::Net => net += 1,
				Api::Eth => eth += 1,
				Api::Personal => personal += 1,
				Api::Signer => signer += 1,
				Api::Ethcore => ethcore += 1,
				Api::EthcoreSet => ethcore_set += 1,
				Api::Traces => traces += 1,
				Api::Rpc => rpc += 1,
			}
		}

		assert_eq!(web3, 1);
		assert_eq!(net, 1);
		assert_eq!(eth, 1);
		assert_eq!(personal, 1);
		assert_eq!(signer, 0);
		assert_eq!(ethcore, 1);
		assert_eq!(ethcore_set, 0);
		assert_eq!(traces, 1);
		assert_eq!(rpc, 1);
	}

	#[test]
	fn test_api_set_safe_context() {
		let mut web3 = 0;
		let mut net = 0;
		let mut eth = 0;
		let mut personal = 0;
		let mut signer = 0;
		let mut ethcore = 0;
		let mut ethcore_set = 0;
		let mut traces = 0;
		let mut rpc = 0;

		for i in &ApiSet::SafeContext.list_apis() {
			match *i {
				Api::Web3 => web3 += 1,
				Api::Net => net += 1,
				Api::Eth => eth += 1,
				Api::Personal => personal += 1,
				Api::Signer => signer += 1,
				Api::Ethcore => ethcore += 1,
				Api::EthcoreSet => ethcore_set += 1,
				Api::Traces => traces += 1,
				Api::Rpc => rpc += 1,
			}
		}

		assert_eq!(web3, 1);
		assert_eq!(net, 1);
		assert_eq!(eth, 1);
		assert_eq!(personal, 1);
		assert_eq!(signer, 1);
		assert_eq!(ethcore, 1);
		assert_eq!(ethcore_set, 0);
		assert_eq!(traces, 1);
		assert_eq!(rpc, 1);
	}
}
