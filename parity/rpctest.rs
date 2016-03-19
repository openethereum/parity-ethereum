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

extern crate ctrlc;
extern crate docopt;
extern crate rustc_serialize;
extern crate serde_json;
extern crate ethjson;
extern crate ethcore_util as util;
extern crate ethcore;
extern crate ethcore_devtools as devtools;
extern crate ethcore_rpc as rpc;

use std::collections::HashMap;
use std::sync::{Arc, Mutex, Condvar};
use std::process;
use std::fs::File;
use std::path::Path;
use docopt::Docopt;
use ctrlc::CtrlC;
use ethcore::spec::Genesis;
use ethcore::pod_state::PodState;
use ethcore::ethereum;
use ethcore::client::{BlockChainClient, Client, ClientConfig};
use devtools::RandomTempPath;
use util::IoChannel;
use rpc::v1::tests::helpers::{TestSyncProvider, Config as SyncConfig, TestMinerService, TestAccountProvider, TestAccount};
use rpc::v1::{Eth, EthClient, EthFilter, EthFilterClient};
use util::panics::MayPanic;
use util::hash::Address;

const USAGE: &'static str = r#"
Parity rpctest client.
  By Wood/Paronyan/Kotewicz/Drwięga/Volf.
  Copyright 2015, 2016 Ethcore (UK) Limited

Usage:
  rpctest --json <test-file> --name <test-name> [options]
  rpctest --help

Options:
  --jsonrpc-addr HOST      Specify the hostname portion of the JSONRPC API
                           server [default: 127.0.0.1].
  --jsonrpc-port PORT      Specify the port portion of the JSONRPC API server
                           [default: 8545].
"#;

#[derive(Debug, RustcDecodable)]
struct Args {
	arg_test_file: String,
	arg_test_name: String,
	flag_jsonrpc_addr: String,
	flag_jsonrpc_port: u16,
}

struct Configuration {
	args: Args,
}

impl Configuration {
	fn parse() -> Self {
		Configuration {
			args: Docopt::new(USAGE).and_then(|d| d.decode()).unwrap_or_else(|e| e.exit())
		}
	}

	fn execute(&self) {
		println!("file path: {:?}", self.args.arg_test_file);
		println!("test name: {:?}", self.args.arg_test_name);

		let path = Path::new(&self.args.arg_test_file);
		let file = File::open(path).unwrap_or_else(|_| {
			println!("Cannot open file.");
			process::exit(1);
		});

		let	tests: ethjson::blockchain::Test = serde_json::from_reader(file).unwrap_or_else(|err| {
			println!("Invalid json file.");
			println!("{:?}", err);
			process::exit(2);
		});

		let blockchain = tests.get(&self.args.arg_test_name).unwrap_or_else(|| {
			println!("Invalid test name.");
			process::exit(3);
		});

		let genesis = Genesis::from(blockchain.genesis());
		let state = PodState::from(blockchain.pre_state.clone());
		let mut spec = ethereum::new_frontier_test();
		spec.set_genesis_state(state);
		spec.overwrite_genesis_params(genesis);
		assert!(spec.is_state_root_valid());

		let temp = RandomTempPath::new();
		{
			let client: Arc<Client> = Client::new(ClientConfig::default(), spec, temp.as_path(), IoChannel::disconnected()).unwrap();
			for b in &blockchain.blocks_rlp() {
				let _ = client.import_block(b.clone());
				client.flush_queue();
				client.import_verified_blocks(&IoChannel::disconnected());
			}
			let sync = Arc::new(TestSyncProvider::new(SyncConfig {
				protocol_version: 65,
				num_peers: 120
			}));

			let miner = Arc::new(TestMinerService::default());
			let mut accs = HashMap::new();
			accs.insert(Address::from(1), TestAccount::new("test"));
			let accounts = Arc::new(TestAccountProvider::new(accs));
			let server = rpc::RpcServer::new();
			server.add_delegate(EthClient::new(&client, &sync, &accounts, &miner).to_delegate());
			server.add_delegate(EthFilterClient::new(&client, &miner).to_delegate());

			let url = format!("{}:{}", self.args.flag_jsonrpc_addr, self.args.flag_jsonrpc_port);
			let panic_handler = server.start_http(url.as_ref(), "*", 1);
			let exit = Arc::new(Condvar::new());

			let e = exit.clone();
			CtrlC::set_handler(move || { e.notify_all(); });

			let e = exit.clone();
			panic_handler.on_panic(move |_reason| { e.notify_all(); });

			let mutex = Mutex::new(());
			let _ = exit.wait(mutex.lock().unwrap()).unwrap();
		}

	}
}

fn main() {
	Configuration::parse().execute();
}
