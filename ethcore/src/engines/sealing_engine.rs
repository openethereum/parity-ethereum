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
use util::hash::Address;
use builtin::Builtin;
use engines::Engine;
use spec::CommonParams;
use evm::Schedule;
use env_info::EnvInfo;
use block::ExecutedBlock;
use common::Bytes;
use account_provider::AccountProvider;

/// An engine which does not provide any consensus mechanism, just seals blocks internally.
pub struct SealingEngine {
	params: CommonParams,
	builtins: BTreeMap<Address, Builtin>,
}

impl SealingEngine {
	/// Returns new instance of SealingEngine with default VM Factory
	pub fn new(params: CommonParams, builtins: BTreeMap<Address, Builtin>) -> Self {
		SealingEngine{
			params: params,
			builtins: builtins,
		}
	}
}

impl Engine for SealingEngine {
	fn name(&self) -> &str {
		"SealingEngine"
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

	fn generate_seal(&self, _block: &ExecutedBlock, _accounts: Option<&AccountProvider>) -> Option<Vec<Bytes>> {
		Some(Vec::new())
	}
}
