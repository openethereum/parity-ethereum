use util::rlp::*;
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

