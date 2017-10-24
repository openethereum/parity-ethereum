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

//! Client RPC tests

use nanoipc;
use std::sync::Arc;
use std::sync::atomic::{Ordering, AtomicBool};
use client::{Client, BlockChainClient, ClientConfig, BlockId};
use client::remote::RemoteClient;
use tests::helpers::*;
use devtools::*;
use miner::Miner;
use crossbeam;
use io::IoChannel;
use util::kvdb::DatabaseConfig;

pub fn run_test_worker(scope: &crossbeam::Scope, stop: Arc<AtomicBool>, socket_path: &str) {
	let socket_path = socket_path.to_owned();
	scope.spawn(move || {
		let temp = RandomTempPath::create_dir();
		let spec = get_test_spec();
		let db_config = DatabaseConfig::with_columns(::db::NUM_COLUMNS);

		let client = Client::new(
			ClientConfig::default(),
			&spec,
			temp.as_path(),
			Arc::new(Miner::with_spec(&spec)),
			IoChannel::disconnected(),
			&db_config
		).unwrap();
		let mut worker = nanoipc::Worker::new(&(client as Arc<BlockChainClient>));
		worker.add_reqrep(&socket_path).unwrap();
		while !stop.load(Ordering::Relaxed) {
			worker.poll();
		}
	});
}

#[test]
fn can_handshake() {
	crossbeam::scope(|scope| {
		let stop_guard = StopGuard::new();
		let socket_path = "ipc:///tmp/parity-client-rpc-10.ipc";
		run_test_worker(scope, stop_guard.share(), socket_path);
		let remote_client = nanoipc::generic_client::<RemoteClient<_>>(socket_path).unwrap();

		assert!(remote_client.handshake().is_ok());
	})
}

#[test]
fn can_query_block() {
	crossbeam::scope(|scope| {
		let stop_guard = StopGuard::new();
		let socket_path = "ipc:///tmp/parity-client-rpc-20.ipc";
		run_test_worker(scope, stop_guard.share(), socket_path);
		let remote_client = nanoipc::generic_client::<RemoteClient<_>>(socket_path).unwrap();

		let non_existant_block = remote_client.block_header(BlockId::Number(999));

		assert!(non_existant_block.is_none());
	})
}
