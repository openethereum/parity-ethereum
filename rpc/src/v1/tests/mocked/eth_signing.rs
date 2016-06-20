// Copyright 2015, 2016 Ethcore (UK) Ltd.
// This file is part of Parity.

// Parity is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity.  If not, see <http://www.gnu.org/licenses/>.

use std::sync::Arc;
use jsonrpc_core::IoHandler;
use v1::impls::EthSigningQueueClient;
use v1::traits::EthSigning;
use v1::helpers::{ConfirmationsQueue, SigningQueue};
use v1::tests::helpers::TestMinerService;
use util::{Address, FixedHash};

struct EthSigningTester {
	pub queue: Arc<ConfirmationsQueue>,
	pub miner: Arc<TestMinerService>,
	pub io: IoHandler,
}

impl Default for EthSigningTester {
	fn default() -> Self {
		let queue = Arc::new(ConfirmationsQueue::default());
		let miner = Arc::new(TestMinerService::default());
		let io = IoHandler::new();
		io.add_delegate(EthSigningQueueClient::new(&queue, &miner).to_delegate());

		EthSigningTester {
			queue: queue,
			miner: miner,
			io: io,
		}
	}
}

fn eth_signing() -> EthSigningTester {
	EthSigningTester::default()
}


#[test]
fn should_add_transaction_to_queue() {
	// given
	let tester = eth_signing();
	let address = Address::random();
	assert_eq!(tester.queue.requests().len(), 0);

	// when
	let request = r#"{
		"jsonrpc": "2.0",
		"method": "eth_sendTransaction",
		"params": [{
			"from": ""#.to_owned() + format!("0x{:?}", address).as_ref() + r#"",
			"to": "0xd46e8dd67c5d32be8058bb8eb970870f07244567",
			"gas": "0x76c0",
			"gasPrice": "0x9184e72a000",
			"value": "0x9184e72a"
		}],
		"id": 1
	}"#;
	let response = r#"{"jsonrpc":"2.0","result":"0x0000000000000000000000000000000000000000000000000000000000000000","id":1}"#;


	// then
	assert_eq!(tester.io.handle_request(&request), Some(response.to_owned()));
	assert_eq!(tester.queue.requests().len(), 1);

}
