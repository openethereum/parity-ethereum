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

extern crate ethcore;
extern crate ethcore_devtools as devtools;
extern crate test;
extern crate crossbeam;
extern crate ethcore_util as util;
extern crate ethcore_db;

use ethcore::client::{Client, ClientConfig, BlockChainClient, get_db_path};
use ethcore::spec::Spec;
use util::*;
use devtools::*;
use test::{Bencher, black_box};
use ethcore::block::*;
use ethcore::header::*;

pub fn test_block(header: &Header) -> Bytes {
	let mut rlp = RlpStream::new_list(3);
	rlp.append(header);
	rlp.append_raw(&rlp::EMPTY_LIST_RLP, 1);
	rlp.append_raw(&rlp::EMPTY_LIST_RLP, 1);
	rlp.out()
}

pub fn run_block_push(dir: &RandomTempPath, block_number: u32) {
	let client = Client::new(ClientConfig::default(), Spec::new_test(), dir.as_path(), IoChannel::disconnected()).unwrap();
	let test_spec = Spec::new_test();
	let test_engine = &test_spec.engine;
	let state_root = test_spec.genesis_header().state_root;
	let mut rolling_hash = test_spec.genesis_header().hash();
	let mut rolling_block_number = 1;
	let mut rolling_timestamp = 40;

	for _ in 0..block_number {
		let mut header = Header::new();

		header.gas_limit = test_engine.params().min_gas_limit;
		header.difficulty = U256::from(0x20000);
		header.timestamp = rolling_timestamp;
		header.number = rolling_block_number;
		header.parent_hash = rolling_hash;
		header.state_root = state_root.clone();

		rolling_hash = header.hash();
		rolling_block_number = rolling_block_number + 1;
		rolling_timestamp = rolling_timestamp + 10;

		if let Err(e) = client.import_block(test_block(&header)) {
			panic!("error importing block which is valid by definition: {:?}", e);
		}
	}
	client.flush_queue();
	client.import_verified_blocks(&IoChannel::disconnected());
}


#[bench]
fn block_write(bencher: &mut Bencher) {

	bencher.iter(|| {
		let temp = devtools::RandomTempPath::create_dir();
		let db_path = get_db_path(
			temp.as_path(),
			ClientConfig::default().pruning,
			Spec::new_test().genesis_header().hash()).to_str().unwrap().to_owned();

		crossbeam::scope(move |scope| {
			let stop = StopGuard::new();
			ethcore_db::run_worker(scope, stop.share(), &ethcore_db::extras_service_url(&db_path).unwrap());
			ethcore_db::run_worker(scope, stop.share(), &ethcore_db::blocks_service_url(&db_path).unwrap());

			run_block_push(&temp, 10);
		});
	});

}
