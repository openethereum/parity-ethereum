use util::hash::*;
use util::uint::*;

#[derive(Serialize)]
pub struct Block {
	hash: H256,
	#[serde(rename="parentHash")]
	parent_hash: H256,
	#[serde(rename="sha3Uncles")]
	uncles_hash: H256,
	author: Address,
	// TODO: get rid of this one
	miner: Address,
	#[serde(rename="stateRoot")]
	state_root: H256,
	#[serde(rename="transactionsRoot")]
	transactions_root: H256,
	#[serde(rename="receiptsRoot")]
	receipts_root: H256,
	number: u64,
	#[serde(rename="gasUsed")]
	gas_used: U256,
	#[serde(rename="gasLimit")]
	gas_limit: U256,
	// TODO: figure out how to properly serialize bytes
	//#[serde(rename="extraData")]
	//extra_data: Vec<u8>,
	#[serde(rename="logsBloom")]
	logs_bloom: H2048,
	timestamp: u64
}

