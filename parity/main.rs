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

//! Ethcore client application.

#![warn(missing_docs)]
#![feature(plugin)]
#![plugin(docopt_macros)]
#![plugin(clippy)]
extern crate docopt;
extern crate rustc_serialize;
extern crate ethcore_util as util;
extern crate ethcore;
extern crate ethsync;
extern crate log as rlog;
extern crate env_logger;
extern crate ctrlc;

#[cfg(feature = "rpc")]
extern crate ethcore_rpc as rpc;

use std::env;
use rlog::{LogLevelFilter};
use env_logger::LogBuilder;
use ctrlc::CtrlC;
use util::*;
use ethcore::client::*;
use ethcore::service::{ClientService, NetSyncMessage};
use ethcore::ethereum;
use ethcore::blockchain::CacheSize;
use ethsync::EthSync;

docopt!(Args derive Debug, "
Parity. Ethereum Client.

Usage:
  parity [options]
  parity [options] <enode>...

Options:
  -l --logging LOGGING     Specify the logging level.
  -j --jsonrpc             Enable the JSON-RPC API sever.
  --jsonrpc-url URL        Specify URL for JSON-RPC API server [default: 127.0.0.1:8545].

  --cache-pref-size BYTES  Specify the prefered size of the blockchain cache in bytes [default: 16384].
  --cache-max-size BYTES   Specify the maximum size of the blockchain cache in bytes [default: 262144].

  -h --help                Show this screen.
", flag_cache_pref_size: usize, flag_cache_max_size: usize);

fn setup_log(init: &str) {
	let mut builder = LogBuilder::new();
	builder.filter(None, LogLevelFilter::Info);

	if env::var("RUST_LOG").is_ok() {
		builder.parse(&env::var("RUST_LOG").unwrap());
	}

	builder.parse(init);

	builder.init().unwrap();
}


#[cfg(feature = "rpc")]
fn setup_rpc_server(client: Arc<Client>, sync: Arc<EthSync>, url: &str) {
	use rpc::v1::*;
	
	let mut server = rpc::HttpServer::new(1);
	server.add_delegate(Web3Client::new().to_delegate());
	server.add_delegate(EthClient::new(client.clone()).to_delegate());
	server.add_delegate(EthFilterClient::new(client).to_delegate());
	server.add_delegate(NetClient::new(sync).to_delegate());
	server.start_async(url);
}

#[cfg(not(feature = "rpc"))]
fn setup_rpc_server(_client: Arc<Client>, _sync: Arc<EthSync>, _url: &str) {
}

fn main() {
	let args: Args = Args::docopt().decode().unwrap_or_else(|e| e.exit());

	setup_log(&args.flag_logging);

	let spec = ethereum::new_frontier();
	let init_nodes = match args.arg_enode.len() {
		0 => spec.nodes().clone(),
		_ => args.arg_enode.clone(),
	};
	let mut net_settings = NetworkConfiguration::new();
	net_settings.boot_nodes = init_nodes;
	let mut service = ClientService::start(spec, net_settings).unwrap();
	let client = service.client().clone();
	client.configure_cache(args.flag_cache_pref_size, args.flag_cache_max_size);
	let sync = EthSync::register(service.network(), client);
	if args.flag_jsonrpc {
		setup_rpc_server(service.client(), sync.clone(), &args.flag_jsonrpc_url);
	}
	let io_handler  = Arc::new(ClientIoHandler { client: service.client(), info: Default::default(), sync: sync });
	service.io().register_handler(io_handler).expect("Error registering IO handler");

	let exit = Arc::new(Condvar::new());
	let e = exit.clone();
	CtrlC::set_handler(move || { e.notify_all(); });
	let mutex = Mutex::new(());
	let _ = exit.wait(mutex.lock().unwrap()).unwrap();
}

struct Informant {
	chain_info: RwLock<Option<BlockChainInfo>>,
	cache_info: RwLock<Option<CacheSize>>,
	report: RwLock<Option<ClientReport>>,
}

impl Default for Informant {
	fn default() -> Self {
		Informant {
			chain_info: RwLock::new(None),
			cache_info: RwLock::new(None),
			report: RwLock::new(None),
		}
	}
}

impl Informant {
	pub fn tick(&self, client: &Client, sync: &EthSync) {
		// 5 seconds betwen calls. TODO: calculate this properly.
		let dur = 5usize;

		let chain_info = client.chain_info();
		let queue_info = client.queue_info();
		let cache_info = client.cache_info();
		let report = client.report();
		let sync_info = sync.status();

		if let (_, &Some(ref last_cache_info), &Some(ref last_report)) = (self.chain_info.read().unwrap().deref(), self.cache_info.read().unwrap().deref(), self.report.read().unwrap().deref()) {
			println!("[ {} {} ]---[ {} blk/s | {} tx/s | {} gas/s  //··· {}/{} peers, {} downloaded, {}+{} queued ···//  {} ({}) bl  {} ({}) ex ]",
				chain_info.best_block_number,
				chain_info.best_block_hash,
				(report.blocks_imported - last_report.blocks_imported) / dur,
				(report.transactions_applied - last_report.transactions_applied) / dur,
				(report.gas_processed - last_report.gas_processed) / From::from(dur),

				sync_info.num_active_peers,
				sync_info.num_peers,
				sync_info.blocks_received,
				queue_info.unverified_queue_size,
				queue_info.verified_queue_size,

				cache_info.blocks,
				cache_info.blocks as isize - last_cache_info.blocks as isize,
				cache_info.block_details,
				cache_info.block_details as isize - last_cache_info.block_details as isize
			);
		}

		*self.chain_info.write().unwrap().deref_mut() = Some(chain_info);
		*self.cache_info.write().unwrap().deref_mut() = Some(cache_info);
		*self.report.write().unwrap().deref_mut() = Some(report);
	}
}

const INFO_TIMER: TimerToken = 0;

struct ClientIoHandler {
	client: Arc<Client>,
	sync: Arc<EthSync>,
	info: Informant,
}

impl IoHandler<NetSyncMessage> for ClientIoHandler {
	fn initialize(&self, io: &IoContext<NetSyncMessage>) { 
		io.register_timer(INFO_TIMER, 5000).expect("Error registering timer");
	}

	fn timeout(&self, _io: &IoContext<NetSyncMessage>, timer: TimerToken) {
		if INFO_TIMER == timer {
			self.info.tick(&self.client, &self.sync);
		}
	}
}

/// Parity needs at least 1 test to generate coverage reports correctly.
#[test]
fn if_works() {
}
