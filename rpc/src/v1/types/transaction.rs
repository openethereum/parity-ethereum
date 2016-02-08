use util::hash::*;
use util::uint::*;
use v1::types::Bytes;

#[derive(Debug, Default, Serialize)]
pub struct Transaction {
	hash: H256,
	nonce: U256,
	#[serde(rename="blockHash")]
	block_hash: H256,
	#[serde(rename="blockNumber")]
	block_number: U256,
	#[serde(rename="transactionIndex")]
	transaction_index: U256,
	from: Address,
	to: Address,
	value: U256,
	#[serde(rename="gasPrice")]
	gas_price: U256,
	gas: U256,
	input: Bytes
}

#[cfg(test)]
mod tests {
	use super::*;
	use serde_json;

	#[test]
	fn test_transaction_serialize() {
		let t = Transaction::default();
		let serialized = serde_json::to_string(&t).unwrap();
		assert_eq!(serialized, r#"{"hash":"0x0000000000000000000000000000000000000000000000000000000000000000","nonce":"0x00","blockHash":"0x0000000000000000000000000000000000000000000000000000000000000000","blockNumber":"0x00","transactionIndex":"0x00","from":"0x0000000000000000000000000000000000000000","to":"0x0000000000000000000000000000000000000000","value":"0x00","gasPrice":"0x00","gas":"0x00","input":"0x00"}"#);
	}
}

