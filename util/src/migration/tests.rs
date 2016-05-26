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
use migration::{Error, Destination, Migration, Manager, Config};

impl Destination for BTreeMap<Vec<u8>, Vec<u8>> {
	fn commit(&mut self, batch: BTreeMap<Vec<u8>, Vec<u8>>) -> Result<(), Error> {
		self.extend(batch);
		Ok(())
	}
}

struct Migration0;

impl Migration for Migration0 {
	fn version(&self) -> u32 {
		1
	}

	fn simple_migrate(&self, key: Vec<u8>, value: Vec<u8>) -> Option<(Vec<u8>, Vec<u8>)> {
		let mut key = key;
		key.push(0x11);
		let mut value = value;
		value.push(0x22);
		Some((key, value))
	}
}

struct Migration1;

impl Migration for Migration1 {
	fn version(&self) -> u32 {
		2
	}

	fn simple_migrate(&self, key: Vec<u8>, _value: Vec<u8>) -> Option<(Vec<u8>, Vec<u8>)> {
		Some((key, vec![]))
	}
}

#[test]
fn one_simple_migration() {
	let mut manager = Manager::new(Config::default());
	let keys = vec![vec![], vec![1u8]];
	let values = vec![vec![], vec![1u8]];
	let db = keys.into_iter().zip(values.into_iter());

	let expected_keys = vec![vec![0x11u8], vec![1, 0x11]];
	let expected_values = vec![vec![0x22u8], vec![1, 0x22]];
	let expected_db = expected_keys.into_iter().zip(expected_values.into_iter()).collect::<BTreeMap<_, _>>();

	let mut result = BTreeMap::new();
	manager.add_migration(Migration0).unwrap();
	manager.execute(db, 0, &mut result).unwrap();
	assert_eq!(expected_db, result);
}

#[test]
#[should_panic]
fn no_migration_needed() {
	let mut manager = Manager::new(Config::default());
	let keys = vec![vec![], vec![1u8]];
	let values = vec![vec![], vec![1u8]];
	let db = keys.into_iter().zip(values.into_iter());

	let mut result = BTreeMap::new();
	manager.add_migration(Migration0).unwrap();
	manager.execute(db, 1, &mut result).unwrap();
}

#[test]
fn multiple_migrations() {
	let mut manager = Manager::new(Config::default());
	let keys = vec![vec![], vec![1u8]];
	let values = vec![vec![], vec![1u8]];
	let db = keys.into_iter().zip(values.into_iter());

	let expected_keys = vec![vec![0x11u8], vec![1, 0x11]];
	let expected_values = vec![vec![], vec![]];
	let expected_db = expected_keys.into_iter().zip(expected_values.into_iter()).collect::<BTreeMap<_, _>>();

	let mut result = BTreeMap::new();
	manager.add_migration(Migration0).unwrap();
	manager.add_migration(Migration1).unwrap();
	manager.execute(db, 0, &mut result).unwrap();
	assert_eq!(expected_db, result);
}

#[test]
fn second_migration() {
	let mut manager = Manager::new(Config::default());
	let keys = vec![vec![], vec![1u8]];
	let values = vec![vec![], vec![1u8]];
	let db = keys.into_iter().zip(values.into_iter());

	let expected_keys = vec![vec![], vec![1u8]];
	let expected_values = vec![vec![], vec![]];
	let expected_db = expected_keys.into_iter().zip(expected_values.into_iter()).collect::<BTreeMap<_, _>>();

	let mut result = BTreeMap::new();
	manager.add_migration(Migration0).unwrap();
	manager.add_migration(Migration1).unwrap();
	manager.execute(db, 1, &mut result).unwrap();
	assert_eq!(expected_db, result);
}

#[test]
fn is_migration_needed() {
	let mut manager = Manager::new(Config::default());
	manager.add_migration(Migration0).unwrap();
	manager.add_migration(Migration1).unwrap();

	assert!(manager.is_needed(0));
	assert!(manager.is_needed(1));
	assert!(!manager.is_needed(2));
}
