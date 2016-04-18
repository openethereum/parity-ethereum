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

//! Extras db utils.

use util::{H264, DBTransaction, Database};
use util::rlp::{encode, Encodable, decode, Decodable};

/// Should be used to get database key associated with given value.
pub trait Key<T> {
	/// Returns db key.
	fn key(&self) -> H264;
}

/// Should be used to write value into database.
pub trait Writable {
	/// Writes key into database.
	fn write<T>(&self, key: &Key<T>, value: &T) where T: Encodable;
}

/// Should be used to read values from database.
pub trait Readable {
	/// Returns value for given key.
	fn read<T>(&self, key: &Key<T>) -> Option<T> where T: Decodable;
	/// Returns true if given value exists.
	fn exists<T>(&self, key: &Key<T>) -> bool;
}

impl Writable for DBTransaction {
	fn write<T>(&self, key: &Key<T>, value: &T) where T: Encodable {
		let result = self.put(&key.key(), &encode(value));
		if let Err(err) = result {
			panic!("db put failed, key: {:?}, err: {:?}", key.key(), err);
		}
	}
}

impl Readable for Database {
	fn read<T>(&self, key: &Key<T>) -> Option<T> where T: Decodable {
		let result = self.get(&key.key());

		match result {
			Ok(option) => option.map(|v| decode(&v)),
			Err(err) => {
				panic!("db get failed, key: {:?}, err: {:?}", key.key(), err);
			}
		}
	}

	fn exists<T>(&self, key: &Key<T>) -> bool {
		let result = self.get(&key.key());

		match result {
			Ok(v) => v.is_some(),
			Err(err) => {
				panic!("db get failed, key: {:?}, err: {:?}", key.key(), err);
			}
		}
	}
}
