use util::rlp::*;
use basic_types::LogBloom;
use super::Trace;

/// Traces created by transactions from the same block.
#[derive(Clone)]
pub struct BlockTraces {
	/// Traces.
	pub traces: Vec<Trace>,
}

impl Decodable for BlockTraces {
	fn decode<D>(decoder: &D) -> Result<Self, DecoderError> where D: Decoder {
		let traces = try!(Decodable::decode(decoder));
		let block_traces = BlockTraces {
			traces: traces
		};
		Ok(block_traces)
	}
}

impl Encodable for BlockTraces {
	fn rlp_append(&self, s: &mut RlpStream) {
		Encodable::rlp_append(&self.traces, s)
	}
}

impl BlockTraces {
	/// Returns bloom of all traces in given block.
	pub fn bloom(&self) -> LogBloom {
		self.traces.iter()
			.fold(LogBloom::default(), |acc, trace| acc | trace.bloom())
	}
}

