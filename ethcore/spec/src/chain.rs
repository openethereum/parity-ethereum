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

//! Load chain specifications for all chains supported by OpenEthereum.

macro_rules! bundle_release_spec {
	($($path: expr => $name: ident), *) => {
		$(
			/// Bundled release spec
			pub fn $name<'a, T: Into<crate::spec::SpecParams<'a>>>(params: T) -> crate::spec::Spec {
				let params = params.into();
				crate::spec::Spec::load(
					params,
					include_bytes!(concat!("../../res/", $path, ".json")) as &[u8]
				).expect(concat!("Chain spec ", $path, " is invalid."))
			}
		)*
	}
}

macro_rules! bundle_test_spec {
	($($path: expr => $name: ident), *) => {
		$(
			/// Bundled test spec
			pub fn $name() -> crate::spec::Spec {
				crate::spec::Spec::load(
					&::std::env::temp_dir(),
					include_bytes!(concat!("../../res/", $path, ".json")) as &[u8]
				).expect(concat!("Chain spec ", $path, " is invalid."))
			}
		)*
	}
}

macro_rules! bundle_custom_spec {
	($($path: expr => $name: ident), *) => {
		$(
			/// Bundled test spec
			pub fn $name() -> crate::spec::Spec {
				crate::spec::Spec::load(
					&::std::env::temp_dir(),
					include_bytes!(concat!("../../res/", $path, ".json")) as &[u8]
				).expect(concat!("Chain spec ", $path, " is invalid."))
			}
		)*
	}
}

macro_rules! bundle_test_machine {
	($($path: expr => $name: ident), *) => {
		$(
			/// Bundled test spec
			pub fn $name() -> machine::Machine {
				crate::spec::Spec::load_machine(
					include_bytes!(concat!("../../res/", $path, ".json")) as &[u8]
				).expect(concat!("Chain spec ", $path, " is invalid."))
			}
		)*
	}
}

bundle_release_spec! {
	"ethereum/callisto" => new_callisto,
	"ethereum/classic" => new_classic,
	"ethereum/classic_no_phoenix" => new_classic_no_phoenix,
	"ethereum/ellaism" => new_ellaism,
	"ethereum/ethercore" => new_ethercore,
	"ethereum/evancore" => new_evancore,
	"ethereum/evantestcore" => new_evantestcore,
	"ethereum/ewc" => new_ewc,
	"ethereum/expanse" => new_expanse,
	"ethereum/foundation" => new_foundation,
	"ethereum/goerli" => new_goerli,
	"ethereum/kotti" => new_kotti,
	"ethereum/kovan" => new_kovan,
	"ethereum/mix" => new_mix,
	"ethereum/mordor" => new_mordor,
	"ethereum/musicoin" => new_musicoin,
	"ethereum/poacore" => new_poanet,
	"ethereum/poasokol" => new_sokol,
	"ethereum/rinkeby" => new_rinkeby,
	"ethereum/ropsten" => new_ropsten,
	"ethereum/volta" => new_volta,
	"ethereum/xdai" => new_xdai,
	"ethereum/yolov1" => new_yolov1
}

bundle_test_spec! {
	"ethereum/test-specs/berlin_test" => new_berlin_test,
	"ethereum/test-specs/byzantium_test" => new_byzantium_test,
	"ethereum/test-specs/constantinople_test" => new_constantinople_test,
	"ethereum/test-specs/eip150_test" => new_eip150_test,
	"ethereum/test-specs/eip161_test" => new_eip161_test,
	"ethereum/test-specs/eip210_test" => new_eip210_test,
	"ethereum/test-specs/frontier_like_test" => new_mainnet_like,
	"ethereum/test-specs/frontier_test" => new_frontier_test,
	"ethereum/test-specs/homestead_test" => new_homestead_test,
	"ethereum/test-specs/istanbul_test" => new_istanbul_test,
	"ethereum/test-specs/kovan_wasm_test" => new_kovan_wasm_test,
	"ethereum/test-specs/mcip3_test" => new_mcip3_test,
	"ethereum/test-specs/constantinople_fix_test" => new_constantinople_fix_test,
	"ethereum/test-specs/eip158_to_byzantiumat5_test" => new_eip158_to_byzantiumat5_test,
	"ethereum/test-specs/byzantium_to_constantinoplefixat5_test" => new_byzantium_to_constantinoplefixat5_test
}

bundle_custom_spec! {
	"authority_round" => new_test_round,
	"authority_round_block_reward_contract" => new_test_round_block_reward_contract,
	"authority_round_empty_steps" => new_test_round_empty_steps,
	"authority_round_randomness_contract" => new_test_round_randomness_contract,
	"constructor" => new_test_constructor,
	"instant_seal" => new_instant,
	"null" => new_null,
	"null_morden" => new_test,
	"null_morden_with_finality" => new_test_with_finality,
	"null_morden_with_reward" => new_test_with_reward,
	"validator_contract" => new_validator_contract,
	"validator_multi" => new_validator_multi,
	"validator_safe_contract" => new_validator_safe_contract
}

bundle_test_machine! {
	"null_morden" => new_test_machine,
	"ethereum/test-specs/berlin_test" => new_berlin_test_machine,
	"ethereum/test-specs/byzantium_test" => new_byzantium_test_machine,
	"ethereum/test-specs/constantinople_test" => new_constantinople_test_machine,
	"ethereum/test-specs/eip210_test" => new_eip210_test_machine,
	"ethereum/test-specs/frontier_test" => new_frontier_test_machine,
	"ethereum/test-specs/homestead_test" => new_homestead_test_machine,
	"ethereum/test-specs/istanbul_test" => new_istanbul_test_machine,
	"ethereum/test-specs/kovan_wasm_test" => new_kovan_wasm_test_machine,
	"ethereum/test-specs/mcip3_test" => new_mcip3_test_machine,
	"ethereum/test-specs/constantinople_fix_test" => new_constantinople_fix_test_machine
}

#[cfg(test)]
mod tests {
	use account_state::State;
	use common_types::{view, views::BlockView};
	use ethereum_types::U256;
	use tempfile::TempDir;
	use ethcore::test_helpers::get_temp_state_db;

	use super::{new_ropsten, new_foundation, new_berlin_test_machine};

	#[test]
	fn ensure_db_good() {
		let tempdir = TempDir::new().unwrap();
		let spec = new_ropsten(&tempdir.path());
		let engine = &spec.engine;
		let genesis_header = spec.genesis_header();
		let db = spec.ensure_db_good(get_temp_state_db(), &Default::default()).unwrap();
		let s = State::from_existing(db, genesis_header.state_root().clone(), engine.account_start_nonce(0), Default::default()).unwrap();
		assert_eq!(s.balance(&"0000000000000000000000000000000000000001".parse().unwrap()).unwrap(), 1u64.into());
		assert_eq!(s.balance(&"0000000000000000000000000000000000000002".parse().unwrap()).unwrap(), 1u64.into());
		assert_eq!(s.balance(&"0000000000000000000000000000000000000003".parse().unwrap()).unwrap(), 1u64.into());
		assert_eq!(s.balance(&"0000000000000000000000000000000000000004".parse().unwrap()).unwrap(), 1u64.into());
		assert_eq!(s.balance(&"874b54a8bd152966d63f706bae1ffeb0411921e5".parse().unwrap()).unwrap(), U256::from(1000000000000000000000000000000u128));
		assert_eq!(s.balance(&"0000000000000000000000000000000000000000".parse().unwrap()).unwrap(), 1u64.into());
	}

	#[test]
	fn ropsten() {
		let tempdir = TempDir::new().unwrap();
		let ropsten = new_ropsten(&tempdir.path());

		assert_eq!(ropsten.state_root, "217b0bbcfb72e2d57e28f33cb361b9983513177755dc3f33ce3e7022ed62b77b".parse().unwrap());
		let genesis = ropsten.genesis_block();
		assert_eq!(view!(BlockView, &genesis).header_view().hash(), "41941023680923e0fe4d74a34bdac8141f2540e3ae90623718e47d66d1ca4a2d".parse().unwrap());
	}

	#[test]
	fn frontier() {
		let tempdir = TempDir::new().unwrap();
		let frontier = new_foundation(&tempdir.path());

		assert_eq!(frontier.state_root, "d7f8974fb5ac78d9ac099b9ad5018bedc2ce0a72dad1827a1709da30580f0544".parse().unwrap());
		let genesis = frontier.genesis_block();
		assert_eq!(view!(BlockView, &genesis).header_view().hash(), "d4e56740f876aef8c010b86a40d5f56745a118d0906a34e69aec8c0db1cb8fa3".parse().unwrap());
	}

	#[test]
	fn berlin_test_spec() {
		let _ = new_berlin_test_machine();
	}
}
