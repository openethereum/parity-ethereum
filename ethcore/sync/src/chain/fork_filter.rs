// Copyright 2015-2020 Parity Technologies (UK) Ltd.
// This file is part of Open Ethereum.

// Open Ethereum is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Open Ethereum is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Open Ethereum.  If not, see <http://www.gnu.org/licenses/>.

//! This module contains a wrapper that connects this codebase with `ethereum-forkid` crate which provides `FORK_ID`
//! to support Ethereum network protocol, version 64 and above.

// Re-export ethereum-forkid crate contents here.
pub use ethereum_forkid::{BlockNumber, ForkId, RejectReason};

use client_traits::ChainInfo;
use ethereum_forkid::ForkFilter;
use parity_util_mem::MallocSizeOf;

/// Wrapper around fork filter that provides integration with `ForkFilter`.
#[derive(MallocSizeOf)]
pub struct ForkFilterApi {
	inner: ForkFilter,
}

impl ForkFilterApi {
	/// Create `ForkFilterApi` from `ChainInfo` and an `Iterator` over the hard forks.
	pub fn new<C: ?Sized + ChainInfo, I: IntoIterator<Item = BlockNumber>>(client: &C, forks: I) -> Self {
		let chain_info = client.chain_info();
		Self {
			inner: ForkFilter::new(chain_info.best_block_number, chain_info.genesis_hash, forks),
		}
	}

	#[cfg(test)]
	/// Dummy version of ForkFilterApi with no forks.
	pub fn new_dummy<C: ?Sized + ChainInfo>(client: &C) -> Self {
		let chain_info = client.chain_info();
		Self {
			inner: ForkFilter::new(chain_info.best_block_number, chain_info.genesis_hash, vec![]),
		}
	}

	fn update_head<C: ?Sized + ChainInfo>(&mut self, client: &C) {
		self.inner.set_head(client.chain_info().best_block_number);
	}

	/// Wrapper for `ForkFilter::current`
	pub fn current<C: ?Sized + ChainInfo>(&mut self, client: &C) -> ForkId {
		self.update_head(client);
		self.inner.current()
	}

	/// Wrapper for `ForkFilter::is_compatible`
	pub fn is_compatible<C: ?Sized + ChainInfo>(&mut self, client: &C, fork_id: ForkId) -> Result<(), RejectReason> {
		self.update_head(client);
		self.inner.is_compatible(fork_id)
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use spec::Spec;
	use ethcore::test_helpers::TestBlockChainClient;

	fn test_spec<F: Fn() -> Spec>(spec_builder: F, forks: Vec<BlockNumber>) {
		let spec = (spec_builder)();
		let genesis_hash = spec.genesis_header().hash();
		let spec_forks = spec.hard_forks.clone();
		let client = TestBlockChainClient::new_with_spec(spec);

		assert_eq!(
			ForkFilterApi::new(&client, spec_forks).inner,
			ForkFilter::new(0, genesis_hash, forks)
		);
	}

	#[test]
	fn ethereum_spec() {
		test_spec(
			|| spec::new_foundation(&String::new()),
			vec![
				1_150_000,
				1_920_000,
				2_463_000,
				2_675_000,
				4_370_000,
				7_280_000,
				9_069_000,
				9_200_000,
			],
		)
	}

	#[test]
	fn ropsten_spec() {
		test_spec(
			|| spec::new_ropsten(&String::new()),
			vec![
				10,
				1_700_000,
				4_230_000,
				4_939_394,
				6_485_846,
				7_117_117,
			],
		)
	}

	#[test]
	fn rinkeby_spec() {
		test_spec(
			|| spec::new_rinkeby(&String::new()),
			vec![
				1,
				2,
				3,
				1_035_301,
				3_660_663,
				4_321_234,
				5_435_345,
			],
		)
	}

	#[test]
	fn goerli_spec() {
		test_spec(
			|| spec::new_goerli(&String::new()),
			vec![
				1_561_651,
			],
		)
	}

	#[test]
	fn classic_spec() {
		test_spec(
			|| spec::new_classic(&String::new()),
		vec![
				1150000,
				2500000,
				3000000,
				5000000,
				5900000,
				8772000,
				9573000,
				10500839,
			],
		)
	}
}
