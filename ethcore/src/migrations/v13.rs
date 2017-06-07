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

//! Block details struct upgrade. Adds a field for the delta since last epoch transition.

use std::sync::Arc;

use blockchain::extras;
use header::BlockNumber;

use util::{H256, U256};
use util::kvdb::Database;
use util::migration::{Batch, Config, Error, Migration, Progress};
use rlp::{Rlp, RlpStream};

// extras column at the time of migration.
const COL_EXTRA: Option<u32> = Some(3);

// block details extras key index.
const DETAILS_KEY_INDEX: u8 = 0;
const TRANSITION_KEY_INDEX: u8 = 5;
const KEY_LEN: usize = 264;

// migrate details to new details.
fn migrate_details(old_details: &[u8]) -> Vec<u8> {
	let rlp = Rlp::new(old_details);

	let number: BlockNumber = rlp.val_at(0);
	let total_difficulty: U256 = rlp.val_at(1);
	let parent: H256 = rlp.val_at(2);
	let children: Vec<H256> = rlp.list_at(3);

	let mut stream = RlpStream::new_list(5);
	stream.append(&number)
		.append(&total_difficulty)
		.append(&parent)
		.append_list(&children)
		.append(&number); // only 0 for genesis since it's a special case.

	stream.out()
}

/// Block details struct upgrade. Adds a field for the delta since last epoch transition.
///
/// No transitions exist from before this migration, so the delta will always equal the
/// block number minus 1.
#[derive(Default)]
pub struct ToV13(Progress);

impl Migration for ToV13 {
	fn columns(&self) -> Option<u32> { Some(8) }

	fn version(&self) -> u32 { 13 }

	fn migrate(&mut self, source: Arc<Database>, config: &Config, dest: &mut Database, col: Option<u32>) -> Result<(), Error> {
		let mut batch = Batch::new(config, col);

		for (key, value) in source.iter(col).into_iter().flat_map(|inner| inner) {
			self.0.tick();
			let mut value = value.to_vec();

			if col == COL_EXTRA && key.len() == KEY_LEN && key[0] == DETAILS_KEY_INDEX {
				value = migrate_details(&value);
			}

			// don't migrate over any transition entries from the old format.
			if col != COL_EXTRA || key[0] != TRANSITION_KEY_INDEX {
				batch.insert(key.to_vec(), value, dest)?
			}
		}

		batch.commit(dest)
	}
}
