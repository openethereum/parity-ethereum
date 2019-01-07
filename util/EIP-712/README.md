# EIP-712 ![Crates.io](https://img.shields.io/crates/d/EIP-712.svg) [![Released API docs](https://docs.rs/EIP-712/badge.svg)](https://docs.rs/EIP-712)

## Example

```rust
use eip_712::{EIP712, hash_structured_data};
use serde_json::from_str;
use rustc_hex::ToHex;

fn main() {
	let json = r#"{
		"primaryType": "Mail",
		"domain": {
			"name": "Ether Mail",
			"version": "1",
			"chainId": "0x1",
			"verifyingContract": "0xCcCCccccCCCCcCCCCCCcCcCccCcCCCcCcccccccC"
		},
		"message": {
			"from": {
				"name": "Cow",
				"wallet": "0xCD2a3d9F938E13CD947Ec05AbC7FE734Df8DD826"
			},
			"to": {
				"name": "Bob",
				"wallet": "0xbBbBBBBbbBBBbbbBbbBbbbbBBbBbbbbBbBbbBBbB"
			},
			"contents": "Hello, Bob!"
		},
		"types": {
			"EIP712Domain": [
				{ "name": "name", "type": "string" },
				{ "name": "version", "type": "string" },
				{ "name": "chainId", "type": "uint256" },
				{ "name": "verifyingContract", "type": "address" }
			],
			"Person": [
				{ "name": "name", "type": "string" },
				{ "name": "wallet", "type": "address" }
			],
			"Mail": [
				{ "name": "from", "type": "Person" },
				{ "name": "to", "type": "Person" },
				{ "name": "contents", "type": "string" }
			]
		}
	}"#;
	let typed_data = from_str::<EIP712>(json).unwrap();

	assert_eq!(
		hash_structured_data(typed_data).unwrap().to_hex::<String>(),
		"be609aee343fb3c4b28e1df9e632fca64fcfaede20f02e86244efddf30957bd2"
	)
}

```

## License

This crate is distributed under the terms of GNU GENERAL PUBLIC LICENSE version 3.0.

See [LICENSE](../../LICENSE) for details.
