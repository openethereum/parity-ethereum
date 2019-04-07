extern crate ethcore;
extern crate hbbft_engine;
extern crate inventory;

use ethcore::engines::registry::EnginePlugin;
use ethcore::spec::Spec;
use hbbft_engine::HoneyBadgerBFT;

#[test]
fn test_nodes_p2p() {
	hbbft_engine::init();

	// Load the chain specification.
	let spec = Spec::load(
		&::std::env::temp_dir(),
		include_bytes!("../res/honey_badger_bft.json") as &[u8],
	)
	.expect(concat!("Chain spec is invalid."));

	// The engine was created from the spec.
	let _engine = spec.engine;
}
