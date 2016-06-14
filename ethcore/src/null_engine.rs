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
use util::{Address, U256};
use builtin::Builtin;
use engine::Engine;
use spec::CommonParams;
use evm::Schedule;
use env_info::EnvInfo;

/// An engine which does not provide any consensus mechanism.
pub struct NullEngine {
	params: CommonParams,
	builtins: BTreeMap<Address, Builtin>,
}

impl Default for NullEngine {
	fn default() -> Self {
		NullEngine {
			params: CommonParams {
				account_start_nonce: U256::from(0x10_00_00),
				maximum_extra_data_size: 0x20,
				min_gas_limit: U256::from(0x1388),
				network_id: U256::from(0x2),
			},
			builtins: BTreeMap::new(),
		}
	}
}

impl NullEngine {
	/// Returns new instance of NullEngine with default VM Factory
	pub fn new(params: CommonParams, builtins: BTreeMap<Address, Builtin>) -> Self {
		NullEngine{
			params: params,
			builtins: builtins,
		}
	}
}

impl Engine for NullEngine {
	fn name(&self) -> &str {
		"NullEngine"
	}

	fn params(&self) -> &CommonParams {
		&self.params
	}

	fn builtins(&self) -> &BTreeMap<Address, Builtin> {
		&self.builtins
	}

	fn schedule(&self, _env_info: &EnvInfo) -> Schedule {
		Schedule::new_homestead()
	}
}
