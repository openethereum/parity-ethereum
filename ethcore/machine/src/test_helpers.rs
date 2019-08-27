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

//! Provide facilities to create `Machine` instances for testing various networks.

use common_types::engines::params::CommonParams;
use ethjson;
use crate::Machine;

pub fn load_machine(reader: &[u8]) -> Machine {
	let spec = ethjson::spec::Spec::load(reader).expect("chain spec is invalid");

	let builtins = spec.accounts.builtins().into_iter().map(|p| (p.0.into(), From::from(p.1))).collect();
	let params = CommonParams::from(spec.params);

	if let ethjson::spec::Engine::Ethash(ref ethash) = spec.engine {
		Machine::with_ethash_extensions(params, builtins, ethash.params.clone().into())
	} else {
		Machine::regular(params, builtins)
	}
}


/// Create a new Foundation Frontier-era chain spec as though it never changes to Homestead.
pub fn new_frontier_test_machine() -> Machine { load_machine(include_bytes!("../../res/ethereum/frontier_test.json")) }

/// Create a new Foundation Homestead-era chain spec as though it never changed from Frontier.
pub fn new_homestead_test_machine() -> Machine { load_machine(include_bytes!("../../res/ethereum/homestead_test.json")) }

/// Create a new Foundation Homestead-EIP210-era chain spec as though it never changed from Homestead/Frontier.
pub fn new_eip210_test_machine() -> Machine { load_machine(include_bytes!("../../res/ethereum/eip210_test.json")) }

/// Create a new Foundation Byzantium era spec.
pub fn new_byzantium_test_machine() -> Machine { load_machine(include_bytes!("../../res/ethereum/byzantium_test.json")) }

/// Create a new Foundation Constantinople era spec.
pub fn new_constantinople_test_machine() -> Machine { load_machine(include_bytes!("../../res/ethereum/constantinople_test.json")) }

/// Create a new Foundation St. Peter's (Contantinople Fix) era spec.
pub fn new_constantinople_fix_test_machine() -> Machine { load_machine(include_bytes!("../../res/ethereum/st_peters_test.json")) }

/// Create a new Musicoin-MCIP3-era spec.
pub fn new_mcip3_test_machine() -> Machine { load_machine(include_bytes!("../../res/ethereum/mcip3_test.json")) }

/// Create new Kovan spec with wasm activated at certain block
pub fn new_kovan_wasm_test_machine() -> Machine { load_machine(include_bytes!("../../res/ethereum/kovan_wasm_test.json")) }
