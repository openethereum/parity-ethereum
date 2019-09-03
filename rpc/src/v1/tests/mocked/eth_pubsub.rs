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

use jsonrpc_core::MetaIoHandler;
use jsonrpc_core::futures::{self, Stream, Future, sync::mpsc};
use jsonrpc_pubsub::Session;

use std::time::Duration;

use v1::{EthPubSub, EthPubSubClient, Metadata};
use ethcore::client::{TestBlockChainClient, EachBlockWith};
use parity_runtime::Runtime;
use ethereum_types::{Address, H256};
use client_traits::{BlockInfo, ChainNotify};
use types::{
	chain_notify::{NewBlocks, ChainRoute, ChainRouteType},
	log_entry::{LocalizedLogEntry, LogEntry},
	ids::BlockId,
};


const DURATION_ZERO: Duration = Duration::from_millis(0);

#[test]
fn should_subscribe_to_new_heads() {
	// given
	let el = Runtime::with_thread_count(1);
	let mut client = TestBlockChainClient::new();
	// Insert some blocks
	client.add_blocks(3, EachBlockWith::Nothing);
	let h3 = client.block_hash_delta_minus(1);
	let h2 = client.block_hash_delta_minus(2);
	let h1 = client.block_hash_delta_minus(3);

	let (_, pool_receiver) = mpsc::unbounded();

	let pubsub = EthPubSubClient::new(Arc::new(client), el.executor(), pool_receiver);
	let handler = pubsub.handler().upgrade().unwrap();
	let pubsub = pubsub.to_delegate();

	let mut io = MetaIoHandler::default();
	io.extend_with(pubsub);

	let mut metadata = Metadata::default();
	let (sender, receiver) = futures::sync::mpsc::channel(8);
	metadata.session = Some(Arc::new(Session::new(sender)));

	// Subscribe
	let request = r#"{"jsonrpc": "2.0", "method": "eth_subscribe", "params": ["newHeads"], "id": 1}"#;
	let response = r#"{"jsonrpc":"2.0","result":"0x43ca64edf03768e1","id":1}"#;
	assert_eq!(io.handle_request_sync(request, metadata.clone()), Some(response.to_owned()));

	// Check notifications
	handler.new_blocks(NewBlocks::new(vec![], vec![], ChainRoute::new(vec![(h1, ChainRouteType::Enacted)]), vec![], vec![], DURATION_ZERO, true));
	let (res, receiver) = receiver.into_future().wait().unwrap();
	let response = r#"{"jsonrpc":"2.0","method":"eth_subscription","params":{"result":{"author":"0x0000000000000000000000000000000000000000","difficulty":"0x1","extraData":"0x","gasLimit":"0xf4240","gasUsed":"0x0","hash":"0x3457d2fa2e3dd33c78ac681cf542e429becf718859053448748383af67e23218","logsBloom":"0x00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000","miner":"0x0000000000000000000000000000000000000000","number":"0x1","parentHash":"0x0cd786a2425d16f152c658316c423e6ce1181e15c3295826d7c9904cba9ce303","receiptsRoot":"0x56e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421","sealFields":[],"sha3Uncles":"0x1dcc4de8dec75d7aab85b567b6ccd41ad312451b948a7413f0a142fd40d49347","size":"0x1c9","stateRoot":"0x56e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421","timestamp":"0x0","transactionsRoot":"0x56e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421"},"subscription":"0x43ca64edf03768e1"}}"#;
	assert_eq!(res, Some(response.into()));

	// Notify about two blocks
	handler.new_blocks(NewBlocks::new(vec![], vec![], ChainRoute::new(vec![(h2, ChainRouteType::Enacted), (h3, ChainRouteType::Enacted)]), vec![], vec![], DURATION_ZERO, true));

	// Receive both
	let (res, receiver) = receiver.into_future().wait().unwrap();
	let response = r#"{"jsonrpc":"2.0","method":"eth_subscription","params":{"result":{"author":"0x0000000000000000000000000000000000000000","difficulty":"0x2","extraData":"0x","gasLimit":"0xf4240","gasUsed":"0x0","hash":"0x44e5ecf454ea99af9d8a8f2ca0daba96964c90de05db7a78f59b84ae9e749706","logsBloom":"0x00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000","miner":"0x0000000000000000000000000000000000000000","number":"0x2","parentHash":"0x3457d2fa2e3dd33c78ac681cf542e429becf718859053448748383af67e23218","receiptsRoot":"0x56e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421","sealFields":[],"sha3Uncles":"0x1dcc4de8dec75d7aab85b567b6ccd41ad312451b948a7413f0a142fd40d49347","size":"0x1c9","stateRoot":"0x56e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421","timestamp":"0x0","transactionsRoot":"0x56e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421"},"subscription":"0x43ca64edf03768e1"}}"#;
	assert_eq!(res, Some(response.into()));
	let (res, receiver) = receiver.into_future().wait().unwrap();
	let response = r#"{"jsonrpc":"2.0","method":"eth_subscription","params":{"result":{"author":"0x0000000000000000000000000000000000000000","difficulty":"0x3","extraData":"0x","gasLimit":"0xf4240","gasUsed":"0x0","hash":"0xdf04a98bb0c6fa8441bd429822f65a46d0cb553f6bcef602b973e65c81497f8e","logsBloom":"0x00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000","miner":"0x0000000000000000000000000000000000000000","number":"0x3","parentHash":"0x44e5ecf454ea99af9d8a8f2ca0daba96964c90de05db7a78f59b84ae9e749706","receiptsRoot":"0x56e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421","sealFields":[],"sha3Uncles":"0x1dcc4de8dec75d7aab85b567b6ccd41ad312451b948a7413f0a142fd40d49347","size":"0x1c9","stateRoot":"0x56e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421","timestamp":"0x0","transactionsRoot":"0x56e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421"},"subscription":"0x43ca64edf03768e1"}}"#;
	assert_eq!(res, Some(response.into()));

	// And unsubscribe
	let request = r#"{"jsonrpc": "2.0", "method": "eth_unsubscribe", "params": ["0x43ca64edf03768e1"], "id": 1}"#;
	let response = r#"{"jsonrpc":"2.0","result":true,"id":1}"#;
	assert_eq!(io.handle_request_sync(request, metadata), Some(response.to_owned()));

	let (res, _receiver) = receiver.into_future().wait().unwrap();
	assert_eq!(res, None);
}

#[test]
fn should_subscribe_to_logs() {
	// given
	let el = Runtime::with_thread_count(1);
	let mut client = TestBlockChainClient::new();
	// Insert some blocks
	client.add_blocks(1, EachBlockWith::Transaction);
	let h1 = client.block_hash_delta_minus(1);
	let block = client.block(BlockId::Hash(h1)).unwrap();
	let tx_hash = block.transactions()[0].hash();
	client.set_logs(vec![
		LocalizedLogEntry {
			entry: LogEntry {
				address: Address::from_low_u64_be(5),
				topics: vec![H256::from_low_u64_be(1), H256::from_low_u64_be(2), H256::from_low_u64_be(0), H256::from_low_u64_be(0)],
				data: vec![],
			},
			block_hash: h1,
			block_number: block.header().number(),
			transaction_hash: tx_hash,
			transaction_index: 0,
			log_index: 0,
			transaction_log_index: 0,
		}
	]);

	let (_, pool_receiver) = mpsc::unbounded();

	let pubsub = EthPubSubClient::new(Arc::new(client), el.executor(), pool_receiver);
	let handler = pubsub.handler().upgrade().unwrap();
	let pubsub = pubsub.to_delegate();

	let mut io = MetaIoHandler::default();
	io.extend_with(pubsub);

	let mut metadata = Metadata::default();
	let (sender, receiver) = futures::sync::mpsc::channel(8);
	metadata.session = Some(Arc::new(Session::new(sender)));

	// Subscribe
	let request = r#"{"jsonrpc": "2.0", "method": "eth_subscribe", "params": ["logs", {}], "id": 1}"#;
	let response = r#"{"jsonrpc":"2.0","result":"0x43ca64edf03768e1","id":1}"#;
	assert_eq!(io.handle_request_sync(request, metadata.clone()), Some(response.to_owned()));

	// Check notifications (enacted)
	handler.new_blocks(NewBlocks::new(vec![], vec![], ChainRoute::new(vec![(h1, ChainRouteType::Enacted)]), vec![], vec![], DURATION_ZERO, false));
	let (res, receiver) = receiver.into_future().wait().unwrap();
	let response = r#"{"jsonrpc":"2.0","method":"eth_subscription","params":{"result":{"address":"0x0000000000000000000000000000000000000005","blockHash":"0x3457d2fa2e3dd33c78ac681cf542e429becf718859053448748383af67e23218","blockNumber":"0x1","data":"0x","logIndex":"0x0","removed":false,"topics":["0x0000000000000000000000000000000000000000000000000000000000000001","0x0000000000000000000000000000000000000000000000000000000000000002","0x0000000000000000000000000000000000000000000000000000000000000000","0x0000000000000000000000000000000000000000000000000000000000000000"],"transactionHash":""#.to_owned()
		+ &format!("0x{:x}", tx_hash)
		+ r#"","transactionIndex":"0x0","transactionLogIndex":"0x0","type":"mined"},"subscription":"0x43ca64edf03768e1"}}"#;
	assert_eq!(res, Some(response.into()));

	// Check notifications (retracted)
	handler.new_blocks(NewBlocks::new(vec![], vec![], ChainRoute::new(vec![(h1, ChainRouteType::Retracted)]), vec![], vec![], DURATION_ZERO, false));
	let (res, receiver) = receiver.into_future().wait().unwrap();
	let response = r#"{"jsonrpc":"2.0","method":"eth_subscription","params":{"result":{"address":"0x0000000000000000000000000000000000000005","blockHash":"0x3457d2fa2e3dd33c78ac681cf542e429becf718859053448748383af67e23218","blockNumber":"0x1","data":"0x","logIndex":"0x0","removed":true,"topics":["0x0000000000000000000000000000000000000000000000000000000000000001","0x0000000000000000000000000000000000000000000000000000000000000002","0x0000000000000000000000000000000000000000000000000000000000000000","0x0000000000000000000000000000000000000000000000000000000000000000"],"transactionHash":""#.to_owned()
		+ &format!("0x{:x}", tx_hash)
		+ r#"","transactionIndex":"0x0","transactionLogIndex":"0x0","type":"removed"},"subscription":"0x43ca64edf03768e1"}}"#;
	assert_eq!(res, Some(response.into()));

	// And unsubscribe
	let request = r#"{"jsonrpc": "2.0", "method": "eth_unsubscribe", "params": ["0x43ca64edf03768e1"], "id": 1}"#;
	let response = r#"{"jsonrpc":"2.0","result":true,"id":1}"#;
	assert_eq!(io.handle_request_sync(request, metadata), Some(response.to_owned()));

	let (res, _receiver) = receiver.into_future().wait().unwrap();
	assert_eq!(res, None);
}

#[test]
fn should_subscribe_to_pending_transactions() {
	// given
	let el = Runtime::with_thread_count(1);
	let client = TestBlockChainClient::new();

	let (pool_sender, pool_receiver) = mpsc::unbounded();

	let pubsub = EthPubSubClient::new(Arc::new(client), el.executor(), pool_receiver);
	let pubsub = pubsub.to_delegate();

	let mut io = MetaIoHandler::default();
	io.extend_with(pubsub);

	let mut metadata = Metadata::default();
	let (sender, receiver) = futures::sync::mpsc::channel(8);
	metadata.session = Some(Arc::new(Session::new(sender)));

	// Fail if params are provided
	let request = r#"{"jsonrpc": "2.0", "method": "eth_subscribe", "params": ["newPendingTransactions", {}], "id": 1}"#;
	let response = r#"{"jsonrpc":"2.0","error":{"code":-32602,"message":"Couldn't parse parameters: newPendingTransactions","data":"\"Expected no parameters.\""},"id":1}"#;
	assert_eq!(io.handle_request_sync(request, metadata.clone()), Some(response.to_owned()));

	// Subscribe
	let request = r#"{"jsonrpc": "2.0", "method": "eth_subscribe", "params": ["newPendingTransactions"], "id": 1}"#;
	let response = r#"{"jsonrpc":"2.0","result":"0x43ca64edf03768e1","id":1}"#;
	assert_eq!(io.handle_request_sync(request, metadata.clone()), Some(response.to_owned()));

	// Send new transactions
	pool_sender.unbounded_send(Arc::new(vec![H256::from_low_u64_be(5), H256::from_low_u64_be(7)])).unwrap();

	let (res, receiver) = receiver.into_future().wait().unwrap();
	let response = r#"{"jsonrpc":"2.0","method":"eth_subscription","params":{"result":"0x0000000000000000000000000000000000000000000000000000000000000005","subscription":"0x43ca64edf03768e1"}}"#;
	assert_eq!(res, Some(response.into()));

	let (res, receiver) = receiver.into_future().wait().unwrap();
	let response = r#"{"jsonrpc":"2.0","method":"eth_subscription","params":{"result":"0x0000000000000000000000000000000000000000000000000000000000000007","subscription":"0x43ca64edf03768e1"}}"#;
	assert_eq!(res, Some(response.into()));

	// And unsubscribe
	let request = r#"{"jsonrpc": "2.0", "method": "eth_unsubscribe", "params": ["0x43ca64edf03768e1"], "id": 1}"#;
	let response = r#"{"jsonrpc":"2.0","result":true,"id":1}"#;
	assert_eq!(io.handle_request_sync(request, metadata), Some(response.to_owned()));

	let (res, _receiver) = receiver.into_future().wait().unwrap();
	assert_eq!(res, None);
}

#[test]
fn eth_subscribe_syncing() {
	// given
	let el = Runtime::with_thread_count(1);
	let client = TestBlockChainClient::new();
	let (_, pool_receiver) = mpsc::unbounded();
	let pubsub = EthPubSubClient::new(Arc::new(client), el.executor(), pool_receiver);
	let pubsub = pubsub.to_delegate();

	let mut io = MetaIoHandler::default();
	io.extend_with(pubsub);

	let mut metadata = Metadata::default();
	let (sender, _receiver) = futures::sync::mpsc::channel(8);
	metadata.session = Some(Arc::new(Session::new(sender)));

	// Subscribe
	let response = r#"{"jsonrpc":"2.0","result":"0x43ca64edf03768e1","id":1}"#;
	let request = r#"{"jsonrpc": "2.0", "method": "eth_subscribe", "params": ["syncing"], "id": 1}"#;
	assert_eq!(io.handle_request_sync(request, metadata.clone()), Some(response.to_owned()));
}
