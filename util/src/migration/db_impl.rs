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

//! `kvdb::Database` as `migration::Destination`

use std::collections::BTreeMap;
use kvdb::{Database, DatabaseIterator, DBTransaction};
use migration::{Destination, Error};

/// Database iterator with `Item` complient with migration `Manager` interface.
pub struct MigrationIterator {
	iter: DatabaseIterator,
}

impl From<DatabaseIterator> for MigrationIterator {
	fn from(iter: DatabaseIterator) -> Self {
		MigrationIterator {
			iter: iter
		}
	}
}

impl Iterator for MigrationIterator {
	type Item = (Vec<u8>, Vec<u8>);

	fn next(&mut self) -> Option<Self::Item> {
		self.iter.next().map(|(k, v)| (k.to_vec(), v.to_vec()))
	}
}

impl Destination for Database {
	fn commit(&mut self, batch: BTreeMap<Vec<u8>, Vec<u8>>) -> Result<(), Error> {
		let transaction = DBTransaction::new();

		for keypair in &batch {
			try!(transaction.put(&keypair.0, &keypair.1).map_err(Error::Custom))
		}

		self.write(transaction).map_err(Error::Custom)
	}
}

