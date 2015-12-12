use util::hash::*;
use util::uint::*;
use util::rlp::*;
use util::sha3;

/// view onto block header rlp
pub struct BlockView<'a> {
	rlp: Rlp<'a>
}

impl<'a> BlockView<'a> {
	pub fn new(bytes: &'a [u8]) -> BlockView<'a> {
		BlockView {
			rlp: Rlp::new(bytes)
		}
	}

	pub fn new_from_rlp(rlp: Rlp<'a>) -> BlockView<'a> {
		BlockView {
			rlp: rlp
		}
	}

	pub fn parent_hash(&self) -> H256 { self.rlp.val_at(0) }
	pub fn uncles_hash(&self) -> H256 { self.rlp.val_at(1) }
	pub fn coinbase(&self) -> Address { self.rlp.val_at(2) }
	pub fn state_root(&self) -> H256 { self.rlp.val_at(3) }
	pub fn transactions_root(&self) -> H256 { self.rlp.val_at(4) }
	pub fn receipts_root(&self) -> H256 { self.rlp.val_at(5) }
	pub fn log_bloom(&self) -> H2048 { self.rlp.val_at(6) }
	pub fn difficulty(&self) -> U256 { self.rlp.val_at(7) }
	pub fn number(&self) -> U256 { self.rlp.val_at(8) }
	pub fn gas_limit(&self) -> U256 { self.rlp.val_at(9) }
	pub fn gas_usd(&self) -> U256 { self.rlp.val_at(10) }
	pub fn timestamp(&self) -> U256 { self.rlp.val_at(11) }
	pub fn mix_hash(&self) -> H256 { self.rlp.val_at(12) }
	pub fn nonce(&self) -> H64 { self.rlp.val_at(13) }
}

impl<'a> sha3::Hashable for BlockView<'a> {
	fn sha3(&self) -> H256 {
		self.rlp.raw().sha3()
	}
}

/// Data structure represening block header
/// similar to cpp-ethereum's BlockInfo
pub struct BlockHeader {
	parent_hash: H256,
	uncles_hash: H256,
	coinbase: Address,
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
			// return an error if d != 14
			let blockheader = BlockHeader {
				parent_hash: try!(Decodable::decode(&d[0])),
				uncles_hash: try!(Decodable::decode(&d[1])),
				coinbase: try!(Decodable::decode(&d[2])),
				state_root: try!(Decodable::decode(&d[3])),
				transactions_root: try!(Decodable::decode(&d[4])),
				receipts_root: try!(Decodable::decode(&d[5])),
				log_bloom: try!(Decodable::decode(&d[6])),
				difficulty: try!(Decodable::decode(&d[7])),
				number: try!(Decodable::decode(&d[8])),
				gas_limit: try!(Decodable::decode(&d[9])),
				gas_used: try!(Decodable::decode(&d[10])),
				timestamp: try!(Decodable::decode(&d[11])),
				mix_hash: try!(Decodable::decode(&d[12])),
				nonce: try!(Decodable::decode(&d[13]))
			};
			Ok(blockheader)
		})
	}
}

impl Encodable for BlockHeader {
	fn encode<E>(&self, encoder: &mut E) where E: Encoder {
		encoder.emit_list(| e | {
			self.parent_hash.encode(e);
			self.uncles_hash.encode(e);
			self.coinbase.encode(e);
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

#[cfg(test)]
mod tests {
	fn encoding_and_decoding() {
	}
}
