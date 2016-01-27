use util::hash::*;
use util::uint::*;

#[derive(Default, Debug, Serialize)]
pub struct Block {
	pub hash: H256,
	#[serde(rename="parentHash")]
	pub parent_hash: H256,
	#[serde(rename="sha3Uncles")]
	pub uncles_hash: H256,
	pub author: Address,
	// TODO: get rid of this one
	pub miner: Address,
	#[serde(rename="stateRoot")]
	pub state_root: H256,
	#[serde(rename="transactionsRoot")]
	pub transactions_root: H256,
	#[serde(rename="receiptsRoot")]
	pub receipts_root: H256,
	pub number: U256,
	#[serde(rename="gasUsed")]
	pub gas_used: U256,
	#[serde(rename="gasLimit")]
	pub gas_limit: U256,
	// TODO: figure out how to properly serialize bytes
	//#[serde(rename="extraData")]
	//extra_data: Vec<u8>,
	#[serde(rename="logsBloom")]
	pub logs_bloom: H2048,
	pub timestamp: U256,
	pub difficulty: U256,
	#[serde(rename="totalDifficulty")]
	pub total_difficulty: U256,
	pub uncles: Vec<U256>,
	pub transactions: Vec<U256>
}
