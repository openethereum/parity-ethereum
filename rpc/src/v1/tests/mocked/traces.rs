// Copyright 2015-2017 Parity Technologies (UK) Ltd.
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

use ethcore::executed::{Executed, CallError};
use ethcore::trace::trace::{Action, Res, Call};
use ethcore::trace::LocalizedTrace;
use ethcore::client::TestBlockChainClient;

use vm::CallType;

use jsonrpc_core::IoHandler;
use v1::tests::helpers::{TestMinerService};
use v1::{Metadata, Traces, TracesClient};

struct Tester {
	client: Arc<TestBlockChainClient>,
	_miner: Arc<TestMinerService>,
	io: IoHandler<Metadata>,
}

fn io() -> Tester {
	let client = Arc::new(TestBlockChainClient::new());
	*client.traces.write() = Some(vec![LocalizedTrace {
		action: Action::Call(Call {
			from: 0xf.into(),
			to: 0x10.into(),
			value: 0x1.into(),
			gas: 0x100.into(),
			input: vec![1, 2, 3],
			call_type: CallType::Call,
		}),
		result: Res::None,
		subtraces: 0,
		trace_address: vec![0],
		transaction_number: 0,
		transaction_hash: 5.into(),
		block_number: 10,
		block_hash: 10.into(),
	}]);
	*client.execution_result.write() = Some(Ok(Executed {
		exception: None,
		gas: 20_000.into(),
		gas_used: 10_000.into(),
		refunded: 0.into(),
		cumulative_gas_used: 10_000.into(),
		logs: vec![],
		contracts_created: vec![],
		output: vec![1, 2, 3],
		trace: vec![],
		vm_trace: None,
		state_diff: None,
	}));
	let miner = Arc::new(TestMinerService::default());
	let traces = TracesClient::new(&client, &miner);
	let mut io = IoHandler::default();
	io.extend_with(traces.to_delegate());

	Tester {
		client: client,
		_miner: miner,
		io: io,
	}
}

#[test]
fn rpc_trace_filter() {
	let tester = io();

	let request = r#"{"jsonrpc":"2.0","method":"trace_filter","params": [{}],"id":1}"#;
	let response = r#"{"jsonrpc":"2.0","result":[{"action":{"callType":"call","from":"0x000000000000000000000000000000000000000f","gas":"0x100","input":"0x010203","to":"0x0000000000000000000000000000000000000010","value":"0x1"},"blockHash":"0x000000000000000000000000000000000000000000000000000000000000000a","blockNumber":10,"result":null,"subtraces":0,"traceAddress":[0],"transactionHash":"0x0000000000000000000000000000000000000000000000000000000000000005","transactionPosition":0,"type":"call"}],"id":1}"#;

	assert_eq!(tester.io.handle_request_sync(request), Some(response.to_owned()));
}

#[test]
fn rpc_trace_filter_missing_trace() {
	let tester = io();
	*tester.client.traces.write() = None;

	let request = r#"{"jsonrpc":"2.0","method":"trace_filter","params": [{}],"id":1}"#;
	let response = r#"{"jsonrpc":"2.0","result":null,"id":1}"#;

	assert_eq!(tester.io.handle_request_sync(request), Some(response.to_owned()));
}

#[test]
fn rpc_trace_block() {
	let tester = io();

	let request = r#"{"jsonrpc":"2.0","method":"trace_block","params": ["0x10"],"id":1}"#;
	let response = r#"{"jsonrpc":"2.0","result":[{"action":{"callType":"call","from":"0x000000000000000000000000000000000000000f","gas":"0x100","input":"0x010203","to":"0x0000000000000000000000000000000000000010","value":"0x1"},"blockHash":"0x000000000000000000000000000000000000000000000000000000000000000a","blockNumber":10,"result":null,"subtraces":0,"traceAddress":[0],"transactionHash":"0x0000000000000000000000000000000000000000000000000000000000000005","transactionPosition":0,"type":"call"}],"id":1}"#;

	assert_eq!(tester.io.handle_request_sync(request), Some(response.to_owned()));
}

#[test]
fn rpc_trace_block_missing_traces() {
	let tester = io();
	*tester.client.traces.write() = None;

	let request = r#"{"jsonrpc":"2.0","method":"trace_block","params": ["0x10"],"id":1}"#;
	let response = r#"{"jsonrpc":"2.0","result":null,"id":1}"#;

	assert_eq!(tester.io.handle_request_sync(request), Some(response.to_owned()));
}

#[test]
fn rpc_trace_transaction() {
	let tester = io();

	let request = r#"{"jsonrpc":"2.0","method":"trace_transaction","params":["0x0000000000000000000000000000000000000000000000000000000000000005"],"id":1}"#;
	let response = r#"{"jsonrpc":"2.0","result":[{"action":{"callType":"call","from":"0x000000000000000000000000000000000000000f","gas":"0x100","input":"0x010203","to":"0x0000000000000000000000000000000000000010","value":"0x1"},"blockHash":"0x000000000000000000000000000000000000000000000000000000000000000a","blockNumber":10,"result":null,"subtraces":0,"traceAddress":[0],"transactionHash":"0x0000000000000000000000000000000000000000000000000000000000000005","transactionPosition":0,"type":"call"}],"id":1}"#;

	assert_eq!(tester.io.handle_request_sync(request), Some(response.to_owned()));
}

#[test]
fn rpc_trace_transaction_missing_trace() {
	let tester = io();
	*tester.client.traces.write() = None;

	let request = r#"{"jsonrpc":"2.0","method":"trace_transaction","params":["0x0000000000000000000000000000000000000000000000000000000000000005"],"id":1}"#;
	let response = r#"{"jsonrpc":"2.0","result":null,"id":1}"#;

	assert_eq!(tester.io.handle_request_sync(request), Some(response.to_owned()));
}

#[test]
fn rpc_trace_get() {
	let tester = io();

	let request = r#"{"jsonrpc":"2.0","method":"trace_get","params":["0x0000000000000000000000000000000000000000000000000000000000000005", ["0","0","0"]],"id":1}"#;
	let response = r#"{"jsonrpc":"2.0","result":{"action":{"callType":"call","from":"0x000000000000000000000000000000000000000f","gas":"0x100","input":"0x010203","to":"0x0000000000000000000000000000000000000010","value":"0x1"},"blockHash":"0x000000000000000000000000000000000000000000000000000000000000000a","blockNumber":10,"result":null,"subtraces":0,"traceAddress":[0],"transactionHash":"0x0000000000000000000000000000000000000000000000000000000000000005","transactionPosition":0,"type":"call"},"id":1}"#;

	assert_eq!(tester.io.handle_request_sync(request), Some(response.to_owned()));
}

#[test]
fn rpc_trace_get_missing_trace() {
	let tester = io();
	*tester.client.traces.write() = None;

	let request = r#"{"jsonrpc":"2.0","method":"trace_get","params":["0x0000000000000000000000000000000000000000000000000000000000000005", ["0","0","0"]],"id":1}"#;
	let response = r#"{"jsonrpc":"2.0","result":null,"id":1}"#;
	assert_eq!(tester.io.handle_request_sync(request), Some(response.to_owned()));
}

#[test]
fn rpc_trace_call() {
	let tester = io();

	let request = r#"{"jsonrpc":"2.0","method":"trace_call","params":[{}, ["stateDiff", "vmTrace", "trace"]],"id":1}"#;
	let response = r#"{"jsonrpc":"2.0","result":{"output":"0x010203","stateDiff":null,"trace":[],"vmTrace":null},"id":1}"#;

	assert_eq!(tester.io.handle_request_sync(request), Some(response.to_owned()));
}

#[test]
fn rpc_trace_multi_call() {
	let tester = io();

	let request = r#"{"jsonrpc":"2.0","method":"trace_callMany","params":[[[{}, ["stateDiff", "vmTrace", "trace"]]]],"id":1}"#;
	let response = r#"{"jsonrpc":"2.0","result":[{"output":"0x010203","stateDiff":null,"trace":[],"vmTrace":null}],"id":1}"#;

	assert_eq!(tester.io.handle_request_sync(request), Some(response.to_owned()));
}

#[test]
fn rpc_trace_call_state_pruned() {
	let tester = io();
	*tester.client.execution_result.write() = Some(Err(CallError::StatePruned));

	let request = r#"{"jsonrpc":"2.0","method":"trace_call","params":[{}, ["stateDiff", "vmTrace", "trace"]],"id":1}"#;
	let response = r#"{"jsonrpc":"2.0","error":{"code":-32000,"message":"This request is not supported because your node is running with state pruning. Run with --pruning=archive."},"id":1}"#;

	assert_eq!(tester.io.handle_request_sync(request), Some(response.to_owned()));
}

#[test]
fn rpc_trace_raw_transaction() {
	let tester = io();

	let request = r#"{"jsonrpc":"2.0","method":"trace_rawTransaction","params":["0xf869018609184e72a0008276c094d46e8dd67c5d32be8058bb8eb970870f07244567849184e72a801ba0617f39c1a107b63302449c476d96a6cb17a5842fc98ff0c5bcf4d5c4d8166b95a009fdb6097c6196b9bbafc3a59f02f38d91baeef23d0c60a8e4f23c7714cea3a9", ["stateDiff", "vmTrace", "trace"]],"id":1}"#;
	let response = r#"{"jsonrpc":"2.0","result":{"output":"0x010203","stateDiff":null,"trace":[],"vmTrace":null},"id":1}"#;

	assert_eq!(tester.io.handle_request_sync(request), Some(response.to_owned()));
}

#[test]
fn rpc_trace_raw_transaction_state_pruned() {
	let tester = io();
	*tester.client.execution_result.write() = Some(Err(CallError::StatePruned));

	let request = r#"{"jsonrpc":"2.0","method":"trace_rawTransaction","params":["0xf869018609184e72a0008276c094d46e8dd67c5d32be8058bb8eb970870f07244567849184e72a801ba0617f39c1a107b63302449c476d96a6cb17a5842fc98ff0c5bcf4d5c4d8166b95a009fdb6097c6196b9bbafc3a59f02f38d91baeef23d0c60a8e4f23c7714cea3a9", ["stateDiff", "vmTrace", "trace"]],"id":1}"#;
	let response = r#"{"jsonrpc":"2.0","error":{"code":-32000,"message":"This request is not supported because your node is running with state pruning. Run with --pruning=archive."},"id":1}"#;

	assert_eq!(tester.io.handle_request_sync(request), Some(response.to_owned()));
}

#[test]
fn rpc_trace_replay_transaction() {
	let tester = io();

	let request = r#"{"jsonrpc":"2.0","method":"trace_replayTransaction","params":["0x0000000000000000000000000000000000000000000000000000000000000005", ["trace", "stateDiff", "vmTrace"]],"id":1}"#;
	let response = r#"{"jsonrpc":"2.0","result":{"output":"0x010203","stateDiff":null,"trace":[],"vmTrace":null},"id":1}"#;

	assert_eq!(tester.io.handle_request_sync(request), Some(response.to_owned()));
}

#[test]
fn rpc_trace_replay_transaction_state_pruned() {
	let tester = io();
	*tester.client.execution_result.write() = Some(Err(CallError::StatePruned));

	let request = r#"{"jsonrpc":"2.0","method":"trace_replayTransaction","params":["0x0000000000000000000000000000000000000000000000000000000000000005", ["trace", "stateDiff", "vmTrace"]],"id":1}"#;
	let response = r#"{"jsonrpc":"2.0","error":{"code":-32000,"message":"This request is not supported because your node is running with state pruning. Run with --pruning=archive."},"id":1}"#;

	assert_eq!(tester.io.handle_request_sync(request), Some(response.to_owned()));
}
