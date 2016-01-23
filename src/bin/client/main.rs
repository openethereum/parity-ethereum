#![feature(plugin)]
//#![plugin(docopt_macros)]

extern crate docopt;
extern crate rustc_serialize;
extern crate ethcore_util as util;
extern crate ethcore;
extern crate log;
extern crate env_logger;
extern crate ctrlc;

use std::env;
use log::{LogLevelFilter};
use env_logger::LogBuilder;
use ctrlc::CtrlC;
use util::*;
use ethcore::client::*;
use ethcore::service::{ClientService, NetSyncMessage};
use ethcore::ethereum;
use ethcore::blockchain::CacheSize;
use ethcore::sync::EthSync;
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
	let io_handler  = Arc::new(ClientIoHandler { client: service.client(), info: Default::default(), sync: service.sync() });
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

