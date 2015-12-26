use std::cell::Cell;
use util::hash::*;
use util::sha3::*;
use util::bytes::*;
use util::uint::*;
use util::rlp::*;

pub static ZERO_ADDRESS: Address = Address([0x00; 20]);
pub static ZERO_H256: H256 = H256([0x00; 32]);
pub static ZERO_LOGBLOOM: LogBloom = H2048([0x00; 256]);

pub type LogBloom = H2048;

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

	pub hash: Cell<Option<H256>>, //TODO: make this private
}

impl Header {
	pub fn new() -> Header {
		Header {
			parent_hash: ZERO_H256.clone(),
			timestamp: BAD_U256.clone(),
			number: ZERO_U256.clone(),
			author: ZERO_ADDRESS.clone(),

			transactions_root: ZERO_H256.clone(),
			uncles_hash: ZERO_H256.clone(),
			extra_data: vec![],

			state_root: ZERO_H256.clone(),
			receipts_root: ZERO_H256.clone(),
			log_bloom: ZERO_LOGBLOOM.clone(),
			gas_used: ZERO_U256.clone(),
			gas_limit: ZERO_U256.clone(),

			difficulty: ZERO_U256.clone(),
			seal: vec![],
			hash: Cell::new(None),
		}
	}

	pub fn hash(&self) -> H256 {
		let hash = self.hash.get();
		match hash {
			Some(h) => h,
			None => {
				let mut stream = RlpStream::new();
				stream.append(self);
				let h = stream.raw().sha3();
				self.hash.set(Some(h.clone()));
				h
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
			hash: Cell::new(Some(r.raw().sha3()))
		};

		for i in 13..r.item_count() {
			blockheader.seal.push(try!(r.val_at(i)))
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
				b.encode(e);
			}
		})
	}
}

#[cfg(test)]
mod tests {
}
