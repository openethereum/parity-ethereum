use util::*;
use verification::*;
use error::*;
use engine::Engine;
use sync::*;

/// A queue of blocks. Sits between network or other I/O and the BlockChain.
/// Sorts them ready for blockchain insertion.
pub struct BlockQueue {
	engine: Arc<Box<Engine>>,
	message_channel: IoChannel<NetSyncMessage>
}

impl BlockQueue {
	/// Creates a new queue instance.
	pub fn new(engine: Arc<Box<Engine>>, message_channel: IoChannel<NetSyncMessage>) -> BlockQueue {
		BlockQueue {
			engine: engine,
			message_channel: message_channel
		}
	}

	/// Clear the queue and stop verification activity.
	pub fn clear(&mut self) {
	}

	/// Add a block to the queue.
	pub fn import_block(&mut self, bytes: &[u8]) -> ImportResult {
		try!(verify_block_basic(bytes, self.engine.deref().deref()));
		try!(verify_block_unordered(bytes, self.engine.deref().deref()));
		try!(self.message_channel.send(UserMessage(SyncMessage::BlockVerified(bytes.to_vec()))).map_err(|e| Error::from(e)));
		Ok(())
	}
}

