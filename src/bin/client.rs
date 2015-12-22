extern crate ethcore_util as util;

use std::io::*;
use util::network::{NetworkService};


fn main() {
	let mut service = NetworkService::start().unwrap();
	loop {
		let mut cmd = String::new();
		stdin().read_line(&mut cmd).unwrap();
		if cmd == "quit" || cmd == "exit" || cmd == "q" {
			break;
		}
	}
}

