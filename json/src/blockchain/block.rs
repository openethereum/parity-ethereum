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

//! Blockchain test block deserializer.

use bytes::Bytes;
use blockchain::header::Header;
use blockchain::transaction::Transaction;

/// Blockchain test block deserializer.
#[derive(Debug, PartialEq, Deserialize)]
pub struct Block {
	#[serde(rename = "blockHeader")]
	header: Option<Header>,
	rlp: Bytes,
	transactions: Option<Vec<Transaction>>,
	#[serde(rename = "uncleHeaders")]
	uncles: Option<Vec<Header>>,
}

impl Block {
	/// Returns block rlp.
	pub fn rlp(&self) -> Vec<u8> {
		self.rlp.clone().into()
	}
}

#[cfg(test)]
mod tests {
	use serde_json;
	use blockchain::block::Block;

	#[test]
	fn block_deserialization() {
		let s = r#"{
			"blockHeader" : {
				"bloom" : "00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000",
				"coinbase" : "8888f1f195afa192cfee860698584c030f4c9db1",
				"difficulty" : "0x020000",
				"extraData" : "0x",
				"gasLimit" : "0x2fefba",
				"gasUsed" : "0x00",
				"hash" : "65ebf1b97fb89b14680267e0723d69267ec4bf9a96d4a60ffcb356ae0e81c18f",
				"mixHash" : "13735ab4156c9b36327224d92e1692fab8fc362f8e0f868c94d421848ef7cd06",
				"nonce" : "931dcc53e5edc514",
				"number" : "0x01",
				"parentHash" : "5a39ed1020c04d4d84539975b893a4e7c53eab6c2965db8bc3468093a31bc5ae",
				"receiptTrie" : "56e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421",
				"stateRoot" : "c5c83ff43741f573a0c9b31d0e56fdd745f4e37d193c4e78544f302777aafcf3",
				"timestamp" : "0x56850b7b",
				"transactionsTrie" : "56e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421",
				"uncleHash" : "1dcc4de8dec75d7aab85b567b6ccd41ad312451b948a7413f0a142fd40d49347"
			},
			"blocknumber" : "1",
			"rlp" : "0xf901fcf901f7a05a39ed1020c04d4d84539975b893a4e7c53eab6c2965db8bc3468093a31bc5aea01dcc4de8dec75d7aab85b567b6ccd41ad312451b948a7413f0a142fd40d49347948888f1f195afa192cfee860698584c030f4c9db1a0c5c83ff43741f573a0c9b31d0e56fdd745f4e37d193c4e78544f302777aafcf3a056e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421a056e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421b90100000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000008302000001832fefba808456850b7b80a013735ab4156c9b36327224d92e1692fab8fc362f8e0f868c94d421848ef7cd0688931dcc53e5edc514c0c0",
			"transactions" : [],
			"uncleHeaders" : []
		}"#;
		let _deserialized: Block = serde_json::from_str(s).unwrap();
		// TODO: validate all fields
	}
}
