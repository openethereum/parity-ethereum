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

use std::sync::{atomic, Arc};

use jsonrpc_core::{self as core, MetaIoHandler};
use jsonrpc_core::futures::{self, Stream, Future};
use jsonrpc_pubsub::Session;

use parity_runtime::Runtime;
use v1::{PubSub, PubSubClient, Metadata};

fn rpc() -> MetaIoHandler<Metadata, core::NoopMiddleware> {
	let mut io = MetaIoHandler::default();
	let called = atomic::AtomicBool::new(false);
	io.add_method("hello", move |_| {
		if !called.load(atomic::Ordering::SeqCst) {
			called.store(true, atomic::Ordering::SeqCst);
			Ok(core::Value::String("hello".into()))
		} else {
			Ok(core::Value::String("world".into()))
		}
	});
	io
}

#[test]
fn should_subscribe_to_a_method() {
	// given
	let el = Runtime::with_thread_count(1);
	let rpc = rpc();
	let pubsub = PubSubClient::new_test(rpc, el.executor()).to_delegate();

	let mut io = MetaIoHandler::default();
	io.extend_with(pubsub);

	let mut metadata = Metadata::default();
	let (sender, receiver) = futures::sync::mpsc::channel(8);
	metadata.session = Some(Arc::new(Session::new(sender)));

	// Subscribe
	let request = r#"{"jsonrpc": "2.0", "method": "parity_subscribe", "params": ["hello", []], "id": 1}"#;
	let response = r#"{"jsonrpc":"2.0","result":"0x43ca64edf03768e1","id":1}"#;
	assert_eq!(io.handle_request_sync(request, metadata.clone()), Some(response.to_owned()));

	// Check notifications
	let (res, receiver) = receiver.into_future().wait().unwrap();
	let response =
		r#"{"jsonrpc":"2.0","method":"parity_subscription","params":{"result":"hello","subscription":"0x43ca64edf03768e1"}}"#;
	assert_eq!(res, Some(response.into()));

	let (res, receiver) = receiver.into_future().wait().unwrap();
	let response =
		r#"{"jsonrpc":"2.0","method":"parity_subscription","params":{"result":"world","subscription":"0x43ca64edf03768e1"}}"#;
	assert_eq!(res, Some(response.into()));

	// And unsubscribe
	let request = r#"{"jsonrpc": "2.0", "method": "parity_unsubscribe", "params": ["0x43ca64edf03768e1"], "id": 1}"#;
	let response = r#"{"jsonrpc":"2.0","result":true,"id":1}"#;
	assert_eq!(io.handle_request_sync(request, metadata), Some(response.to_owned()));

	let (res, _receiver) = receiver.into_future().wait().unwrap();
	assert_eq!(res, None);
}
