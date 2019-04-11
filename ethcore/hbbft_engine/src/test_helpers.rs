use std::sync::Arc;

use rustc_hex::FromHex;

use ethcore::client::Client;
use ethcore::miner::{Miner, MinerService};
use ethcore::spec::Spec;
use ethcore::test_helpers::generate_dummy_client_with_spec;
use ethcore::test_helpers::TestNotify;
use ethereum_types::U256;
use ethkey::{Generator, Random};
use types::transaction::{Action, Transaction};

pub fn hbbft_spec() -> Spec {
	Spec::load(
		&::std::env::temp_dir(),
		include_bytes!("../res/honey_badger_bft.json") as &[u8],
	)
	.expect(concat!("Chain spec is invalid."))
}

pub fn hbbft_client() -> std::sync::Arc<ethcore::client::Client> {
	generate_dummy_client_with_spec(hbbft_spec)
}

pub fn hbbft_client_setup() -> (Arc<Client>, Arc<TestNotify>, Arc<Miner>) {
	// Create client
	let client = hbbft_client();

	// Get hbbft Engine reference and initialize it with a back-reference to the Client
	{
		let engine = client.engine();
		engine.register_client(Arc::downgrade(&client) as _);
	}

	// Register notify object for capturing consensus messages
	let notify = Arc::new(TestNotify::default());
	client.add_notify(notify.clone());

	// Get miner reference
	let miner = client.miner();

	(client, notify, miner)
}

pub fn inject_transaction(client: &Arc<Client>, miner: &Arc<Miner>) {
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
