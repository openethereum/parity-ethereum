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

//! Ethereum protocol module.
//!
//! Contains all Ethereum network specific stuff, such as denominations and
//! consensus specifications.

/// Export the ethash module.
pub mod ethash;
/// Export the denominations module.
pub mod denominations;

pub use self::ethash::{Ethash};
pub use self::denominations::*;

use super::spec::*;

/// Create a new Olympic chain spec.
pub fn new_olympic() -> Spec { Spec::load(include_bytes!("../../res/ethereum/olympic.json")) }

/// Create a new Frontier mainnet chain spec.
pub fn new_frontier() -> Spec { Spec::load(include_bytes!("../../res/ethereum/frontier.json")) }

/// Create a new Frontier chain spec as though it never changes to Homestead.
pub fn new_frontier_test() -> Spec { Spec::load(include_bytes!("../../res/ethereum/frontier_test.json")) }

/// Create a new Homestead chain spec as though it never changed from Frontier.
pub fn new_homestead_test() -> Spec { Spec::load(include_bytes!("../../res/ethereum/homestead_test.json")) }

/// Create a new Frontier main net chain spec without genesis accounts.
pub fn new_mainnet_like() -> Spec { Spec::load(include_bytes!("../../res/ethereum/frontier_like_test.json")) }

/// Create a new Morden chain spec.
pub fn new_morden() -> Spec { Spec::load(include_bytes!("../../res/ethereum/morden.json")) }

#[cfg(test)]
mod tests {
	use common::*;
	use state::*;
	use engine::*;
	use super::*;
	use tests::helpers::*;

	#[test]
	fn ensure_db_good() {
		let spec = new_morden();
		let engine = &spec.engine;
		let genesis_header = spec.genesis_header();
		let mut db_result = get_temp_journal_db();
		let mut db = db_result.take();
		spec.ensure_db_good(db.as_hashdb_mut());
		let s = State::from_existing(db, genesis_header.state_root.clone(), engine.account_start_nonce());
		assert_eq!(s.balance(&address_from_hex("0000000000000000000000000000000000000001")), U256::from(1u64));
		assert_eq!(s.balance(&address_from_hex("0000000000000000000000000000000000000002")), U256::from(1u64));
		assert_eq!(s.balance(&address_from_hex("0000000000000000000000000000000000000003")), U256::from(1u64));
		assert_eq!(s.balance(&address_from_hex("0000000000000000000000000000000000000004")), U256::from(1u64));
		assert_eq!(s.balance(&address_from_hex("102e61f5d8f9bc71d0ad4a084df4e65e05ce0e1c")), U256::from(1u64) << 200);
		assert_eq!(s.balance(&address_from_hex("0000000000000000000000000000000000000000")), U256::from(0u64));
	}

	#[test]
	fn morden() {
		let morden = new_morden();

		assert_eq!(morden.state_root(), H256::from_str("f3f4696bbf3b3b07775128eb7a3763279a394e382130f27c21e70233e04946a9").unwrap());
		let genesis = morden.genesis_block();
		assert_eq!(BlockView::new(&genesis).header_view().sha3(), H256::from_str("0cd786a2425d16f152c658316c423e6ce1181e15c3295826d7c9904cba9ce303").unwrap());

		let _ = morden.engine;
	}

	#[test]
	fn frontier() {
		let frontier = new_frontier();

		assert_eq!(frontier.state_root(), H256::from_str("d7f8974fb5ac78d9ac099b9ad5018bedc2ce0a72dad1827a1709da30580f0544").unwrap());
		let genesis = frontier.genesis_block();
		assert_eq!(BlockView::new(&genesis).header_view().sha3(), H256::from_str("d4e56740f876aef8c010b86a40d5f56745a118d0906a34e69aec8c0db1cb8fa3").unwrap());

		let _ = frontier.engine;
	}
}
