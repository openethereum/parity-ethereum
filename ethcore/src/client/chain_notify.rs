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

use bytes::Bytes;
use ethereum_types::H256;
use transaction::UnverifiedTransaction;
use blockchain::ImportRoute;
use std::time::Duration;
use std::collections::HashMap;

/// Messages to broadcast via chain
pub enum ChainMessageType {
	/// Consensus message
	Consensus(Vec<u8>),
	/// Message with private transaction
	PrivateTransaction(Vec<u8>),
	/// Message with signed private transaction
	SignedPrivateTransaction(Vec<u8>),
}

/// Route type to indicate whether it is enacted or retracted.
#[derive(Clone)]
pub enum ChainRouteType {
	/// Enacted block
	Enacted,
	/// Retracted block
	Retracted
}

/// A complete chain enacted retracted route.
#[derive(Default, Clone)]
pub struct ChainRoute {
	route: Vec<(H256, ChainRouteType)>,
	enacted: Vec<H256>,
	retracted: Vec<H256>,
}

impl<'a> From<&'a [ImportRoute]> for ChainRoute {
	fn from(import_results: &'a [ImportRoute]) -> ChainRoute {
		ChainRoute::new(import_results.iter().flat_map(|route| {
			route.retracted.iter().map(|h| (*h, ChainRouteType::Retracted))
				.chain(route.enacted.iter().map(|h| (*h, ChainRouteType::Enacted)))
		}).collect())
	}
}

impl ChainRoute {
	/// Create a new ChainRoute based on block hash and route type pairs.
	pub fn new(route: Vec<(H256, ChainRouteType)>) -> Self {
		let (enacted, retracted) = Self::to_enacted_retracted(&route);

		Self { route, enacted, retracted }
	}

	/// Gather all non-duplicate enacted and retracted blocks.
	fn to_enacted_retracted(route: &[(H256, ChainRouteType)]) -> (Vec<H256>, Vec<H256>) {
		fn map_to_vec(map: Vec<(H256, bool)>) -> Vec<H256> {
			map.into_iter().map(|(k, _v)| k).collect()
		}

		// Because we are doing multiple inserts some of the blocks that were enacted in import `k`
		// could be retracted in import `k+1`. This is why to understand if after all inserts
		// the block is enacted or retracted we iterate over all routes and at the end final state
		// will be in the hashmap
		let map = route.iter().fold(HashMap::new(), |mut map, route| {
			match &route.1 {
				&ChainRouteType::Enacted => {
					map.insert(route.0, true);
				},
				&ChainRouteType::Retracted => {
					map.insert(route.0, false);
				},
			}
			map
		});

		// Split to enacted retracted (using hashmap value)
		let (enacted, retracted) = map.into_iter().partition(|&(_k, v)| v);
		// And convert tuples to keys
		(map_to_vec(enacted), map_to_vec(retracted))
	}

	/// Consume route and return the enacted retracted form.
	pub fn into_enacted_retracted(self) -> (Vec<H256>, Vec<H256>) {
		(self.enacted, self.retracted)
	}

	/// All non-duplicate enacted blocks.
	pub fn enacted(&self) -> &[H256] {
		&self.enacted
	}

	/// All non-duplicate retracted blocks.
	pub fn retracted(&self) -> &[H256] {
		&self.retracted
	}

	/// All blocks in the route.
	pub fn route(&self) -> &[(H256, ChainRouteType)] {
		&self.route
	}
}

/// Represents what has to be handled by actor listening to chain events
pub trait ChainNotify : Send + Sync {
	/// fires when chain has new blocks.
	fn new_blocks(
		&self,
		_imported: Vec<H256>,
		_invalid: Vec<H256>,
		_route: ChainRoute,
		_sealed: Vec<H256>,
		// Block bytes.
		_proposed: Vec<Bytes>,
		_duration: Duration,
	) {
		// does nothing by default
	}

	/// fires when chain achieves active mode
	fn start(&self) {
		// does nothing by default
	}

	/// fires when chain achieves passive mode
	fn stop(&self) {
		// does nothing by default
	}

	/// fires when chain broadcasts a message
	fn broadcast(&self, _message_type: ChainMessageType) {}

	/// fires when new transactions are received from a peer
	fn transactions_received(&self,
		_txs: &[UnverifiedTransaction],
		_peer_id: usize,
	) {
		// does nothing by default
	}
}
