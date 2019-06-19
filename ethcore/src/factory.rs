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

use trie::TrieFactory;
use ethtrie::RlpCodec;
use account_db::Factory as AccountFactory;
use evm::{Factory as EvmFactory, VMType};
use vm::{Exec, ActionParams, Schedule};
use wasm::WasmInterpreter;
use keccak_hasher::KeccakHasher;

const WASM_MAGIC_NUMBER: &'static [u8; 4] = b"\0asm";

/// Virtual machine factory
#[derive(Default, Clone)]
pub struct VmFactory {
	evm: EvmFactory,
}

impl VmFactory {
	pub fn create(&self, params: ActionParams, schedule: &Schedule, depth: usize) -> Box<dyn Exec> {
		if schedule.wasm.is_some() && params.code.as_ref().map_or(false, |code| code.len() > 4 && &code[0..4] == WASM_MAGIC_NUMBER) {
			Box::new(WasmInterpreter::new(params))
		} else {
			self.evm.create(params, schedule, depth)
		}
	}

	pub fn new(evm: VMType, cache_size: usize) -> Self {
		VmFactory { evm: EvmFactory::new(evm, cache_size) }
	}
}

impl From<EvmFactory> for VmFactory {
	fn from(evm: EvmFactory) -> Self {
		VmFactory { evm: evm }
	}
}

/// Collection of factories.
#[derive(Default, Clone)]
pub struct Factories {
	/// factory for evm.
	pub vm: VmFactory,
	/// factory for tries.
	pub trie: TrieFactory<KeccakHasher, RlpCodec>,
	/// factory for account databases.
	pub accountdb: AccountFactory,
}
