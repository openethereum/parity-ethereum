extern crate ethcore;

#[cfg(test)]
mod tests {
	use ethcore::client::TestBlockChainClient;

	#[test]
	fn test_nodes_p2p() {
		let _node_0 = TestBlockChainClient::default();
		let _node_1 = TestBlockChainClient::default();
    }
}

