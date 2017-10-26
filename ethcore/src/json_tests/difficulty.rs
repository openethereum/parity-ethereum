// Copyright 2015-2017 Parity Technologies (UK) Ltd.
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

use ethjson;
use header::Header;
use bigint::prelude::U256;
use spec::Spec;

pub fn json_difficulty_test(json_data: &[u8], spec: Spec) -> Vec<String> {
	::ethcore_logger::init_log();
	let tests = ethjson::test::DifficultyTest::load(json_data).unwrap();
	let engine = &spec.engine;

	for (name, test) in tests.into_iter() {
		flush!("   - {}...", name);
		println!("   - {}...", name);

		let mut parent_header = Header::new();
		let block_number: u64 = test.current_block_number.into();
		parent_header.set_number(block_number - 1);
		parent_header.set_gas_limit(0x20000.into());
		parent_header.set_timestamp(test.parent_timestamp.into());
		parent_header.set_difficulty(test.parent_difficulty.into());
		parent_header.set_uncles_hash(test.parent_uncles.into());
		let mut header = Header::new();
		header.set_number(block_number);
		header.set_timestamp(test.current_timestamp.into());
		engine.populate_from_parent(&mut header, &parent_header);
		let expected_difficulty: U256 = test.current_difficulty.into();
		assert_eq!(header.difficulty(), &expected_difficulty);
		flushln!("ok");
	}
	vec![]
}

mod difficulty_test_byzantium {
	use super::json_difficulty_test;

	fn do_json_test(json_data: &[u8]) -> Vec<String> {
		json_difficulty_test(json_data, ::ethereum::new_byzantium_test())
	}

	declare_test!{DifficultyTests_difficultyByzantium, "BasicTests/difficultyByzantium.json"}
}


mod difficulty_test_foundation {
	use super::json_difficulty_test;

	fn do_json_test(json_data: &[u8]) -> Vec<String> {
		json_difficulty_test(json_data, ::ethereum::new_foundation(&::std::env::temp_dir()))
	}

	declare_test!{DifficultyTests_difficultyMainNetwork, "BasicTests/difficultyMainNetwork.json"}
}



