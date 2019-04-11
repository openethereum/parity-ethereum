extern crate ethcore;

pub mod hbbft_engine;

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
		let _engine = HoneyBadgerBFT::new(machine);

		// create test clients (which also creates the miner)
		let _node_0 = TestBlockChainClient::default();
		let _node_1 = TestBlockChainClient::default();
	}
}
