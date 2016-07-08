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

//! This migration migrates the state db to use an accountdb which ensures uniqueness
//! using an address' hash as opposed to the address itself.

use std::collections::HashMap;

use util::Bytes;
use util::hash::{Address, FixedHash, H256};
use util::kvdb::Database;
use util::migration::{Batch, Config, Error, Migration, SimpleMigration};
use util::rlp::{decode, Rlp, RlpStream, Stream, View};
use util::sha3::Hashable;

// attempt to migrate a key, value pair. Err if migration not possible.
fn attempt_migrate(mut key_h: H256, val: &[u8]) -> Result<H256, H256> {
	let val_hash = val.sha3();

	if key_h != val_hash {
		// this is a key which has been xor'd with an address.
		// recover the address.
		let address = key_h ^ val_hash;

		// check that the address is actually a 20-byte value.
		// the leftmost 12 bytes should be zero.
		if &address[0..12] != &[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0] {
			return Err(key_h);
		}

		let address_hash = Address::from(address).sha3();

		// create the xor'd key in place.
		key_h.copy_from_slice(&*val_hash);
		assert_eq!(key_h, val_hash);

		let last_src: &[u8] = &*address_hash;
		let last_dst: &mut [u8] = &mut *key_h;
		for (k, a) in last_dst[12..].iter_mut().zip(&last_src[12..]) {
			*k ^= *a;
		}
	}

	Ok(key_h)
}

/// Version for ArchiveDB.
pub struct ArchiveV7;

impl SimpleMigration for ArchiveV7 {
	fn version(&self) -> u32 {
		7
	}

	fn simple_migrate(&mut self, key: Vec<u8>, value: Vec<u8>) -> Option<(Vec<u8>, Vec<u8>)> {
		if key.len() != 32 {
			// metadata key, ignore.
			return Some((key, value));
		}

		let key_h = H256::from_slice(&key[..]);
		let migrated = attempt_migrate(key_h, &value[..])
			.expect("no 32-bit metadata keys in this version of archive; qed");

		Some((migrated[..].to_owned(), value))
	}
}

/// Version for OverlayRecent database.
/// more involved than the archive version because of journaling.
#[derive(Default)]
pub struct OverlayRecentV7 {
	migrated_keys: HashMap<H256, H256>,
}

impl OverlayRecentV7 {
	// walk all journal entries in the database backwards,
	// replacing any known migrated keys with their counterparts
	// and then committing again.
	fn migrate_journal(&self, source: &Database, mut batch: Batch, dest: &mut Database) -> Result<(), Error> {
		// re-written here because it may change in the journaldb module.
		const V7_LATEST_ERA_KEY: &'static [u8] = &[ b'l', b'a', b's', b't', 0, 0, 0, 0, 0, 0, 0, 0 ];
		const PADDING : [u8; 10] = [0u8; 10];


		if let Some(val) = source.get(V7_LATEST_ERA_KEY).expect("Low-level database error.") {
			try!(batch.insert(V7_LATEST_ERA_KEY.into(), val.to_owned(), dest));

			let mut era = decode::<u64>(&val);
			loop {
				let mut index: usize = 0;
				loop {
					let entry_key = {
						let mut r = RlpStream::new_list(3);
						r.append(&era).append(&index).append(&&PADDING[..]);
						r.out()
					};

					if let Some(journal_raw) = source.get(&entry_key).expect("Low-level database error.") {
						let rlp = Rlp::new(&journal_raw);
						let id: H256 = rlp.val_at(0);
						let mut inserted_keys: Vec<(H256, Bytes)> = Vec::new();

						// migrate all inserted keys.
						for r in rlp.at(1).iter() {
							let old_key: H256 = r.val_at(0);
							let v: Bytes = r.val_at(1);

							let key = match self.migrated_keys.get(&old_key) {
								Some(new) => new.clone(),
								None => old_key.clone(),
							};

							inserted_keys.push((key, v));
						}

						// migrate all deleted keys.
						let mut deleted_keys: Vec<H256> = rlp.val_at(2);
						for old_key in &mut deleted_keys {
							if let Some(new) = self.migrated_keys.get(&*old_key) {
								*old_key = new.clone();
							}
						}

						// rebuild the journal entry rlp.
						let mut stream = RlpStream::new_list(3);
						stream.append(&id);
						stream.begin_list(inserted_keys.len());
						for (k, v) in inserted_keys {
							stream.begin_list(2).append(&k).append(&v);
						}

						stream.append(&deleted_keys);

						// and insert it into the new database.
						try!(batch.insert(entry_key, stream.out(), dest));
						index += 1;
					} else {
						break;
					}
				}

				if index == 0 || era == 0 {
					break;
				}
				era -= 1;
			}
		}
		Ok(())
	}
}

impl Migration for OverlayRecentV7 {
	fn version(&self) -> u32 { 7 }

	// walk all records in the database, attempting to migrate any possible and
	// keeping records of those that we do. then migrate the journal using
	// this information.
	fn migrate(&mut self, source: &Database, config: &Config, dest: &mut Database) -> Result<(), Error> {
		let mut batch = Batch::new(config);

		for (key, value) in source.iter().filter(|&(ref k, _)| k.len() == 32) {
			let key_h = H256::from_slice(&key[..]);
			if let Ok(new_key) = attempt_migrate(key_h.clone(), &value) {
				self.migrated_keys.insert(key_h, new_key);
				try!(batch.insert(new_key[..].to_owned(), value.into_vec(), dest));
			}
		}

		self.migrate_journal(source, batch, dest)
	}
}