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

use std::collections::BTreeMap;
use serde::Deserialize;
use serde::de::DeserializeOwned;

/// A genric wrapper over a `BTreeMap` for tests
#[derive(Deserialize)]
pub struct GenericTester<T: Ord, U>(BTreeMap<T, U>);

impl<T: Ord, U> IntoIterator for GenericTester<T, U> {
	type Item = <BTreeMap<T, U> as IntoIterator>::Item;
	type IntoIter = <BTreeMap<T, U> as IntoIterator>::IntoIter;

	fn into_iter(self) -> Self::IntoIter {
		self.0.into_iter()
	}
}

impl<T, U> GenericTester<T, U>
where
	T: DeserializeOwned + Ord,
	U: DeserializeOwned
{
	/// Loads test from json.
	pub fn load<R>(reader: R) -> Result<Self, serde_json::Error> where R: std::io::Read {
		serde_json::from_reader(reader)
	}
}
