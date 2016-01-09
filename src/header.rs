use std::cell::RefCell;
use util::hash::*;
use util::sha3::*;
use util::bytes::*;
use util::uint::*;
use util::rlp::*;

/// Type for a 2048-bit log-bloom, as used by our blocks.
pub type LogBloom = H2048;

/// Constant address for point 0. Often used as a default.
pub static ZERO_ADDRESS: Address = Address([0x00; 20]);
/// Constant 256-bit datum for 0. Often used as a default.
pub static ZERO_H256: H256 = H256([0x00; 32]);
/// Constant 2048-bit datum for 0. Often used as a default.
pub static ZERO_LOGBLOOM: LogBloom = H2048([0x00; 256]);

/// A block header.
///
/// Reflects the specific RLP fields of a block in the chain with additional room for the seal
/// which is non-specific.
///
/// Doesn't do all that much on its own.
#[derive(Debug)]
pub struct Header {
	pub parent_hash: H256,
	pub timestamp: U256,
	pub number: U256,
	pub author: Address,

	pub transactions_root: H256,
	pub uncles_hash: H256,
	pub extra_data: Bytes,

	pub state_root: H256,
	pub receipts_root: H256,
	pub log_bloom: LogBloom,
	pub gas_used: U256,
	pub gas_limit: U256,

	pub difficulty: U256,
	pub seal: Vec<Bytes>,

	pub hash: RefCell<Option<H256>>, //TODO: make this private
}

impl Header {
	/// Create a new, default-valued, header.
	pub fn new() -> Header {
		Header {
			parent_hash: ZERO_H256.clone(),
			timestamp: BAD_U256,
			number: ZERO_U256,
			author: ZERO_ADDRESS.clone(),

			transactions_root: SHA3_NULL_RLP,
			uncles_hash: SHA3_EMPTY_LIST_RLP,
			extra_data: vec![],

			state_root: SHA3_NULL_RLP,
			receipts_root: SHA3_NULL_RLP,
			log_bloom: ZERO_LOGBLOOM.clone(),
			gas_used: ZERO_U256,
			gas_limit: ZERO_U256,

			difficulty: ZERO_U256,
			seal: vec![],
			hash: RefCell::new(None),
		}
	}

	pub fn hash(&self) -> H256 {
		let mut hash = self.hash.borrow_mut();
		match &mut *hash {
			&mut Some(ref h) => h.clone(),
			hash @ &mut None => {
				let mut stream = RlpStream::new();
				stream.append(self);
				let h = stream.as_raw().sha3();
				*hash = Some(h.clone());
				h.clone()
			}
		}
	}
}

impl Decodable for Header {
	fn decode<D>(decoder: &D) -> Result<Self, DecoderError> where D: Decoder {
		let r = decoder.as_rlp();

		let mut blockheader = Header {
			parent_hash: try!(r.val_at(0)),
			uncles_hash: try!(r.val_at(1)),
			author: try!(r.val_at(2)),
			state_root: try!(r.val_at(3)),
			transactions_root: try!(r.val_at(4)),
			receipts_root: try!(r.val_at(5)),
			log_bloom: try!(r.val_at(6)),
			difficulty: try!(r.val_at(7)),
			number: try!(r.val_at(8)),
			gas_limit: try!(r.val_at(9)),
			gas_used: try!(r.val_at(10)),
			timestamp: try!(r.val_at(11)),
			extra_data: try!(r.val_at(12)),
			seal: vec![],
			hash: RefCell::new(Some(r.as_raw().sha3()))
		};

		for i in 13..r.item_count() {
			blockheader.seal.push(try!(r.at(i)).as_raw().to_vec())
		}

		Ok(blockheader)
	}
}

impl Encodable for Header {
	fn encode<E>(&self, encoder: &mut E) where E: Encoder {
		encoder.emit_list(| e | {
			self.parent_hash.encode(e);
			self.uncles_hash.encode(e);
			self.author.encode(e);
			self.state_root.encode(e);
			self.transactions_root.encode(e);
			self.receipts_root.encode(e);
			self.log_bloom.encode(e);
			self.difficulty.encode(e);
			self.number.encode(e);
			self.gas_limit.encode(e);
			self.gas_used.encode(e);
			self.timestamp.encode(e);
			self.extra_data.encode(e);

			for b in self.seal.iter() {
				e.emit_raw(&b);
			}
		})
	}
}
/*
trait RlpStandard {
	fn append(&self, s: &mut RlpStream);
}

impl RlpStandard for Header {
	fn append(&self, s: &mut RlpStream) {
		s.append_list(13);
		s.append(self.parent_hash);
		s.append_raw(self.seal[0]);
		s.append_standard(self.x);
	}
	fn populate(&mut self, s: &Rlp) {
	}
}

impl RlpStream {
	fn append_standard<O>(&mut self, o: &O) where O: RlpStandard {
		o.append(self);
	}
}
*/

#[cfg(test)]
mod tests {
}
