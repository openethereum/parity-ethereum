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

//! Client RPC tests

use nanoipc;
use std::sync::Arc;
use std::io::Write;
use std::sync::atomic::{Ordering, AtomicBool};
use client::{BlockChainClient, MiningBlockChainClient, Client, ClientConfig, BlockID, RemoteClient};
use block::IsBlock;
use tests::helpers::*;
use common::*;
use devtools::*;
use miner::Miner;
use crossbeam;

pub fn run_test_worker(scope: &crossbeam::Scope, stop: Arc<AtomicBool>, socket_path: &str) {
	let socket_path = socket_path.to_owned();
	scope.spawn(move || {
		let client = Client::new(
			ClientConfig::default(),
			get_test_spec(),
			dir.as_path(),
			Arc::new(Miner::with_spec(get_test_spec())),
			IoChannel::disconnected()).unwrap();
		let mut worker = nanoipc::Worker::new(&Arc::new(client));
		worker.add_reqrep(&socket_path).unwrap();
		while !stop.load(Ordering::Relaxed) {
			worker.poll();
		}
	});
}

#[test]
fn can_be_created() {
	crossbeam::scope(|scope| {
		let stop_guard = StopGuard::new();
		let socket_path = "ipc:///tmp/parity-client-rpc-10.ipc";
		run_test_worker(scope, stop_guard.share(), socket_path);
		let remote_client = nanoipc::init_client::<RemoteClient<_>>(socket_path).unwrap();

    	let non_existant = remote_client.block_header(BlockID::Number(188));

		assert!(non_existant.is_none());
	})
}
