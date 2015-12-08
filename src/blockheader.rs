use util::hash::*;
use util::uint::*;
use util::rlp::*;

pub struct BlockHeader {
	parent_hash: H256,
	ommers_hash: H256,
	beneficiary: Address,
	state_root: H256,
	transactions_root: H256,
	receipts_root: H256,
	log_bloom: H2048,
	difficulty: U256,
	number: U256,
	gas_limit: U256,
	gas_used: U256,
	timestamp: U256,
	mix_hash: H256,
	nonce: H64
}

impl Decodable for BlockHeader {
	fn decode<D>(decoder: &D) -> Result<Self, DecoderError>  where D: Decoder {
		decoder.read_list(| d | {
			let blockheader = BlockHeader {
				parent_hash: try!(H256::decode(&d[0])),
				ommers_hash: try!(H256::decode(&d[1])),
				beneficiary: try!(Address::decode(&d[2])),
				state_root: try!(H256::decode(&d[3])),
				transactions_root: try!(H256::decode(&d[4])),
				receipts_root: try!(H256::decode(&d[5])),
				log_bloom: try!(H2048::decode(&d[6])),
				difficulty: try!(U256::decode(&d[7])),
				number: try!(U256::decode(&d[8])),
				gas_limit: try!(U256::decode(&d[9])),
				gas_used: try!(U256::decode(&d[10])),
				timestamp: try!(U256::decode(&d[11])),
				mix_hash: try!(H256::decode(&d[12])),
				nonce: try!(H64::decode(&d[13]))
			};
			Ok(blockheader)
		})
	}
}

impl Encodable for BlockHeader {
	fn encode<E>(&self, encoder: &mut E) where E: Encoder {
		encoder.emit_list(| e | {
			self.parent_hash.encode(e);
			self.ommers_hash.encode(e);
			self.beneficiary.encode(e);
			self.state_root.encode(e);
			self.transactions_root.encode(e);
			self.receipts_root.encode(e);
			self.log_bloom.encode(e);
			self.difficulty.encode(e);
			self.number.encode(e);
			self.gas_limit.encode(e);
			self.gas_used.encode(e);
			self.timestamp.encode(e);
			self.mix_hash.encode(e);
			self.nonce.encode(e);
		})
	}
}

