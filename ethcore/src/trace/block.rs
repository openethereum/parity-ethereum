use util::rlp::*;
use basic_types::LogBloom;
use super::Trace;

/// Traces created by transactions from the same block.
#[derive(Clone)]
pub struct BlockTraces(Vec<Trace>);

impl From<Vec<Trace>> for BlockTraces {
	fn from(traces: Vec<Trace>) -> Self {
		BlockTraces(traces)
	}
}

impl Into<Vec<Trace>> for BlockTraces {
	fn into(self) -> Vec<Trace> {
		self.0
	}
}

impl Decodable for BlockTraces {
	fn decode<D>(decoder: &D) -> Result<Self, DecoderError> where D: Decoder {
		let traces = try!(Decodable::decode(decoder));
		let block_traces = BlockTraces(traces);
		Ok(block_traces)
	}
}

impl Encodable for BlockTraces {
	fn rlp_append(&self, s: &mut RlpStream) {
		Encodable::rlp_append(&self.0, s)
	}
}

impl BlockTraces {
	/// Returns bloom of all traces in given block.
	pub fn bloom(&self) -> LogBloom {
		self.0.iter()
			.fold(LogBloom::default(), |acc, trace| acc | trace.bloom())
	}
}

