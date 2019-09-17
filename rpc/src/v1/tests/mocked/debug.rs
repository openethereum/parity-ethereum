// Copyright 2015-2019 Parity Technologies (UK) Ltd.
// This file is part of Parity Ethereum.

// Parity Ethereum is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity Ethereum is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity Ethereum.  If not, see <http://www.gnu.org/licenses/>.

use std::sync::Arc;

use ethcore::test_helpers::TestBlockChainClient;

use jsonrpc_core::IoHandler;
use v1::{Debug, DebugClient};

fn io() -> IoHandler {
	let client = Arc::new(TestBlockChainClient::new());

	let mut io = IoHandler::new();
	io.extend_with(DebugClient::new(client).to_delegate());
	io
}

#[test]
fn rpc_debug_get_bad_blocks() {
	let request = r#"{"jsonrpc": "2.0", "method": "debug_getBadBlocks", "params": [], "id": 1}"#;
	let response = "{\"jsonrpc\":\"2.0\",\"result\":[{\"author\":\"0x0000000000000000000000000000000000000000\",\"difficulty\":\"0x0\",\"extraData\":\"0x\",\"gasLimit\":\"0x0\",\"gasUsed\":\"0x0\",\"hash\":\"0x27bfb37e507ce90da141307204b1c6ba24194380613590ac50ca4b1d7198ff65\",\"logsBloom\":\"0x00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000\",\"miner\":\"0x0000000000000000000000000000000000000000\",\"number\":\"0x0\",\"parentHash\":\"0x0000000000000000000000000000000000000000000000000000000000000000\",\"reason\":\"Invalid block\",\"receiptsRoot\":\"0x56e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421\",\"rlp\":\"\\\"0x010203\\\"\",\"sealFields\":[],\"sha3Uncles\":\"0x1dcc4de8dec75d7aab85b567b6ccd41ad312451b948a7413f0a142fd40d49347\",\"size\":\"0x3\",\"stateRoot\":\"0x56e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421\",\"timestamp\":\"0x0\",\"totalDifficulty\":null,\"transactions\":[],\"transactionsRoot\":\"0x56e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421\",\"uncles\":[]}],\"id\":1}";
	assert_eq!(io().handle_request_sync(request), Some(response.to_owned()));
}
