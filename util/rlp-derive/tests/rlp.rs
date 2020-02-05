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

use rlp::{decode, encode};
use rlp_derive::{RlpDecodable, RlpDecodableWrapper, RlpEncodable, RlpEncodableWrapper};

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
	let foo = Foo { a: "cat".into() };

	let expected = vec![0xc4, 0x83, b'c', b'a', b't'];
	let out = encode(&foo);
	assert_eq!(out, expected);

	let decoded = decode(&expected).expect("decode failure");
	assert_eq!(foo, decoded);
}

#[test]
fn test_encode_foo_wrapper() {
	let foo = FooWrapper { a: "cat".into() };

	let expected = vec![0x83, b'c', b'a', b't'];
	let out = encode(&foo);
	assert_eq!(out, expected);

	let decoded = decode(&expected).expect("decode failure");
	assert_eq!(foo, decoded);
}

#[test]
fn test_encode_foo_default() {
	#[derive(Debug, PartialEq, RlpEncodable, RlpDecodable)]
	struct FooDefault {
		a: String,
		/// It works with other attributes.
		#[rlp(default)]
		b: Option<Vec<u8>>,
	}

	let attack_of = String::from("clones");
	let foo = Foo { a: attack_of.clone() };

	let expected = vec![0xc7, 0x86, b'c', b'l', b'o', b'n', b'e', b's'];
	let out = encode(&foo);
	assert_eq!(out, expected);

	let foo_default = FooDefault { a: attack_of.clone(), b: None };

	let decoded = decode(&expected).expect("default failure");
	assert_eq!(foo_default, decoded);

	let foo_some = FooDefault { a: attack_of.clone(), b: Some(vec![1, 2, 3]) };
	let out = encode(&foo_some);
	assert_eq!(decode(&out), Ok(foo_some));
}
