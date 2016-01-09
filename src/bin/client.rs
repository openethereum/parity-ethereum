extern crate ethcore_util as util;
extern crate ethcore;
extern crate rustc_serialize;

use std::io::*;
use std::env;
use std::sync::Arc;
use util::hash::*;
use util::network::{NetworkService};
use ethcore::client::Client;
use ethcore::sync::EthSync;
use ethcore::ethereum;

fn main() {
	let mut service = NetworkService::start().unwrap();
	//TODO: replace with proper genesis and chain params.
	let frontier = ethereum::new_frontier();
	let mut dir = env::temp_dir();
	dir.push(H32::random().hex());
	let client = Arc::new(Client::new(&frontier.genesis_block(), &dir));
	EthSync::register(&mut service, client);
	loop {
		let mut cmd = String::new();
		stdin().read_line(&mut cmd).unwrap();
		if cmd == "quit\n" || cmd == "exit\n" || cmd == "q\n" {
			break;
		}
	}
}

