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

//! Parity database ipc service

extern crate ethcore_db as db;
extern crate ethcore_ipc_nano as nanoipc;
extern crate rustc_serialize;
extern crate docopt;
extern crate ethcore_ipc_hypervisor as hypervisor;
extern crate ctrlc;
extern crate ethcore_devtools as devtools;

use db::database::Database;
use docopt::Docopt;
use std::sync::Arc;
use hypervisor::{HypervisorServiceClient, BLOCKCHAIN_MODULE_ID, HYPERVISOR_IPC_URL};
use ctrlc::CtrlC;
use std::sync::atomic::*;

const USAGE: &'static str = "
Ethcore database service

Usage:
  db <path>
";

#[derive(Debug, RustcDecodable)]
struct Args {
	arg_path: String,
}

fn init_worker(addr: &str) -> nanoipc::Worker<Database> {
	let mut worker = nanoipc::Worker::<Database>::new(&Arc::new(Database::new()));
	worker.add_reqrep(addr).unwrap();
	worker
}

fn main() {
	let args: Args = Docopt::new(USAGE)
							.and_then(|d| d.decode())
							.unwrap_or_else(|e| e.exit());

//	println!("Database: {}", args.arg_path);

	let blocks_url = db::blocks_service_url(&args.arg_path).unwrap();
	let extras_url = db::extras_service_url(&args.arg_path).unwrap();

	let stop = Arc::new(AtomicBool::new(false));
	let extras_stop = stop.clone();
	let main_stop = stop.clone();

	std::thread::spawn(move || {
		let mut extras_db_worker = init_worker(&extras_url);
		while !extras_stop.load(Ordering::Relaxed) {
			extras_db_worker.poll();
		}
	});

	let mut blocks_db_worker = init_worker(&blocks_url);

	let hypervisor_client = nanoipc::init_client::<HypervisorServiceClient<_>>(HYPERVISOR_IPC_URL).unwrap();
	hypervisor_client.handshake().unwrap();
	hypervisor_client.module_ready(BLOCKCHAIN_MODULE_ID);

	CtrlC::set_handler(move || {
		std::thread::sleep(std::time::Duration::new(1, 0));
		stop.store(true, Ordering::Relaxed);
	});

	while !main_stop.load(Ordering::Relaxed) {
		blocks_db_worker.poll();
	}
}
