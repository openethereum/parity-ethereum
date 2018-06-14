// Copyright 2015-2018 Parity Technologies (UK) Ltd.
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

use std::sync::Arc;
use bytes::Bytes;
use ethereum_types::Address;
use ethkey::Public;
use listener::service_contract::ServiceContract;
use listener::service_contract_listener::ServiceTask;
use {ServerKeyId};

/// Aggregated on-chain service contract.
pub struct OnChainServiceContractAggregate {
	/// All hosted service contracts.
	contracts: Vec<Arc<ServiceContract>>,
}

impl OnChainServiceContractAggregate {
	/// Create new aggregated service contract listener.
	pub fn new(contracts: Vec<Arc<ServiceContract>>) -> Self {
		debug_assert!(contracts.len() > 1);
		OnChainServiceContractAggregate {
			contracts: contracts,
		}
	}
}

impl ServiceContract for OnChainServiceContractAggregate {
	fn update(&self) -> bool {
		let mut result = false;
		for contract in &self.contracts {
			result = contract.update() || result;
		}
		result
	}

	fn read_logs(&self) -> Box<Iterator<Item=ServiceTask>> {
		self.contracts.iter()
			.fold(Box::new(::std::iter::empty()) as Box<Iterator<Item=ServiceTask>>, |i, c|
				Box::new(i.chain(c.read_logs())))
	}

	fn read_pending_requests(&self) -> Box<Iterator<Item=(bool, ServiceTask)>> {
		self.contracts.iter()
			.fold(Box::new(::std::iter::empty()) as Box<Iterator<Item=(bool, ServiceTask)>>, |i, c|
				Box::new(i.chain(c.read_pending_requests())))
	}

	// in current implementation all publish methods are independent of actual contract adddress
	// (tx is sent to origin) => we do not care which contract to use for publish data in methods below

	fn publish_generated_server_key(&self, origin: &Address, server_key_id: &ServerKeyId, server_key: Public) -> Result<(), String> {
		self.contracts[0].publish_generated_server_key(origin, server_key_id, server_key)
	}

	fn publish_server_key_generation_error(&self, origin: &Address, server_key_id: &ServerKeyId) -> Result<(), String> {
		self.contracts[0].publish_server_key_generation_error(origin, server_key_id)
	}

	fn publish_retrieved_server_key(&self, origin: &Address, server_key_id: &ServerKeyId, server_key: Public, threshold: usize) -> Result<(), String> {
		self.contracts[0].publish_retrieved_server_key(origin, server_key_id, server_key, threshold)
	}

	fn publish_server_key_retrieval_error(&self, origin: &Address, server_key_id: &ServerKeyId) -> Result<(), String> {
		self.contracts[0].publish_server_key_retrieval_error(origin, server_key_id)
	}

	fn publish_stored_document_key(&self, origin: &Address, server_key_id: &ServerKeyId) -> Result<(), String> {
		self.contracts[0].publish_stored_document_key(origin, server_key_id)
	}

	fn publish_document_key_store_error(&self, origin: &Address, server_key_id: &ServerKeyId) -> Result<(), String> {
		self.contracts[0].publish_document_key_store_error(origin, server_key_id)
	}

	fn publish_retrieved_document_key_common(&self, origin: &Address, server_key_id: &ServerKeyId, requester: &Address, common_point: Public, threshold: usize) -> Result<(), String> {
		self.contracts[0].publish_retrieved_document_key_common(origin, server_key_id, requester, common_point, threshold)
	}

	fn publish_retrieved_document_key_personal(&self, origin: &Address, server_key_id: &ServerKeyId, requester: &Address, participants: &[Address], decrypted_secret: Public, shadow: Bytes) -> Result<(), String> {
		self.contracts[0].publish_retrieved_document_key_personal(origin, server_key_id, requester, participants, decrypted_secret, shadow)
	}

	fn publish_document_key_retrieval_error(&self, origin: &Address, server_key_id: &ServerKeyId, requester: &Address) -> Result<(), String> {
		self.contracts[0].publish_document_key_retrieval_error(origin, server_key_id, requester)
	}
}
