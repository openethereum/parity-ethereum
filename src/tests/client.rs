use ethereum;
use client::{BlockChainClient,Client};
use std::env;
use pod_state::*;
use super::test_common::*;


#[test]
fn test_client_is_created() {

	let mut spec = ethereum::new_frontier_like_test();
	spec.set_genesis_state(PodState::from_json(test.find("pre").unwrap()));
	spec.overwrite_genesis(test.find("genesisBlockHeader").unwrap());

	let mut dir = env::temp_dir();
	dir.push(H32::random().hex());

	let client_result = Client::new(spec, &dir, IOChannel::disconnected());

	assert!(client_result.is_ok());
}