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
	message_channel: IoChannel<NetSyncMessage>,
	bad: HashSet<H256>,
}

impl BlockQueue {
	/// Creates a new queue instance.
	pub fn new(engine: Arc<Box<Engine>>, message_channel: IoChannel<NetSyncMessage>) -> BlockQueue {
		BlockQueue {
			engine: engine,
			message_channel: message_channel,
			bad: HashSet::new(),
		}
	}

	/// Clear the queue and stop verification activity.
	pub fn clear(&mut self) {
	}

	/// Add a block to the queue.
	pub fn import_block(&mut self, bytes: &[u8]) -> ImportResult {
		let header = BlockView::new(bytes).header();
		if self.bad.contains(&header.hash()) {
			return Err(ImportError::Bad(None));
		}

		if self.bad.contains(&header.parent_hash) {
			self.bad.insert(header.hash());
			return Err(ImportError::Bad(None));
		}

		try!(verify_block_basic(&header, bytes, self.engine.deref().deref()).map_err(|e| {
			warn!(target: "client", "Stage 1 block verification failed for {}\nError: {:?}", BlockView::new(&bytes).header_view().sha3(), e);
			e
		}));
		try!(verify_block_unordered(&header, bytes, self.engine.deref().deref()).map_err(|e| {
			warn!(target: "client", "Stage 2 block verification failed for {}\nError: {:?}", BlockView::new(&bytes).header_view().sha3(), e);
			e
		}));
		try!(self.message_channel.send(UserMessage(SyncMessage::BlockVerified(bytes.to_vec()))).map_err(|e| Error::from(e)));
		Ok(())
	}

	pub fn mark_as_bad(&mut self, hash: &H256) {
		self.bad.insert(hash.clone());
		//TODO: walk the queue
	}
}

