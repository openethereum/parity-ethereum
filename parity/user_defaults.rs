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

use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::collections::BTreeMap;
use serde::{Serialize, Serializer, Error, Deserialize, Deserializer};
use serde::de::{Visitor, MapVisitor};
use serde::de::impls::BTreeMapVisitor;
use serde_json::Value;
use serde_json::de::from_reader;
use serde_json::ser::to_string;
use util::journaldb::Algorithm;

pub struct UserDefaults {
	pub is_first_launch: bool,
	pub pruning: Algorithm,
	pub tracing: bool,
	pub fat_db: bool,
}

impl Serialize for UserDefaults {
	fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
	where S: Serializer {
		let mut map: BTreeMap<String, Value> = BTreeMap::new();
		map.insert("pruning".into(), Value::String(self.pruning.as_str().into()));
		map.insert("tracing".into(), Value::Bool(self.tracing));
		map.insert("fat_db".into(), Value::Bool(self.fat_db));
		map.serialize(serializer)
	}
}

struct UserDefaultsVisitor;

impl Deserialize for UserDefaults {
	fn deserialize<D>(deserializer: &mut D) -> Result<Self, D::Error>
	where D: Deserializer {
		deserializer.deserialize(UserDefaultsVisitor)
	}
}

impl Visitor for UserDefaultsVisitor {
	type Value = UserDefaults;

	fn visit_map<V>(&mut self, visitor: V) -> Result<Self::Value, V::Error>
	where V: MapVisitor {
		let mut map: BTreeMap<String, Value> = try!(BTreeMapVisitor::new().visit_map(visitor));
		let pruning: Value = try!(map.remove("pruning".into()).ok_or_else(|| Error::custom("missing pruning")));
		let pruning = try!(pruning.as_str().ok_or_else(|| Error::custom("invalid pruning value")));
		let pruning = try!(pruning.parse().map_err(|_| Error::custom("invalid pruning method")));
		let tracing: Value = try!(map.remove("tracing".into()).ok_or_else(|| Error::custom("missing tracing")));
		let tracing = try!(tracing.as_bool().ok_or_else(|| Error::custom("invalid tracing value")));
		let fat_db: Value = map.remove("fat_db".into()).unwrap_or_else(|| Value::Bool(false));
		let fat_db = try!(fat_db.as_bool().ok_or_else(|| Error::custom("invalid fat_db value")));

		let user_defaults = UserDefaults {
			is_first_launch: false,
			pruning: pruning,
			tracing: tracing,
			fat_db: fat_db,
		};

		Ok(user_defaults)
	}
}

impl Default for UserDefaults {
	fn default() -> Self {
		UserDefaults {
			is_first_launch: true,
			pruning: Algorithm::default(),
			tracing: false,
			fat_db: false,
		}
	}
}

impl UserDefaults {
	pub fn load<P>(path: P) -> Result<Self, String> where P: AsRef<Path> {
		match File::open(path) {
			Ok(file) => from_reader(file).map_err(|e| e.to_string()),
			_ => Ok(UserDefaults::default()),
		}
	}

	pub fn save<P>(self, path: P) -> Result<(), String> where P: AsRef<Path> {
		let mut file: File = try!(File::create(path).map_err(|_| "Cannot create user defaults file".to_owned()));
		file.write_all(to_string(&self).unwrap().as_bytes()).map_err(|_| "Failed to save user defaults".to_owned())
	}
}
