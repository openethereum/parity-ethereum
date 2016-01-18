extern crate ethcore_util as util;
extern crate ethcore;
extern crate rustc_serialize;
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
use ethcore::sync::*;

fn setup_log() {
	let mut builder = LogBuilder::new();
	builder.filter(None, LogLevelFilter::Info);

	if env::var("RUST_LOG").is_ok() {
		builder.parse(&env::var("RUST_LOG").unwrap());
	}

	builder.init().unwrap();
}

fn main() {
	setup_log();
	let spec = ethereum::new_frontier();
	let mut service = ClientService::start(spec).unwrap();
	let io_handler  = Box::new(ClientIoHandler { client: service.client(), timer: 0 });
	service.io().register_handler(io_handler).expect("Error registering IO handler");
	loop {
		let mut cmd = String::new();
		stdin().read_line(&mut cmd).unwrap();
		if cmd == "quit\n" || cmd == "exit\n" || cmd == "q\n" {
			break;
		}
	}
}


struct ClientIoHandler {
	client: Arc<RwLock<Client>>,
	timer: TimerToken,
}

impl IoHandler<NetSyncMessage> for ClientIoHandler {
	fn initialize<'s>(&'s mut self, io: &mut IoContext<'s, NetSyncMessage>) { 
		self.timer = io.register_timer(5000).expect("Error registering timer");
	}

	fn timeout<'s>(&'s mut self, _io: &mut IoContext<'s, NetSyncMessage>, timer: TimerToken) {
		if self.timer == timer {
			self.client.tick();
			println!("Chain info: {}", self.client.read().unwrap().deref().chain_info());
			println!("Cache info: {:?}", self.client.read().unwrap().deref().cache_info());
		}
	}
}

