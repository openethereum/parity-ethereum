use util::hash::*;
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
}

impl Header {
	/// Create a new, default-valued, header.
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
		}
	}
}

impl Decodable for Header {
	fn decode<D>(decoder: &D) -> Result<Self, DecoderError> where D: Decoder {
		let d = try!(decoder.as_list());

		let mut blockheader = Header {
			parent_hash: try!(Decodable::decode(&d[0])),
			uncles_hash: try!(Decodable::decode(&d[1])),
			author: try!(Decodable::decode(&d[2])),
			state_root: try!(Decodable::decode(&d[3])),
			transactions_root: try!(Decodable::decode(&d[4])),
			receipts_root: try!(Decodable::decode(&d[5])),
			log_bloom: try!(Decodable::decode(&d[6])),
			difficulty: try!(Decodable::decode(&d[7])),
			number: try!(Decodable::decode(&d[8])),
			gas_limit: try!(Decodable::decode(&d[9])),
			gas_used: try!(Decodable::decode(&d[10])),
			timestamp: try!(Decodable::decode(&d[11])),
			extra_data: try!(Decodable::decode(&d[12])),
			seal: vec![],
		};

		for i in 13..d.len() {
			blockheader.seal.push(d[i].as_raw().to_vec());
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
