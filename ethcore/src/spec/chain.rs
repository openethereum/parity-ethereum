// Copyright 2015-2019 Parity Technologies (UK) Ltd.
// This file is part of Parity Ethereum.

// Parity Ethereum is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity Ethereum is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity Ethereum.  If not, see <http://www.gnu.org/licenses/>.

//! Load chain specifications for all chains supported by the parity-ethereum client.

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

macro_rules! bundle_test_machine {
	($($path: expr => $name: ident), *) => {
		$(
			/// Bundled test spec
			pub fn $name() -> crate::machine::Machine {
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
	"ethereum/ellaism" => new_ellaism,
	"ethereum/expanse" => new_expanse,
	"ethereum/foundation" => new_foundation,
	"ethereum/goerli" => new_goerli,
	"ethereum/kotti" => new_kotti,
	"ethereum/kovan" => new_kovan,
	"ethereum/mix" => new_mix,
	"ethereum/morden" => new_morden,
	"ethereum/musicoin" => new_musicoin,
	"ethereum/poacore" => new_poanet,
	"ethereum/poasokol" => new_sokol,
	"ethereum/rinkeby" => new_rinkeby,
	"ethereum/ropsten" => new_ropsten,
	"ethereum/volta" => new_volta,
	"ethereum/ewc" => new_ewc
}

bundle_test_spec! {
	"authority_round" => new_test_round,
	"authority_round_block_reward_contract" => new_test_round_block_reward_contract,
	"authority_round_empty_steps" => new_test_round_empty_steps,
	"constructor" => new_test_constructor,
	"ethereum/byzantium_test" => new_byzantium_test,
	"ethereum/constantinople_test" => new_constantinople_test,
	"ethereum/eip150_test" => new_eip150_test,
	"ethereum/eip161_test" => new_eip161_test,
	"ethereum/eip210_test" => new_eip210_test,
	"ethereum/frontier_like_test" => new_mainnet_like,
	"ethereum/frontier_test" => new_frontier_test,
	"ethereum/homestead_test" => new_homestead_test,
	"ethereum/kovan_wasm_test" => new_kovan_wasm_test,
	"ethereum/mcip3_test" => new_mcip3_test,
	"ethereum/morden" => new_morden_test,
	"ethereum/ropsten" => new_ropsten_test,
	"ethereum/st_peters_test" => new_constantinople_fix_test,
	"ethereum/transition_test" => new_transition_test,
	"instant_seal" => new_instant,
	"null" => new_null,
	"null_morden" => new_test,
	"null_morden_with_reward" => new_test_with_reward,
	"validator_contract" => new_validator_contract,
	"validator_multi" => new_validator_multi,
	"validator_safe_contract" => new_validator_safe_contract
}

bundle_test_machine! {
	"ethereum/byzantium_test" => new_byzantium_test_machine,
	"ethereum/constantinople_test" => new_constantinople_test_machine,
	"ethereum/eip210_test" => new_eip210_test_machine,
	"ethereum/frontier_test" => new_frontier_test_machine,
	"ethereum/homestead_test" => new_homestead_test_machine,
	"ethereum/kovan_wasm_test" => new_kovan_wasm_test_machine,
	"null_morden" => new_test_machine
}

#[cfg(test)]
mod tests {
	use account_state::State;
	use ethereum_types::U256;
	use tempdir::TempDir;
	use test_helpers::get_temp_state_db;
	use types::{view, views::BlockView};

	use super::{new_morden, new_foundation};

	#[test]
	fn ensure_db_good() {
		let tempdir = TempDir::new("").unwrap();
		let spec = new_morden(&tempdir.path());
		let engine = &spec.engine;
		let genesis_header = spec.genesis_header();
		let db = spec.ensure_db_good(get_temp_state_db(), &Default::default()).unwrap();
		let s = State::from_existing(db, genesis_header.state_root().clone(), engine.account_start_nonce(0), Default::default()).unwrap();
		assert_eq!(s.balance(&"0000000000000000000000000000000000000001".parse().unwrap()).unwrap(), 1u64.into());
		assert_eq!(s.balance(&"0000000000000000000000000000000000000002".parse().unwrap()).unwrap(), 1u64.into());
		assert_eq!(s.balance(&"0000000000000000000000000000000000000003".parse().unwrap()).unwrap(), 1u64.into());
		assert_eq!(s.balance(&"0000000000000000000000000000000000000004".parse().unwrap()).unwrap(), 1u64.into());
		assert_eq!(s.balance(&"102e61f5d8f9bc71d0ad4a084df4e65e05ce0e1c".parse().unwrap()).unwrap(), U256::from(1u64) << 200);
		assert_eq!(s.balance(&"0000000000000000000000000000000000000000".parse().unwrap()).unwrap(), 0u64.into());
	}

	#[test]
	fn morden() {
		let tempdir = TempDir::new("").unwrap();
		let morden = new_morden(&tempdir.path());

		assert_eq!(morden.state_root, "f3f4696bbf3b3b07775128eb7a3763279a394e382130f27c21e70233e04946a9".parse().unwrap());
		let genesis = morden.genesis_block();
		assert_eq!(view!(BlockView, &genesis).header_view().hash(), "0cd786a2425d16f152c658316c423e6ce1181e15c3295826d7c9904cba9ce303".parse().unwrap());
	}

	#[test]
	fn frontier() {
		let tempdir = TempDir::new("").unwrap();
		let frontier = new_foundation(&tempdir.path());

		assert_eq!(frontier.state_root, "d7f8974fb5ac78d9ac099b9ad5018bedc2ce0a72dad1827a1709da30580f0544".parse().unwrap());
		let genesis = frontier.genesis_block();
		assert_eq!(view!(BlockView, &genesis).header_view().hash(), "d4e56740f876aef8c010b86a40d5f56745a118d0906a34e69aec8c0db1cb8fa3".parse().unwrap());
	}
}
