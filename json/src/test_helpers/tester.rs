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
