use util::hash::*;

#[derive(Default, Debug, Serialize)]
pub struct SyncStatus {
	#[serde(rename="startingBlock")]
	pub starting_block: H256,
	#[serde(rename="currentBlock")]
	pub current_block: H256,
	#[serde(rename="highestBlock")]
	pub highest_block: H256,
}
