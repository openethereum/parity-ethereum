// Copyright 2015-2020 Parity Technologies (UK) Ltd.
// This file is part of OpenEthereum.

// OpenEthereum is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// OpenEthereum is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with OpenEthereum.  If not, see <http://www.gnu.org/licenses/>.

use ethereum_types::U256;
use ethjson;
use spec::Spec;
use std::path::Path;
use types::header::Header;

use super::HookType;

pub fn json_difficulty_test<H: FnMut(&str, HookType)>(
    path: &Path,
    json_data: &[u8],
    spec: Spec,
    start_stop_hook: &mut H,
) -> Vec<String> {
    let mut ret = Vec::new();
    let _ = env_logger::try_init();
    let tests = ethjson::test::DifficultyTest::load(json_data).expect(&format!(
        "Could not parse JSON difficulty test data from {}",
        path.display()
    ));
    let engine = &spec.engine;

    for (name, test) in tests.into_iter() {
        if !super::debug_include_test(&name) {
            continue;
        }

        start_stop_hook(&name, HookType::OnStart);

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
        if header.difficulty() == &expected_difficulty {
            flushln!("   - difficulty: {}...OK", name);
        } else {
            flushln!("   - difficulty: {}...FAILED", name);
            ret.push(format!("{}:{}", path.to_string_lossy(), name));
        }

        start_stop_hook(&name, HookType::OnStop);
    }
    ret
}
