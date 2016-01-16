extern crate ethcore_util as util;
extern crate ethcore;
extern crate rustc_serialize;
extern crate log;
extern crate env_logger;

use std::io::*;
use std::env;
use log::{LogLevelFilter};
use env_logger::LogBuilder;
use util::hash::*;
use ethcore::service::ClientService;
use ethcore::ethereum;

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
	let mut _service = ClientService::start(spec).unwrap();
	loop {
		let mut cmd = String::new();
		stdin().read_line(&mut cmd).unwrap();
		if cmd == "quit\n" || cmd == "exit\n" || cmd == "q\n" {
			break;
		}
	}
}

