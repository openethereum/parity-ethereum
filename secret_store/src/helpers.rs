use ethcore::client::{Client, BlockChainClient, BlockId};
use ethereum_types::H256;

/// Number of confirmations required before request can be processed.
pub const REQUEST_CONFIRMATIONS_REQUIRED: u64 = 3;

/// Get hash of the last block with at least n confirmations.
pub fn get_confirmed_block_hash(client: &Client, confirmations: u64) -> Option<H256> {
	client.block_number(BlockId::Latest)
		.map(|b| b.saturating_sub(confirmations))
		.and_then(|b| client.block_hash(BlockId::Number(b)))
}
