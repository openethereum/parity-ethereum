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

//! This migration migrates the state db to use an accountdb which ensures uniqueness
//! using an address' hash as opposed to the address itself.

use std::collections::HashMap;

use bigint::hash::H256;
use util::Address;
use bytes::Bytes;
use util::kvdb::Database;
use util::migration::{Batch, Config, Error, Migration, SimpleMigration, Progress};
use hash::keccak;
use std::sync::Arc;

use rlp::{decode, Rlp, RlpStream};

// attempt to migrate a key, value pair. None if migration not possible.
fn attempt_migrate(mut key_h: H256, val: &[u8]) -> Option<H256> {
	let val_hash = keccak(val);

	if key_h != val_hash {
		// this is a key which has been xor'd with an address.
		// recover the address.
		let address = key_h ^ val_hash;

		// check that the address is actually a 20-byte value.
		// the leftmost 12 bytes should be zero.
		if &address[0..12] != &[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0] {
			return None;
		}

		let address_hash = keccak(Address::from(address));

		// create the xor'd key in place.
		key_h.copy_from_slice(&*val_hash);
		assert_eq!(key_h, val_hash);

		{
			let last_src: &[u8] = &*address_hash;
			let last_dst: &mut [u8] = &mut *key_h;
			for (k, a) in last_dst[12..].iter_mut().zip(&last_src[12..]) {
				*k ^= *a;
			}
		}

		Some(key_h)
	} else {
		None
	}
}

/// Version for `ArchiveDB`.
#[derive(Default)]
pub struct ArchiveV7(Progress);

impl SimpleMigration for ArchiveV7 {

	fn columns(&self) -> Option<u32> { None }

	fn version(&self) -> u32 { 7 }

	fn simple_migrate(&mut self, key: Vec<u8>, value: Vec<u8>) -> Option<(Vec<u8>, Vec<u8>)> {
		self.0.tick();

		if key.len() != 32 {
			// metadata key, ignore.
			return Some((key, value));
		}

		let key_h = H256::from_slice(&key[..]);
		if let Some(new_key) = attempt_migrate(key_h, &value[..]) {
			Some((new_key[..].to_owned(), value))
		} else {
			Some((key, value))
		}
	}
}

// magic numbers and constants for overlay-recent at v6.
// re-written here because it may change in the journaldb module.
const V7_LATEST_ERA_KEY: &'static [u8] = &[ b'l', b'a', b's', b't', 0, 0, 0, 0, 0, 0, 0, 0 ];
const V7_VERSION_KEY: &'static [u8] = &[ b'j', b'v', b'e', b'r', 0, 0, 0, 0, 0, 0, 0, 0 ];
const DB_VERSION: u32 = 0x203;
const PADDING : [u8; 10] = [0u8; 10];

/// Version for `OverlayRecent` database.
/// more involved than the archive version because of journaling.
#[derive(Default)]
pub struct OverlayRecentV7 {
	migrated_keys: HashMap<H256, H256>,
}

impl OverlayRecentV7 {
	// walk all journal entries in the database backwards.
	// find migrations for any possible inserted keys.
	fn walk_journal(&mut self, source: Arc<Database>) -> Result<(), Error> {
		if let Some(val) = source.get(None, V7_LATEST_ERA_KEY).map_err(Error::Custom)? {
			let mut era = decode::<u64>(&val);
			loop {
				let mut index: usize = 0;
				loop {
					let entry_key = {
						let mut r = RlpStream::new_list(3);
						r.append(&era).append(&index).append(&&PADDING[..]);
						r.out()
					};

					if let Some(journal_raw) = source.get(None, &entry_key).map_err(Error::Custom)? {
						let rlp = Rlp::new(&journal_raw);

						// migrate all inserted keys.
						for r in rlp.at(1).iter() {
							let key: H256 = r.val_at(0);
							let v: Bytes = r.val_at(1);

							if self.migrated_keys.get(&key).is_none() {
								if let Some(new_key) = attempt_migrate(key, &v) {
									self.migrated_keys.insert(key, new_key);
								}
							}
						}
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

	// walk all journal entries in the database backwards.
	// replace all possible inserted/deleted keys with their migrated counterparts
	// and commit the altered entries.
	fn migrate_journal(&self, source: Arc<Database>, mut batch: Batch, dest: &mut Database) -> Result<(), Error> {
		if let Some(val) = source.get(None, V7_LATEST_ERA_KEY).map_err(Error::Custom)? {
			batch.insert(V7_LATEST_ERA_KEY.into(), val.clone().into_vec(), dest)?;

			let mut era = decode::<u64>(&val);
			loop {
				let mut index: usize = 0;
				loop {
					let entry_key = {
						let mut r = RlpStream::new_list(3);
						r.append(&era).append(&index).append(&&PADDING[..]);
						r.out()
					};

					if let Some(journal_raw) = source.get(None, &entry_key).map_err(Error::Custom)? {
						let rlp = Rlp::new(&journal_raw);
						let id: H256 = rlp.val_at(0);
						let mut inserted_keys: Vec<(H256, Bytes)> = Vec::new();

						// migrate all inserted keys.
						for r in rlp.at(1).iter() {
							let mut key: H256 = r.val_at(0);
							let v: Bytes = r.val_at(1);

							if let Some(new_key) = self.migrated_keys.get(&key) {
								key = *new_key;
							}

							inserted_keys.push((key, v));
						}

						// migrate all deleted keys.
						let mut deleted_keys: Vec<H256> = rlp.list_at(2);
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

						stream.append_list(&deleted_keys);

						// and insert it into the new database.
						batch.insert(entry_key, stream.out(), dest)?;

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
		batch.commit(dest)
	}
}

impl Migration for OverlayRecentV7 {

	fn columns(&self) -> Option<u32> { None }

	fn version(&self) -> u32 { 7 }

	// walk all records in the database, attempting to migrate any possible and
	// keeping records of those that we do. then migrate the journal using
	// this information.
	fn migrate(&mut self, source: Arc<Database>, config: &Config, dest: &mut Database, col: Option<u32>) -> Result<(), Error> {
		let mut batch = Batch::new(config, col);

		// check version metadata.
		match source.get(None, V7_VERSION_KEY).map_err(Error::Custom)? {
			Some(ref version) if decode::<u32>(&*version) == DB_VERSION => {}
			_ => return Err(Error::MigrationImpossible), // missing or wrong version
		}

		let mut count = 0;
		for (key, value) in source.iter(None).into_iter().flat_map(|inner| inner) {
			count += 1;
			if count == 100_000 {
				count = 0;
				flush!(".");
			}

			let mut key = key.into_vec();
			if key.len() == 32 {
				let key_h = H256::from_slice(&key[..]);
				if let Some(new_key) = attempt_migrate(key_h.clone(), &value) {
					self.migrated_keys.insert(key_h, new_key);
					key.copy_from_slice(&new_key[..]);
				}
			}

			batch.insert(key, value.into_vec(), dest)?;
		}

		self.walk_journal(source.clone())?;
		self.migrate_journal(source, batch, dest)
	}
}
