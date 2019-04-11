extern crate ethcore;
extern crate hbbft_engine;
extern crate inventory;

use ethcore::engines::registry::EnginePlugin;
use ethcore::spec::Spec;
use hbbft_engine::HoneyBadgerBFT;

// TODO: This shouldn't be necessary, since `submit!` is also called in `lib.rs`.
inventory::submit!(EnginePlugin("HoneyBadgerBFT", HoneyBadgerBFT::new));

#[test]
fn test_nodes_p2p() {
	// Load the chain specification.
	let spec = Spec::load(
		&::std::env::temp_dir(),
		include_bytes!("../res/honey_badger_bft.json") as &[u8],
	)
	.expect(concat!("Chain spec is invalid."));

	// The engine was created from the spec.
	let _engine = spec.engine;
}
