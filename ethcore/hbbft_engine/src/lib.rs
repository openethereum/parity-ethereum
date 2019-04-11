extern crate ethcore;
extern crate ethereum_types;
extern crate inventory;
extern crate keccak_hash as hash;
extern crate parking_lot;
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
	use std::sync::Arc;

	use ethcore::client::Client;
	use ethcore::miner::Miner;
	use ethcore::spec::Spec;
	use ethcore::test_helpers::{generate_dummy_client_with_spec_and_accounts, TestNotify};

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
}
