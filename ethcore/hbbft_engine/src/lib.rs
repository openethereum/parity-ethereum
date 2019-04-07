extern crate ethcore;
extern crate inventory;
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
	use ethcore::client::TestBlockChainClient;
	use ethcore::machine::EthereumMachine;
	use ethcore::spec::CommonParams;
	use hbbft_engine::HoneyBadgerBFT;

	#[test]
	fn test_nodes_p2p() {
		// create machine
		let c_params = CommonParams::default();
		let machine = EthereumMachine::regular(c_params, Default::default());

		// create engine
		let _engine = HoneyBadgerBFT::new(&serde_json::Value::Null, machine);

		// create test clients (which also creates the miner)
		let _node_0 = TestBlockChainClient::default();
		let _node_1 = TestBlockChainClient::default();
	}
}
