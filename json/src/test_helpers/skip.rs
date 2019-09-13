use std::collections::BTreeMap;
use serde::Deserialize;

/// Test to skip (only if issue ongoing)
#[derive(Debug, PartialEq, Deserialize)]
pub struct SkipStates {
	/// Block tests
	pub block: Vec<BlockSkipStates>,
	/// State tests
	pub state: Vec<StateSkipStates>,

}

/// Block test to skip.
#[derive(Debug, PartialEq, Deserialize)]
pub struct BlockSkipStates {
	/// Issue reference.
	pub reference: String,
	/// Test failing name.
	pub failing: String,
	/// Items failing for the test.
	pub subtests: Vec<String>,
}

/// State test to skip.
#[derive(Debug, PartialEq, Deserialize)]
pub struct StateSkipStates {
	/// Issue reference.
	pub reference: String,
	/// Test failing name.
	pub failing: String,
	/// Items failing for the test.
	pub subtests: BTreeMap<String, StateSkipSubStates>
}

/// State subtest to skip.
#[derive(Debug, PartialEq, Deserialize)]
pub struct StateSkipSubStates {
	/// State test number of this item. Or '*' for all state.
	pub subnumbers: Vec<String>,
	/// Chain for this items.
	pub chain: String,
}

impl SkipStates {
	/// Empty skip states.
	pub fn empty() -> Self {
		SkipStates {
			block: Vec::new(),
			state: Vec::new(),
		}
	}

	/// Loads test from json.
	pub fn load<R>(reader: R) -> Result<Self, serde_json::Error> where R: std::io::Read {
		serde_json::from_reader(reader)
	}
}
