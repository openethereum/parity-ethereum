use std::sync::Arc;

use rustc_hex::FromHex;

use ethcore::client::Client;
use ethcore::engines::signer::from_keypair;
use ethcore::miner::{Miner, MinerService};
use ethcore::spec::Spec;
use ethcore::test_helpers::generate_dummy_client_with_spec;
use ethcore::test_helpers::TestNotify;
use ethereum_types::U256;
use ethkey::{Generator, Random};
use hash::keccak;
use hbbft::NetworkInfo;
use types::transaction::{Action, SignedTransaction, Transaction};

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

pub struct HbbftTestData {
	pub client: Arc<Client>,
	pub notify: Arc<TestNotify>,
	pub miner: Arc<Miner>,
}

pub fn hbbft_client_setup(net_info: NetworkInfo<usize>) -> HbbftTestData {
	let client = hbbft_client();
	client.set_netinfo(net_info);

	let engine = client.engine();
	// Set the signer *before* registering the client with the engine.
	let signer = from_keypair(
		ethkey::KeyPair::from_secret(keccak("1").into()).expect("KeyPair generation must succeed"),
	);
	engine.set_signer(signer);
	engine.register_client(Arc::downgrade(&client) as _);

	// Register notify object for capturing consensus messages
	let notify = Arc::new(TestNotify::default());
	client.add_notify(notify.clone());

	// Get miner reference
	let miner = client.miner();

	HbbftTestData {
		client: client.clone(),
		notify,
		miner,
	}
}

pub fn create_transaction() -> SignedTransaction {
	let keypair = Random.generate().unwrap();
	Transaction {
		action: Action::Create,
		value: U256::zero(),
		data: "3331600055".from_hex().unwrap(),
		gas: U256::from(100_000),
		gas_price: U256::zero(),
		nonce: U256::zero(),
	}
	.sign(keypair.secret(), None)
}

pub fn inject_transaction(client: &Arc<Client>, miner: &Arc<Miner>) {
	let transaction = create_transaction();
	miner
		.import_own_transaction(client.as_ref(), transaction.into())
		.unwrap();
}
