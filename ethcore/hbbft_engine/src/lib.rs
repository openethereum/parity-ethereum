extern crate common_types as types;
extern crate ethcore;
extern crate ethcore_miner;
extern crate ethereum_types;
extern crate ethkey;
extern crate hbbft;
extern crate hbbft_testing;
extern crate inventory;
extern crate itertools;
extern crate keccak_hash as hash;
extern crate parking_lot;
extern crate rand;
extern crate rustc_hex;
#[macro_use(Serialize, Deserialize)]
extern crate serde;
extern crate rlp;
extern crate serde_json;
#[macro_use]
extern crate log;

#[cfg(test)]
extern crate proptest;

#[cfg(test)]
extern crate ethcore_accounts as accounts;

mod contribution;
mod hbbft_engine;

#[cfg(any(test, feature = "test-helpers"))]
pub mod test_helpers;

use ethcore::engines::registry::EnginePlugin;

pub use hbbft_engine::HoneyBadgerBFT;

/// Registers the `HoneyBadgerBFT` engine. This must be called before parsing the chain spec.
pub fn init() {
	inventory::submit(EnginePlugin("HoneyBadgerBFT", HoneyBadgerBFT::new));
}

#[cfg(test)]
mod tests {
	use crate::test_helpers::{hbbft_client_setup, inject_transaction, HbbftTestData};
	use ethcore::client::{BlockId, BlockInfo};
	use ethereum_types::H256;
	use hbbft::NetworkInfo;
	use hbbft_testing::proptest::{gen_seed, TestRng, TestRngSeed};
	use proptest::{prelude::ProptestConfig, proptest};
	use rand::SeedableRng;

	proptest! {
		#![proptest_config(ProptestConfig {
			cases: 1, .. ProptestConfig::default()
		})]

		#[test]
		#[allow(clippy::unnecessary_operation)]
		fn test_miner_transaction_injection(seed in gen_seed()) {
			do_test_miner_transaction_injection(seed)
		}

		#[test]
		#[allow(clippy::unnecessary_operation)]
		fn test_with_multiple_clients(seed in gen_seed()) {
			do_test_with_two_clients(seed)
		}
	}

	fn do_test_miner_transaction_injection(seed: TestRngSeed) {
		super::init();

		let mut rng = TestRng::from_seed(seed);
		let net_infos = NetworkInfo::generate_map(0..1usize, &mut rng)
			.expect("NetworkInfo generation is expected to always succeed");

		let net_info = net_infos
			.get(&0)
			.expect("A NetworkInfo must exist for node 0");

		let test_data = hbbft_client_setup(net_info.clone());

		// Verify that we actually start at block 0.
		assert_eq!(test_data.client.chain().best_block_number(), 0);

		// Inject a transaction, with instant sealing a block will be created right away.
		inject_transaction(&test_data.client, &test_data.miner);

		// Expect a new block to be created.
		assert_eq!(test_data.client.chain().best_block_number(), 1);

		// Expect one transaction in the block.
		let block = test_data
			.client
			.block(BlockId::Number(1))
			.expect("Block 1 must exist");
		assert_eq!(block.transactions_count(), 1);
	}

	fn do_test_with_two_clients(seed: TestRngSeed) {
		super::init();

		let mut rng = TestRng::from_seed(seed);
		let num_clients: usize = 2;
		let net_infos = NetworkInfo::generate_map(0..num_clients, &mut rng)
			.expect("NetworkInfo generation to always succeed");

		let nodes: Vec<_> = net_infos
			.into_iter()
			.map(|(_, netinfo)| hbbft_client_setup(netinfo))
			.collect();

		// Verify that we actually start at block 0.
		assert_eq!(nodes[0].client.chain().best_block_number(), 0);

		// Inject transactions to kick off block creation.
		for n in &nodes {
			inject_transaction(&n.client, &n.miner);
		}

		// Returns `true` if the node has not output all transactions yet.
		// If it has, and has advanced another epoch, it clears all messages for later epochs.
		let has_messages = |node: &HbbftTestData| !node.notify.targeted_messages.read().is_empty();

		// Rudimentary network simulation.
		while nodes.iter().any(has_messages) {
			for (from, n) in nodes.iter().enumerate() {
				for m in n.notify.targeted_messages.write().iter() {
					nodes[m.1]
						.client
						.engine()
						.handle_message(&m.0, from)
						.expect("message handling to succeed");
				}
				n.notify.targeted_messages.write().clear();
			}
		}

		// All nodes need to have produced a block.
		for n in &nodes {
			assert_eq!(n.client.chain().best_block_number(), 1);
		}

		// All nodes need to produce the same block with the same hash.
		let mut expected: Option<H256> = None;
		for n in &nodes {
			match expected {
				None => expected = Some(n.client.chain().best_block_hash()),
				Some(h) => assert_eq!(n.client.chain().best_block_hash(), h),
			}
		}
	}
}
