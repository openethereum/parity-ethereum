extern crate ethcore_util as util;
extern crate ethcore;
extern crate rustc_serialize;
extern crate env_logger;

use std::io::*;
use util::hash::*;
use ethcore::service::ClientService;
use ethcore::ethereum;

fn main() {
	::env_logger::init().ok();
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

