extern crate common_types as types;
extern crate ethcore;
extern crate ethcore_miner;
extern crate ethereum_types;
extern crate ethkey;
extern crate hbbft;
extern crate hbbft_testing;
extern crate inventory;
extern crate keccak_hash as hash;
extern crate parking_lot;
extern crate rustc_hex;
extern crate serde;
extern crate serde_json;
#[macro_use(Serialize, Deserialize)]
extern crate serde_derive;
extern crate rlp;

#[cfg(test)]
extern crate proptest;

#[cfg(test)]
extern crate rand;

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
	use crate::test_helpers::{hbbft_client_setup, inject_transaction};
	use ethcore::client::{BlockId, BlockInfo};
	use ethcore::engines::signer::from_keypair;
	use hash::keccak;
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
	}

	fn do_test_miner_transaction_injection(seed: TestRngSeed) {
		super::init();

		let keypair = ethkey::KeyPair::from_secret(keccak("1").into())
			.expect("KeyPair generation must succeed");

		let (client, _, miner) = hbbft_client_setup(from_keypair(keypair));

		// Generate a new set of cryptographic keys for threshold cryptography.
		let mut rng = TestRng::from_seed(seed);
		let size = 1;
		let net_infos = NetworkInfo::generate_map(0..size as usize, &mut rng)
			.expect("NetworkInfo generation is expected to always succeed");

		let net_info = net_infos
			.get(&0)
			.expect("A NetworkInfo must exist for node 0");
		client.set_netinfo(net_info.clone());

		// Verify that we actually start at block 0.
		assert_eq!(client.chain().best_block_number(), 0);

		// Inject a transaction, with instant sealing a block will be created right away.
		inject_transaction(&client, &miner);

		// Expect a new block to be created.
		assert_eq!(client.chain().best_block_number(), 1);

		// Expect one transaction in the block.
		let block = client
			.block(BlockId::Number(1))
			.expect("Block 1 must exist");
		assert_eq!(block.transactions_count(), 1);
	}
}
