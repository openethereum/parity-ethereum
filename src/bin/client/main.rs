#![feature(plugin)]
//#![plugin(docopt_macros)]

extern crate docopt;
extern crate rustc_serialize;
extern crate ethcore_util as util;
extern crate ethcore;
extern crate log;
extern crate env_logger;

use std::io::stdin;
use std::env;
use log::{LogLevelFilter};
use env_logger::LogBuilder;
use util::*;
use ethcore::client::*;
use ethcore::service::ClientService;
use ethcore::ethereum;
use ethcore::blockchain::CacheSize;
use ethcore::sync::*;
use docopt::Docopt;

const USAGE: &'static str = "
Parity. Ethereum Client.

Usage:
  parity [options]
  parity [options] <enode>...

Options:
  -l --logging LOGGING  Specify the logging level
  -h --help             Show this screen.
";

#[derive(Debug, RustcDecodable)]
struct Args {
    arg_enode: Option<Vec<String>>,
    flag_logging: Option<String>,
}

fn setup_log(init: &Option<String>) {
	let mut builder = LogBuilder::new();
	builder.filter(None, LogLevelFilter::Info);

	if env::var("RUST_LOG").is_ok() {
		builder.parse(&env::var("RUST_LOG").unwrap());
	}

	if let &Some(ref x) = init {
		builder.parse(x);
	}

	builder.init().unwrap();
}

fn main() {
	let args: Args = Docopt::new(USAGE).and_then(|d| d.decode()).unwrap_or_else(|e| e.exit());

	setup_log(&args.flag_logging);

	let spec = ethereum::new_frontier();

	let init_nodes = match &args.arg_enode {
		&None => spec.nodes().clone(),
		&Some(ref enodes) => enodes.clone(),
	};

	let mut service = ClientService::start(spec, &init_nodes).unwrap();
	let io_handler  = Box::new(ClientIoHandler { client: service.client(), timer: 0, info: Default::default() });

	service.io().register_handler(io_handler).expect("Error registering IO handler");
	loop {
		let mut cmd = String::new();
		stdin().read_line(&mut cmd).unwrap();
		if cmd == "quit\n" || cmd == "exit\n" || cmd == "q\n" {
			break;
		}
	}
}

#[derive(Default, Debug)]
struct Informant {
	chain_info: Option<BlockChainInfo>,
	cache_info: Option<CacheSize>,
	report: Option<ClientReport>,
}

impl Informant {
	pub fn tick(&mut self, client: &Client) {
		// 5 seconds betwen calls. TODO: calculate this properly.
		let dur = 5usize;

		let chain_info = client.chain_info();
		let cache_info = client.cache_info();
		let report = client.report();

		if let (_, &Some(ref last_cache_info), &Some(ref last_report)) = (&self.chain_info, &self.cache_info, &self.report) {
			println!("[ {} {} ]---[ {} blk/s | {} tx/s | {} gas/s  //···{}···//  {} ({}) bl  {} ({}) ex ]",
				chain_info.best_block_number,
				chain_info.best_block_hash,
				(report.blocks_imported - last_report.blocks_imported) / dur,
				(report.transactions_applied - last_report.transactions_applied) / dur,
				(report.gas_processed - last_report.gas_processed) / From::from(dur),
				0, // TODO: peers
				cache_info.blocks,
				cache_info.blocks as isize - last_cache_info.blocks as isize,
				cache_info.block_details,
				cache_info.block_details as isize - last_cache_info.block_details as isize
			);
		}

		self.chain_info = Some(chain_info);
		self.cache_info = Some(cache_info);
		self.report = Some(report);
	}
}

struct ClientIoHandler {
	client: Arc<RwLock<Client>>,
	timer: TimerToken,
	info: Informant,
}

impl IoHandler<NetSyncMessage> for ClientIoHandler {
	fn initialize<'s>(&'s mut self, io: &mut IoContext<'s, NetSyncMessage>) { 
		self.timer = io.register_timer(5000).expect("Error registering timer");
	}

	fn timeout<'s>(&'s mut self, _io: &mut IoContext<'s, NetSyncMessage>, timer: TimerToken) {
		if self.timer == timer {
			let client = self.client.read().unwrap();
			client.tick();
			self.info.tick(client.deref());
		}
	}
}

