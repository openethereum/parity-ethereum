use crate::{hash::H256, uint::Uint};
use serde::Deserialize;

/// Blockchain test header deserializer.
#[derive(Debug, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DifficultyTestCase {
	/// Parent timestamp.
	pub parent_timestamp: Uint,
	/// Parent difficulty.
	pub parent_difficulty: Uint,
	/// Parent uncle hash.
	pub parent_uncles: H256,
	/// Current timestamp.
	pub current_timestamp: Uint,
	/// Current difficulty.
	pub current_difficulty: Uint,
	/// Current block number.
	pub current_block_number: Uint,
}

/// Type for running `Difficulty` tests
pub type DifficultyTest = super::tester::GenericTester<String, DifficultyTestCase>;
