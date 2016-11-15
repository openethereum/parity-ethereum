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
pub struct InstantSeal {
	params: CommonParams,
	builtins: BTreeMap<Address, Builtin>,
}

impl InstantSeal {
	/// Returns new instance of InstantSeal with default VM Factory
	pub fn new(params: CommonParams, builtins: BTreeMap<Address, Builtin>) -> Self {
		InstantSeal {
			params: params,
			builtins: builtins,
		}
	}
}

impl Engine for InstantSeal {
	fn name(&self) -> &str {
		"InstantSeal"
	}

	fn params(&self) -> &CommonParams {
		&self.params
	}

	fn builtins(&self) -> &BTreeMap<Address, Builtin> {
		&self.builtins
	}

	fn schedule(&self, _env_info: &EnvInfo) -> Schedule {
		Schedule::new_post_eip150(usize::max_value(), false, false, false)
	}

	fn generate_seal(&self, _block: &ExecutedBlock, _accounts: Option<&AccountProvider>) -> Option<Vec<Bytes>> {
		Some(Vec::new())
	}
}

#[cfg(test)]
mod tests {
	use common::*;
	use tests::helpers::*;
	use account_provider::AccountProvider;
	use spec::Spec;
	use block::*;

	/// Create a new test chain spec with `BasicAuthority` consensus engine.
	fn new_test_instant() -> Spec { Spec::load(include_bytes!("../../res/instant_seal.json")) }

	#[test]
	fn instant_can_seal() {
		let tap = AccountProvider::transient_provider();
		let addr = tap.insert_account("".sha3(), "").unwrap();

		let spec = new_test_instant();
		let engine = &*spec.engine;
		let genesis_header = spec.genesis_header();
		let mut db_result = get_temp_state_db();
		let mut db = db_result.take();
		spec.ensure_db_good(&mut db).unwrap();
		let last_hashes = Arc::new(vec![genesis_header.hash()]);
		let vm_factory = Default::default();
		let b = OpenBlock::new(engine, &vm_factory, Default::default(), false, db, &genesis_header, last_hashes, addr, (3141562.into(), 31415620.into()), vec![]).unwrap();
		let b = b.close_and_lock();
		// Seal with empty AccountProvider.
		let seal = engine.generate_seal(b.block(), Some(&tap)).unwrap();
		assert!(b.try_seal(engine, seal).is_ok());
	}

	#[test]
	fn instant_cant_verify() {
		let engine = new_test_instant().engine;
		let mut header: Header = Header::default();

		assert!(engine.verify_block_basic(&header, None).is_ok());

		header.set_seal(vec![rlp::encode(&Signature::zero()).to_vec()]);

		assert!(engine.verify_block_unordered(&header, None).is_ok());
	}
}
