use util::uint::*;
use util::hash::*;
use util::rlp::*;

pub struct BlockDetails {
	pub number: U256,
	pub total_difficulty: U256,
	pub parent: H256,
	pub children: Vec<H256>
}

impl Decodable for BlockDetails {
	fn decode<D>(decoder: &D) -> Result<Self, DecoderError> where D: Decoder {
		decoder.read_list(| d | {
			let details = BlockDetails {
				number: try!(Decodable::decode(&d[0])),
				total_difficulty: try!(Decodable::decode(&d[1])),
				parent: try!(Decodable::decode(&d[2])),
				children: try!(Decodable::decode(&d[3]))
			};
			Ok(details)
		})
	}
}

impl Encodable for BlockDetails {
	fn encode<E>(&self, encoder: &mut E) where E: Encoder {
		encoder.emit_list(| e | {
			self.number.encode(e);
			self.total_difficulty.encode(e);
			self.parent.encode(e);
			self.children.encode(e);
		})
	}
}
