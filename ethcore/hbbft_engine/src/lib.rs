extern crate ethcore;
extern crate ethcore_transaction as transaction;
extern crate ethereum_types;
extern crate ethkey;
extern crate inventory;
extern crate keccak_hash as hash;
extern crate parking_lot;
extern crate rustc_hex;
extern crate serde_json;

extern crate common_types as types;

mod hbbft_engine;

use ethcore::engines::registry::EnginePlugin;

pub use hbbft_engine::HoneyBadgerBFT;

/// Registers the `HoneyBadgerBFT` engine. This must be called before parsing the chain spec.
pub fn init() {
	inventory::submit(EnginePlugin("HoneyBadgerBFT", HoneyBadgerBFT::new));
}

#[cfg(test)]
mod tests {
	use rustc_hex::FromHex;
	use std::sync::Arc;

	use ethereum_types::U256;
	use ethkey::{Generator, Random};

	use ethcore::client::{BlockId, BlockInfo, Client};
	use ethcore::miner::{Miner, MinerService};
	use ethcore::spec::Spec;
	use ethcore::test_helpers::{generate_dummy_client_with_spec_and_accounts, TestNotify};
	use transaction::{Action, Transaction};

	fn hbbft_spec() -> Spec {
		Spec::load(
			&::std::env::temp_dir(),
			include_bytes!("../res/honey_badger_bft.json") as &[u8],
		)
		.expect(concat!("Chain spec is invalid."))
	}

	fn hbbft_client() -> std::sync::Arc<ethcore::client::Client> {
		generate_dummy_client_with_spec_and_accounts(hbbft_spec, None)
	}

	fn hbbft_client_setup() -> (Arc<Client>, Arc<TestNotify>, Arc<Miner>) {
		// Create client
		let client = hbbft_client();

		// Register notify object for capturing consensus messages
		let notify = Arc::new(TestNotify::default());
		client.add_notify(notify.clone());

		// Get miner reference
		let miner = client.miner();

		(client, notify, miner)
	}

	#[test]
	fn test_client_miner_engine() {
		super::init();

		let (client, _, _) = hbbft_client_setup();

		// Get hbbft Engine reference and initialize it with a back-reference to the Client
		let engine = client.engine();
		engine.register_client(Arc::downgrade(&client) as _);
	}

	fn inject_transaction(client: &Arc<Client>, miner: &Arc<Miner>) {
		let keypair = Random.generate().unwrap();
		let transaction = Transaction {
			action: Action::Create,
			value: U256::zero(),
			data: "3331600055".from_hex().unwrap(),
			gas: U256::from(100_000),
			gas_price: U256::zero(),
			nonce: U256::zero(),
		}
		.sign(keypair.secret(), None);
		miner
			.import_own_transaction(client.as_ref(), transaction.into())
			.unwrap();
	}

	#[test]
	fn test_miner_transaction_injection() {
		super::init();

		let (client, _, miner) = hbbft_client_setup();

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
