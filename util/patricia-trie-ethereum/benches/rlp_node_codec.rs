// Copyright 2015-2020 Parity Technologies (UK) Ltd.
// This file is part of Parity Ethereum.

// Parity Ethereum is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity Ethereum is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity Ethereum.  If not, see <http://www.gnu.org/licenses/>.

//! Benchmarking RlpNodeCodec decoding performance

extern crate criterion;
extern crate patricia_trie_ethereum as ethtrie;
extern crate trie_db;
extern crate ethereum_types;
extern crate rlp;

use criterion::{Criterion, criterion_group, criterion_main};
use ethereum_types::H256;
use ethtrie::RlpNodeCodec;
use rlp::RlpStream;
use trie_db::NodeCodec;

fn decoding(c: &mut Criterion) {
	c.bench_function("decode leaf (inline)", |b| {
		let mut stream = RlpStream::new_list(2);
		stream.append(&"cat").append(&"dog");
		let data = stream.out();
		b.iter(|| { RlpNodeCodec::decode(&data) });
	});

	c.bench_function("decode extension (inline)", |b| {
		let mut stream = RlpStream::new_list(2);
		let payload = vec![0x1, 0x2, 0x3, 0x4, 0x5, 0x6, 0x7, 0x8, 0x9u8];
		stream.append(&"").append(&payload);
		let data = stream.out();
		b.iter(|| { RlpNodeCodec::decode(&data) });
	});

	c.bench_function("decode extension (hash)", |b| {
		let mut stream = RlpStream::new_list(2);
		let payload = H256::random();
		stream.append(&"").append(&payload);
		let data = stream.out();
		b.iter(|| { RlpNodeCodec::decode(&data) });
	});

	c.bench_function("decode branch (hash)", |b| {
		let mut stream = RlpStream::new_list(17);
		for _ in 0..17 {
			stream.append(&H256::random());

		}
		let data = stream.out();
		b.iter(|| { RlpNodeCodec::decode(&data) });
	});

	c.bench_function("decode branch (inline)", |b| {
		let mut stream = RlpStream::new_list(17);
		for _ in 0..17 {
			stream.append(&[&H256::random().as_bytes(), H256::random().as_bytes()].concat());
		}
		let data = stream.out();
		b.iter(|| { RlpNodeCodec::decode(&data) });
	});

	c.bench_function("decode empty data", |b| {
		let mut stream = RlpStream::new();
		stream.append_empty_data();
		let data = stream.out();
		b.iter(|| { RlpNodeCodec::decode(&data)});
	});
}

criterion_group!(benches, decoding);
criterion_main!(benches);
