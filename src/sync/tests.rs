use std::collections::HashMap;
use util::bytes::Bytes;
use util::hash::H256;

struct TestBlockChainClient {
	blocks: Vec<Bytes>,
 	hashes: HashMap<H256, usize>,
}


#[test]
fn full_sync() {
}
