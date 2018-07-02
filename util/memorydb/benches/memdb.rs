// Copyright 2015-2018 Parity Technologies (UK) Ltd.
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

extern crate hashdb;
extern crate memorydb;
extern crate keccak_hasher;
extern crate keccak_hash;
extern crate rlp;
extern crate test;

use memorydb::MemoryDB;
use keccak_hasher::KeccakHasher;
use hashdb::{HashDB, Hasher};
use keccak_hash::KECCAK_NULL_RLP;
use rlp::NULL_RLP;
use test::{Bencher, black_box};


#[bench]
fn instantiation(b: &mut Bencher) {
    b.iter(|| {
        MemoryDB::<KeccakHasher>::new();
    })
}

#[bench]
fn compare_to_null_embedded_in_struct(b: &mut Bencher) {
    struct X {a_hash: <KeccakHasher as Hasher>::Out};
    let x = X {a_hash: KeccakHasher::hash(&NULL_RLP)};
    let key = KeccakHasher::hash(b"abc");

    b.iter(|| {
        black_box(key == x.a_hash);
    })
}

#[bench]
fn compare_to_null_in_const(b: &mut Bencher) {
    let key = KeccakHasher::hash(b"abc");

    b.iter(|| {
        black_box(key == KECCAK_NULL_RLP);
    })
}

#[bench]
fn contains_with_non_null_key(b: &mut Bencher) {
    let mut m = MemoryDB::<KeccakHasher>::new();
    let key = KeccakHasher::hash(b"abc");
    m.insert(b"abcefghijklmnopqrstuvxyz");
    b.iter(|| {
        m.contains(&key);
    })
}

#[bench]
fn contains_with_null_key(b: &mut Bencher) {
    let mut m = MemoryDB::<KeccakHasher>::new();
    let null_key = KeccakHasher::hash(&NULL_RLP);
    m.insert(b"abcefghijklmnopqrstuvxyz");
    b.iter(|| {
        m.contains(&null_key);
    })
}