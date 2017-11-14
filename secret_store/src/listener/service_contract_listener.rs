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

use std::sync::{Arc, Weak};
use parking_lot::Mutex;	
use ethcore::filter::Filter;
use ethcore::client::{Client, BlockChainClient, BlockId, ChainNotify};
use native_contracts::SecretStoreService;
use ethkey::{Random, Generator, sign};
use bytes::Bytes;
use hash::keccak;
use bigint::hash::H256;
use util::Address;
use {NodeKeyPair, KeyServer};

/// Name of the SecretStore contract in the registry.
const SERVICE_CONTRACT_REGISTRY_NAME: &'static str = "secretstore_service";

/// Key server has been added to the set.
const SERVER_KEY_REQUESTED_EVENT_NAME: &'static [u8] = &*b"ServerKeyRequested(bytes32)";

lazy_static! {
	static ref SERVER_KEY_REQUESTED_EVENT_NAME_HASH: H256 = keccak(SERVER_KEY_REQUESTED_EVENT_NAME);
}

/// SecretStore <-> Authority connector. Duties:
/// 1. Listen for new requests on SecretStore contract
/// 2. Redirects requests for key server
/// 3. Publishes response on SecretStore contract
pub struct ServiceContractListener {
	/// Cached on-chain contract.
	contract: Mutex<CachedContract>,
}

/// Cached on-chain Key Server set contract.
struct CachedContract {
	/// Blockchain client.
	client: Weak<Client>,
	/// Contract.
	contract: SecretStoreService,
	/// Contract address.
	contract_addr: Option<Address>,
	/// Key server reference.
	key_server: Arc<KeyServer>,
	/// This node key pair.
	self_key_pair: Arc<NodeKeyPair>,
}

impl ServiceContractListener {
	pub fn new(client: &Arc<Client>, key_server: Arc<KeyServer>, self_key_pair: Arc<NodeKeyPair>) -> Arc<ServiceContractListener> {
		let contract = Arc::new(ServiceContractListener {
			contract: Mutex::new(CachedContract::new(client, key_server, self_key_pair)),
		});
		client.add_notify(contract.clone());
		contract
	}
}

impl ChainNotify for ServiceContractListener {
	fn new_blocks(&self, _imported: Vec<H256>, _invalid: Vec<H256>, enacted: Vec<H256>, _retracted: Vec<H256>, _sealed: Vec<H256>, _proposed: Vec<Bytes>, _duration: u64) {
		if !enacted.is_empty() {
			self.contract.lock().update(enacted)
		}
	}
}

impl CachedContract {
	pub fn new(client: &Arc<Client>, key_server: Arc<KeyServer>, self_key_pair: Arc<NodeKeyPair>) -> Self {
		CachedContract {
			client: Arc::downgrade(client),
			contract: SecretStoreService::new(Default::default()), // we aren't going to call contract => could use default address
			contract_addr: client.registry_address(SERVICE_CONTRACT_REGISTRY_NAME.to_owned()),
			key_server: key_server,
			self_key_pair: self_key_pair,
		}
	}

	pub fn update(&mut self, enacted: Vec<H256>) {
		if let Some(client) = self.client.upgrade() {
			// update contract address
			self.contract_addr = client.registry_address(SERVICE_CONTRACT_REGISTRY_NAME.to_owned());

			// check for new key requests.
			// NOTE: If contract is changed, or unregistered && there are several enacted blocks
			// in single update call, some requests in old contract can be abandoned (we get contract_address from latest block)
			// && check for requests in this contract for every enacted block.
			// The opposite is also true (we can process requests of contract, before it actually becames a SS contract).
			if let Some(contract_addr) = self.contract_addr.as_ref() {
				// TODO: in case of reorgs we might process requests for free (maybe wait for several confirmations???) && publish keys without request
				// TODO: in case of reorgs we might publish keys to forked branch (re-submit transaction???)
				for block in enacted {
					let request_logs = client.logs(Filter {
						from_block: BlockId::Hash(block.clone()),
						to_block: BlockId::Hash(block),
						address: Some(vec![contract_addr.clone()]),
						topics: vec![
							Some(vec![*SERVER_KEY_REQUESTED_EVENT_NAME_HASH]),
							None,
							None,
							None,
						],
						limit: None,
					});

					// TODO: it actually should queue tasks to separate thread
					// + separate thread at the beginning should read all requests from contract
					// and then start processing logs
					for request in request_logs {
						// TODO: check if we are selected to process this request
						let key_id = request.entry.topics[1];
						let key = Random.generate().unwrap();
						let signature = sign(key.secret(), &key_id).unwrap();
						let server_key = self.key_server.generate_key(&key_id, &signature, 0).unwrap();
println!("=== generated key: {:?}", server_key);
						// publish generated key
						let server_key_hash = keccak(server_key);
						let signed_key = self.self_key_pair.sign(&server_key_hash).unwrap();
						let transaction_data = self.contract.encode_server_key_generated_input(key_id, server_key.to_vec(), signed_key.v(), signed_key.r().into(), signed_key.s().into()).unwrap();
						client.transact_contract(contract_addr.clone(), transaction_data).unwrap();
					}
				}
			}
		}
	}
}
