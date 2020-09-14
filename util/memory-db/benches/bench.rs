// Copyright 2017, 2018 Parity Technologies
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

#[macro_use]
extern crate criterion;
use criterion::{black_box, Criterion};
criterion_group!(
    benches,
    instantiation,
    compare_to_null_embedded_in_struct,
    compare_to_null_in_const,
    contains_with_non_null_key,
    contains_with_null_key
);
criterion_main!(benches);

extern crate hash_db;
extern crate keccak_hasher;
extern crate memory_db;

use hash_db::{HashDB, Hasher};
use keccak_hasher::KeccakHasher;
use memory_db::MemoryDB;

fn instantiation(b: &mut Criterion) {
    b.bench_function("instantiation", |b| {
        b.iter(|| {
            MemoryDB::<KeccakHasher, Vec<u8>>::default();
        })
    });
}

fn compare_to_null_embedded_in_struct(b: &mut Criterion) {
    struct X {
        a_hash: <KeccakHasher as Hasher>::Out,
    };
    let x = X {
        a_hash: KeccakHasher::hash(&[0u8][..]),
    };
    let key = KeccakHasher::hash(b"abc");

    b.bench_function("compare_to_null_embedded_in_struct", move |b| {
        b.iter(|| {
            black_box(key == x.a_hash);
        })
    });
}

fn compare_to_null_in_const(b: &mut Criterion) {
    let key = KeccakHasher::hash(b"abc");

    b.bench_function("compare_to_null_in_const", move |b| {
        b.iter(|| {
            black_box(key == [0u8; 32]);
        })
    });
}

fn contains_with_non_null_key(b: &mut Criterion) {
    let mut m = MemoryDB::<KeccakHasher, Vec<u8>>::default();
    let key = KeccakHasher::hash(b"abc");
    m.insert(b"abcefghijklmnopqrstuvxyz");
    b.bench_function("contains_with_non_null_key", move |b| {
        b.iter(|| {
            m.contains(&key);
        })
    });
}

fn contains_with_null_key(b: &mut Criterion) {
    let mut m = MemoryDB::<KeccakHasher, Vec<u8>>::default();
    let null_key = KeccakHasher::hash(&[0u8][..]);
    m.insert(b"abcefghijklmnopqrstuvxyz");
    b.bench_function("contains_with_null_key", move |b| {
        b.iter(|| {
            m.contains(&null_key);
        })
    });
}
