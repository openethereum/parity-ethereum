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
	let out = encode(&foo).into_vec();
	assert_eq!(out, expected);

	let decoded = decode(&expected);
	assert_eq!(foo, decoded);
}

#[test]
fn test_encode_foo_wrapper() {
	let foo = FooWrapper {
		a: "cat".into(),
	};

	let expected = vec![0x83, b'c', b'a', b't'];
	let out = encode(&foo).into_vec();
	assert_eq!(out, expected);

	let decoded = decode(&expected);
	assert_eq!(foo, decoded);
}

