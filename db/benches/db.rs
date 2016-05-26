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

#![feature(test)]

extern crate ethcore_devtools as devtools;
extern crate test;
extern crate crossbeam;
extern crate ethcore_db;
extern crate ethcore_ipc_nano as nanoipc;

use ethcore_db::{DatabaseClient, Database, DatabaseService};
use test::{Bencher};

#[bench]
fn key_write_ipc(bencher: &mut Bencher) {
	crossbeam::scope(|scope| {
		let stop = devtools::StopGuard::new();
		let temp = devtools::RandomTempPath::create_dir();
		let ipc_url = ethcore_db::extras_service_url(temp.as_str()).unwrap();
		ethcore_db::run_worker(&scope, stop.share(), &ipc_url);
		let client = nanoipc::init_client::<DatabaseClient<_>>(&ipc_url).unwrap();
		client.open_default(temp.as_str().to_owned()).unwrap();
		bencher.iter(|| {
			client.put(devtools::random_str(256).as_bytes(), devtools::random_str(256).as_bytes()).unwrap();
			client.put(devtools::random_str(256).as_bytes(), devtools::random_str(2048).as_bytes()).unwrap();
			client.put(devtools::random_str(2048).as_bytes(), devtools::random_str(256).as_bytes()).unwrap();
			client.put(devtools::random_str(2048).as_bytes(), devtools::random_str(2048).as_bytes()).unwrap();
		});
	});
}

#[bench]
fn key_write_direct(bencher: &mut Bencher) {
	let temp = devtools::RandomTempPath::create_dir();
	let client = Database::new();
	client.open_default(temp.as_str().to_owned()).unwrap();
	bencher.iter(|| {
		client.put(devtools::random_str(256).as_bytes(), devtools::random_str(256).as_bytes()).unwrap();
		client.put(devtools::random_str(256).as_bytes(), devtools::random_str(2048).as_bytes()).unwrap();
		client.put(devtools::random_str(2048).as_bytes(), devtools::random_str(256).as_bytes()).unwrap();
		client.put(devtools::random_str(2048).as_bytes(), devtools::random_str(2048).as_bytes()).unwrap();
	});
}
