extern crate common_types as types;
extern crate ethcore;
#[cfg(test)]
extern crate ethcore_accounts as accounts;
extern crate ethcore_miner;
extern crate ethereum_types;
extern crate ethkey;
extern crate hbbft;
extern crate inventory;
extern crate keccak_hash as hash;
extern crate parking_lot;
extern crate rand;
extern crate rustc_hex;
extern crate serde_json;

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
	use rand;

	#[test]
	fn test_miner_transaction_injection() {
		super::init();

		// Generate a new set of cryptographic keys for threshold cryptography.
		let mut rng = rand::thread_rng();
		let size = 1;
		let _net_infos = NetworkInfo::generate_map(0..size as u16, &mut rng)
			.expect("NetworkInfo generation is expected to always succeed");

		let keypair = ethkey::KeyPair::from_secret(keccak("1").into())
			.expect("KeyPair generation must succeed");

		let (client, _, miner) = hbbft_client_setup(from_keypair(keypair));

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
