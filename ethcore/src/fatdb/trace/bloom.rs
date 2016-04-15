use bloomchain::Bloom;
use bloomchain::group::{BloomGroup, GroupPosition};
use util::rlp::*;
use util::{H256, H2048};
use basic_types::LogBloom;

pub struct TraceBloom(LogBloom);

impl From<LogBloom> for TraceBloom {
	fn from(bloom: LogBloom) -> TraceBloom {
		TraceBloom(bloom)
	}
}

impl From<Bloom> for TraceBloom {
	fn from(bloom: Bloom) -> TraceBloom {
		let bytes: [u8; 256] = bloom.into();
		TraceBloom(LogBloom::from(bytes))
	}
}

impl Into<Bloom> for TraceBloom {
	fn into(self) -> Bloom {
		let log = self.0;
		Bloom::from(log.0)
	}
}

pub struct TraceBloomGroup {
	blooms: Vec<TraceBloom>,
}

impl From<BloomGroup> for TraceBloomGroup {
	fn from(group: BloomGroup) -> Self {
		let blooms = group.blooms
			.into_iter()
			.map(From::from)
			.collect();

		TraceBloomGroup {
			blooms: blooms
		}
	}
}

impl Into<BloomGroup> for TraceBloomGroup {
	fn into(self) -> BloomGroup {
		let blooms = self.blooms
			.into_iter()
			.map(Into::into)
			.collect();

		BloomGroup {
			blooms: blooms
		}
	}
}

impl Decodable for TraceBloom {
	fn decode<D>(decoder: &D) -> Result<Self, DecoderError> where D: Decoder {
		Decodable::decode(decoder).map(TraceBloom)
	}
}

impl Encodable for TraceBloom {
	fn rlp_append(&self, s: &mut RlpStream) {
		Encodable::rlp_append(&self.0, s)
	}
}

impl Decodable for TraceBloomGroup {
	fn decode<D>(decoder: &D) -> Result<Self, DecoderError> where D: Decoder {
		let blooms = try!(Decodable::decode(decoder));
		let group = TraceBloomGroup {
			blooms: blooms
		};
		Ok(group)
	}
}

impl Encodable for TraceBloomGroup {
	fn rlp_append(&self, s: &mut RlpStream) {
		Encodable::rlp_append(&self.blooms, s)
	}
}

/// Represents BloomGroup position in database.
#[derive(PartialEq, Eq, Hash)]
pub struct TraceGroupPosition {
	/// Bloom level.
	pub level: usize,
	/// Group index.
	pub index: usize,
}

impl From<GroupPosition> for TraceGroupPosition {
	fn from(p: GroupPosition) -> Self {
		TraceGroupPosition {
			level: p.level,
			index: p.index,
		}
	}
}

impl TraceGroupPosition {
	pub fn hash(&self) -> H256 {
		use std::ptr;
		let mut hash = H256::default();
		unsafe {
			ptr::copy(&[self.level, self.index] as *const usize as *const u8, hash.as_mut_ptr(), 16);
		}
		hash
	}
}
