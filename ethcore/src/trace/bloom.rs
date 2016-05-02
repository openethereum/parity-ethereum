use bloomchain::Bloom;
use bloomchain::group::{BloomGroup, GroupPosition};
use util::rlp::*;
use basic_types::LogBloom;

/// Helper structure representing bloom of the trace.
#[derive(Clone)]
pub struct BlockTracesBloom(LogBloom);

impl From<LogBloom> for BlockTracesBloom {
	fn from(bloom: LogBloom) -> BlockTracesBloom {
		BlockTracesBloom(bloom)
	}
}

impl From<Bloom> for BlockTracesBloom {
	fn from(bloom: Bloom) -> BlockTracesBloom {
		let bytes: [u8; 256] = bloom.into();
		BlockTracesBloom(LogBloom::from(bytes))
	}
}

impl Into<Bloom> for BlockTracesBloom {
	fn into(self) -> Bloom {
		let log = self.0;
		Bloom::from(log.0)
	}
}

/// Represents group of X consecutive blooms.
#[derive(Clone)]
pub struct BlockTracesBloomGroup {
	blooms: Vec<BlockTracesBloom>,
}

impl From<BloomGroup> for BlockTracesBloomGroup {
	fn from(group: BloomGroup) -> Self {
		let blooms = group.blooms
			.into_iter()
			.map(From::from)
			.collect();

		BlockTracesBloomGroup {
			blooms: blooms
		}
	}
}

impl Into<BloomGroup> for BlockTracesBloomGroup {
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

impl Decodable for BlockTracesBloom {
	fn decode<D>(decoder: &D) -> Result<Self, DecoderError> where D: Decoder {
		Decodable::decode(decoder).map(BlockTracesBloom)
	}
}

impl Encodable for BlockTracesBloom {
	fn rlp_append(&self, s: &mut RlpStream) {
		Encodable::rlp_append(&self.0, s)
	}
}

impl Decodable for BlockTracesBloomGroup {
	fn decode<D>(decoder: &D) -> Result<Self, DecoderError> where D: Decoder {
		let blooms = try!(Decodable::decode(decoder));
		let group = BlockTracesBloomGroup {
			blooms: blooms
		};
		Ok(group)
	}
}

impl Encodable for BlockTracesBloomGroup {
	fn rlp_append(&self, s: &mut RlpStream) {
		Encodable::rlp_append(&self.blooms, s)
	}
}

/// Represents `BloomGroup` position in database.
#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub struct TraceGroupPosition {
	/// Bloom level.
	pub level: u8,
	/// Group index.
	pub index: u32,
}

impl From<GroupPosition> for TraceGroupPosition {
	fn from(p: GroupPosition) -> Self {
		TraceGroupPosition {
			level: p.level as u8,
			index: p.index as u32,
		}
	}
}
