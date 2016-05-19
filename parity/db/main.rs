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

use db::database::Database;

fn init_worker(addr: &str) -> nanoipc::Worker<Database> {
	let mut worker = nanoipc::Worker::<Database>::new(&Arc::new(Database::new()));
	worker.add_reqrep(addr).unwrap();
	worker
}

fn main() {
	let mut blocks_db_worker = init_worker(url);
	loop {
		worker.poll();
		c_worker_is_ready.store(true, Ordering::Relaxed);
	}
}
