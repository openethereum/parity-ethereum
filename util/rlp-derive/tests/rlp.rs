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

extern crate rlp;
#[macro_use]
extern crate rlp_derive;

use rlp::{encode, decode};

#[derive(Debug, PartialEq, RlpEncodable, RlpDecodable)]
struct Foo {
	a: String,
}

#[derive(Debug, PartialEq, RlpEncodableWrapper, RlpDecodableWrapper)]
struct FooWrapper {
	a: String,
}

#[test]
fn test_encode_foo() {
	let foo = Foo {
		a: "cat".into(),
	};

	let expected = vec![0xc4, 0x83, b'c', b'a', b't'];
	let out = encode(&foo);
	assert_eq!(out, expected);

	let decoded = decode(&expected).expect("decode failure");
	assert_eq!(foo, decoded);
}

#[test]
fn test_encode_foo_wrapper() {
	let foo = FooWrapper {
		a: "cat".into(),
	};

	let expected = vec![0x83, b'c', b'a', b't'];
	let out = encode(&foo);
	assert_eq!(out, expected);

	let decoded = decode(&expected).expect("decode failure");
	assert_eq!(foo, decoded);
}
