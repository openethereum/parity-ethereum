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

//! Blockchain deserialization.

use bytes::Bytes;
use hash::H256;
use blockchain::state::State;
use blockchain::header::Header;
use blockchain::block::Block;
use spec::{Genesis, Seal, Ethereum};

/// Blockchain deserialization.
#[derive(Debug, PartialEq, Deserialize)]
pub struct BlockChain {
	/// Genesis block header.
	#[serde(rename="genesisBlockHeader")]
	pub genesis_block: Header,
	/// Genesis block rlp.
	#[serde(rename="genesisRLP")]
	pub genesis_rlp: Option<Bytes>,
	/// Blocks.
	pub blocks: Vec<Block>,
	/// Post state.
	#[serde(rename="postState")]
	pub post_state: State,
	/// Pre state.
	#[serde(rename="pre")]
	pub pre_state: State,
	/// Hash of best block.
	#[serde(rename="lastblockhash")]
	pub best_block: H256
}

impl BlockChain {
	/// Returns blocks rlp.
	pub fn blocks_rlp(&self) -> Vec<Vec<u8>> {
		self.blocks.iter().map(|block| block.rlp()).collect()
	}

	/// Returns spec compatible genesis struct.
	pub fn genesis(&self) -> Genesis {
		Genesis {
			seal: Seal::Ethereum(Ethereum {
				nonce: self.genesis_block.nonce.clone(),
				mix_hash: self.genesis_block.mix_hash.clone(),
			}),
			difficulty: self.genesis_block.difficulty,
			author: self.genesis_block.author.clone(),
			timestamp: self.genesis_block.timestamp,
			parent_hash: self.genesis_block.parent_hash.clone(),
			gas_limit: self.genesis_block.gas_limit,
			transactions_root: Some(self.genesis_block.transactions_root.clone()),
			receipts_root: Some(self.genesis_block.receipts_root.clone()),
			state_root: Some(self.genesis_block.state_root.clone()),
			gas_used: Some(self.genesis_block.gas_used),
			extra_data: Some(self.genesis_block.extra_data.clone()),
		}
	}
}

#[cfg(test)]
mod tests {
	use serde_json;
	use blockchain::blockchain::BlockChain;

	#[test]
	fn blockchain_deserialization() {
		let s = r#"{
			"blocks" : [{
				"blockHeader" : {
					"bloom" : "00000000000000001000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000020000000000000000000800000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000020000000000040000000000000000000000000000000000000000000000000000000",
					"coinbase" : "8888f1f195afa192cfee860698584c030f4c9db1",
					"difficulty" : "0x020000",
					"extraData" : "0x0102030405060708091011121314151617181920212223242526272829303132",
					"gasLimit" : "0x2fefba",
					"gasUsed" : "0x560b",
					"hash" : "06b5b1742bde29468510c92641f36b719c61b3fc3e9a21c92a23978f4f7faa2a",
					"mixHash" : "5266ca43e81d25925a9ba573c3e4f9180bc076d316d90e63c6f8708b272f5ce2",
					"nonce" : "59ba4daed1898e21",
					"number" : "0x01",
					"parentHash" : "f052d217bd5275a5177a3c3b7debdfe2670f1c8394b2965ccd5c1883cc1a524d",
					"receiptTrie" : "c7778a7376099ee2e5c455791c1885b5c361b95713fddcbe32d97fd01334d296",
					"stateRoot" : "bac6177a79e910c98d86ec31a09ae37ac2de15b754fd7bed1ba52362c49416bf",
					"timestamp" : "0x56850c2c",
					"transactionsTrie" : "498785da562aa0c5dd5937cf15f22139b0b1bcf3b4fc48986e1bb1dae9292796",
					"uncleHash" : "1dcc4de8dec75d7aab85b567b6ccd41ad312451b948a7413f0a142fd40d49347"
				},
				"rlp" : "0xf90285f90219a0f052d217bd5275a5177a3c3b7debdfe2670f1c8394b2965ccd5c1883cc1a524da01dcc4de8dec75d7aab85b567b6ccd41ad312451b948a7413f0a142fd40d49347948888f1f195afa192cfee860698584c030f4c9db1a0bac6177a79e910c98d86ec31a09ae37ac2de15b754fd7bed1ba52362c49416bfa0498785da562aa0c5dd5937cf15f22139b0b1bcf3b4fc48986e1bb1dae9292796a0c7778a7376099ee2e5c455791c1885b5c361b95713fddcbe32d97fd01334d296b90100000000000000000010000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000200000000000000000008000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000040000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000200000000000400000000000000000000000000000000000000000000000000000008302000001832fefba82560b8456850c2ca00102030405060708091011121314151617181920212223242526272829303132a05266ca43e81d25925a9ba573c3e4f9180bc076d316d90e63c6f8708b272f5ce28859ba4daed1898e21f866f864800a82c35094095e7baea6a6c7c4c2dfeb977efac326af552d8785012a05f200801ca0ee0b9ec878fbd4258a9473199d8ecc32996a20c323c004e79e0cda20e0418ce3a04e6bc63927d1510bab54f37e46fa036faf4b2c465d271920d9afea1fadf7bd21c0",
				"transactions" : [
					{
						"data" : "0x",
						"gasLimit" : "0xc350",
						"gasPrice" : "0x0a",
						"nonce" : "0x00",
						"r" : "0xee0b9ec878fbd4258a9473199d8ecc32996a20c323c004e79e0cda20e0418ce3",
						"s" : "0x4e6bc63927d1510bab54f37e46fa036faf4b2c465d271920d9afea1fadf7bd21",
						"to" : "095e7baea6a6c7c4c2dfeb977efac326af552d87",
						"v" : "0x1c",
						"value" : "0x012a05f200"
					}
				],
				"uncleHeaders" : [
				]
			}],
			"genesisBlockHeader" : {
				"bloom" : "00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000",
				"coinbase" : "8888f1f195afa192cfee860698584c030f4c9db1",
				"difficulty" : "0x020000",
				"extraData" : "0x42",
				"gasLimit" : "0x2fefd8",
				"gasUsed" : "0x00",
				"hash" : "f052d217bd5275a5177a3c3b7debdfe2670f1c8394b2965ccd5c1883cc1a524d",
				"mixHash" : "56e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421",
				"nonce" : "0102030405060708",
				"number" : "0x00",
				"parentHash" : "0000000000000000000000000000000000000000000000000000000000000000",
				"receiptTrie" : "56e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421",
				"stateRoot" : "925002c3260b44e44c3edebad1cc442142b03020209df1ab8bb86752edbd2cd7",
				"timestamp" : "0x54c98c81",
				"transactionsTrie" : "56e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421",
				"uncleHash" : "1dcc4de8dec75d7aab85b567b6ccd41ad312451b948a7413f0a142fd40d49347"
			},
			"genesisRLP" : "0xf901fcf901f7a00000000000000000000000000000000000000000000000000000000000000000a01dcc4de8dec75d7aab85b567b6ccd41ad312451b948a7413f0a142fd40d49347948888f1f195afa192cfee860698584c030f4c9db1a0925002c3260b44e44c3edebad1cc442142b03020209df1ab8bb86752edbd2cd7a056e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421a056e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421b90100000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000008302000080832fefd8808454c98c8142a056e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421880102030405060708c0c0",
			"lastblockhash" : "06b5b1742bde29468510c92641f36b719c61b3fc3e9a21c92a23978f4f7faa2a",
			"postState" : {
				"095e7baea6a6c7c4c2dfeb977efac326af552d87" : {
					"balance" : "0x012a05f264",
					"code" : "0x7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff600052600060206000a1",
					"nonce" : "0x00",
					"storage" : {
					}
				},
				"8888f1f195afa192cfee860698584c030f4c9db1" : {
					"balance" : "0x4563918244f75c6e",
					"code" : "0x",
					"nonce" : "0x00",
					"storage" : {
					}
				},
				"a94f5374fce5edbc8e2a8697c15331677e6ebf0b" : {
					"balance" : "0x012a029592",
					"code" : "0x",
					"nonce" : "0x01",
					"storage" : {
					}
				}
			},
			"pre" : {
				"095e7baea6a6c7c4c2dfeb977efac326af552d87" : {
					"balance" : "0x64",
					"code" : "0x7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff600052600060206000a1",
					"nonce" : "0x00",
					"storage" : {
					}
				},
				"a94f5374fce5edbc8e2a8697c15331677e6ebf0b" : {
					"balance" : "0x02540be400",
					"code" : "0x",
					"nonce" : "0x00",
					"storage" : {
					}
				}
			}
		}"#;
		let _deserialized: BlockChain = serde_json::from_str(s).unwrap();
		// TODO: validate all fields
	//}
}
