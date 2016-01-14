use util::*;
use verification::*;
use error::*;
use engine::Engine;
use sync::*;
use views::*;

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
		try!(verify_block_basic(bytes, self.engine.deref().deref()).map_err(|e| {
			warn!(target: "client", "Stage 1 block verification failed for {}\nError: {:?}", BlockView::new(&bytes).header_view().sha3(), e);
			e
		}));
		try!(verify_block_unordered(bytes, self.engine.deref().deref()).map_err(|e| {
			warn!(target: "client", "Stage 2 block verification failed for {}\nError: {:?}", BlockView::new(&bytes).header_view().sha3(), e);
			e
		}));
		try!(self.message_channel.send(UserMessage(SyncMessage::BlockVerified(bytes.to_vec()))).map_err(|e| Error::from(e)));
		Ok(())
	}
}

