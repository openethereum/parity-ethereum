//! A queue of blocks. Sits between network or other I/O and the BlockChain.
//! Sorts them ready for blockchain insertion.
use std::thread::{JoinHandle, self};
use std::sync::atomic::{AtomicBool, Ordering as AtomicOrdering};
use util::*;
use verification::*;
use error::*;
use engine::Engine;
use views::*;
use header::*;
use service::*;

/// Block queue status
#[derive(Debug)]
pub struct BlockQueueInfo {
	/// Indicates that queue is full
	pub full: bool,
	/// Number of queued blocks pending verification
	pub unverified_queue_size: usize,
	/// Number of verified queued blocks pending import
	pub verified_queue_size: usize,
}

/// A queue of blocks. Sits between network or other I/O and the BlockChain.
/// Sorts them ready for blockchain insertion.
pub struct BlockQueue {
	engine: Arc<Box<Engine>>,
	more_to_verify: Arc<Condvar>,
	verification: Arc<Mutex<Verification>>,
	verifiers: Vec<JoinHandle<()>>,
	deleting: Arc<AtomicBool>,
	ready_signal: Arc<QueueSignal>,
	processing: HashSet<H256>
}

struct UnVerifiedBlock {
	header: Header,
	bytes: Bytes,
}

struct VerifyingBlock {
	hash: H256,
	block: Option<PreVerifiedBlock>,
}

struct QueueSignal {
	signalled: AtomicBool,
	message_channel: IoChannel<NetSyncMessage>,
}

impl QueueSignal {
	fn set(&self) {
		if self.signalled.compare_and_swap(false, true, AtomicOrdering::Relaxed) == false {
			self.message_channel.send(UserMessage(SyncMessage::BlockVerified)).expect("Error sending BlockVerified message");
		}
	}
	fn reset(&self) {
		self.signalled.store(false, AtomicOrdering::Relaxed);
	}
}

#[derive(Default)]
struct Verification {
	unverified: VecDeque<UnVerifiedBlock>,
	verified: VecDeque<PreVerifiedBlock>,
	verifying: VecDeque<VerifyingBlock>,
	bad: HashSet<H256>,
}

impl BlockQueue {
	/// Creates a new queue instance.
	pub fn new(engine: Arc<Box<Engine>>, message_channel: IoChannel<NetSyncMessage>) -> BlockQueue {
		let verification = Arc::new(Mutex::new(Verification::default()));
		let more_to_verify = Arc::new(Condvar::new());
		let ready_signal = Arc::new(QueueSignal { signalled: AtomicBool::new(false), message_channel: message_channel });
		let deleting = Arc::new(AtomicBool::new(false));

		let mut verifiers: Vec<JoinHandle<()>> = Vec::new();
		let thread_count = max(::num_cpus::get(), 3) - 2;
		for i in 0..thread_count {
			let verification = verification.clone();
			let engine = engine.clone();
			let more_to_verify = more_to_verify.clone();
			let ready_signal = ready_signal.clone();
			let deleting = deleting.clone();
			verifiers.push(thread::Builder::new().name(format!("Verifier #{}", i)).spawn(move || BlockQueue::verify(verification, engine, more_to_verify, ready_signal,  deleting))
				.expect("Error starting block verification thread"));
		}
		BlockQueue {
			engine: engine,
			ready_signal: ready_signal.clone(),
			more_to_verify: more_to_verify.clone(),
			verification: verification.clone(),
			verifiers: verifiers,
			deleting: deleting.clone(),
			processing: HashSet::new(),
		}
	}

	fn verify(verification: Arc<Mutex<Verification>>, engine: Arc<Box<Engine>>, wait: Arc<Condvar>, ready: Arc<QueueSignal>, deleting: Arc<AtomicBool>) {
		while !deleting.load(AtomicOrdering::Relaxed) {
			{
				let mut lock = verification.lock().unwrap();
				while lock.unverified.is_empty() && !deleting.load(AtomicOrdering::Relaxed) {
					lock = wait.wait(lock).unwrap();
				}
				
				if deleting.load(AtomicOrdering::Relaxed) {
					return;
				}
			}

			let block = {
				let mut v = verification.lock().unwrap();
				if v.unverified.is_empty() {
					continue;
				}
				let block = v.unverified.pop_front().unwrap();
				v.verifying.push_back(VerifyingBlock{ hash: block.header.hash(), block: None });
				block
			};

			let block_hash = block.header.hash();
			match verify_block_unordered(block.header, block.bytes, engine.deref().deref()) {
				Ok(verified) => {
					let mut v = verification.lock().unwrap();
					for e in &mut v.verifying {
						if e.hash == block_hash {
							e.block = Some(verified);
							break;
						}
					}
					if !v.verifying.is_empty() && v.verifying.front().unwrap().hash == block_hash {
						// we're next!
						let mut vref = v.deref_mut();
						BlockQueue::drain_verifying(&mut vref.verifying, &mut vref.verified, &mut vref.bad);
						ready.set();
					}
				},
				Err(err) => {
					let mut v = verification.lock().unwrap();
					warn!(target: "client", "Stage 2 block verification failed for {}\nError: {:?}", block_hash, err);
					v.bad.insert(block_hash.clone());
					v.verifying.retain(|e| e.hash != block_hash);
					let mut vref = v.deref_mut();
					BlockQueue::drain_verifying(&mut vref.verifying, &mut vref.verified, &mut vref.bad);
					ready.set();
				}
			}
		}
	}

	fn drain_verifying(verifying: &mut VecDeque<VerifyingBlock>, verified: &mut VecDeque<PreVerifiedBlock>, bad: &mut HashSet<H256>) {
		while !verifying.is_empty() && verifying.front().unwrap().block.is_some() {
			let block = verifying.pop_front().unwrap().block.unwrap();
			if bad.contains(&block.header.parent_hash) {
				bad.insert(block.header.hash());
			}
			else {
				verified.push_back(block);
			}
		}
	}

	/// Clear the queue and stop verification activity.
	pub fn clear(&mut self) {
		let mut verification = self.verification.lock().unwrap();
		verification.unverified.clear();
		verification.verifying.clear();
	}

	/// Add a block to the queue.
	pub fn import_block(&mut self, bytes: Bytes) -> ImportResult {
		let header = BlockView::new(&bytes).header();
		if self.processing.contains(&header.hash()) {
			return Err(ImportError::AlreadyQueued);
		}
		{
			let mut verification = self.verification.lock().unwrap();
			if verification.bad.contains(&header.hash()) {
				return Err(ImportError::Bad(None));
			}

			if verification.bad.contains(&header.parent_hash) {
				verification.bad.insert(header.hash());
				return Err(ImportError::Bad(None));
			}
		}

		match verify_block_basic(&header, &bytes, self.engine.deref().deref()) {
			Ok(()) => {
				self.processing.insert(header.hash());
				self.verification.lock().unwrap().unverified.push_back(UnVerifiedBlock { header: header, bytes: bytes });
				self.more_to_verify.notify_all();
			},
			Err(err) => {
				warn!(target: "client", "Stage 1 block verification failed for {}\nError: {:?}", BlockView::new(&bytes).header_view().sha3(), err);
				self.verification.lock().unwrap().bad.insert(header.hash());
			}
		}
		Ok(())
	}

	/// Mark given block and all its children as bad. Stops verification.
	pub fn mark_as_bad(&mut self, hash: &H256) {
		let mut verification_lock = self.verification.lock().unwrap();
		let mut verification = verification_lock.deref_mut();
		verification.bad.insert(hash.clone());
		let mut new_verified = VecDeque::new();
		for block in verification.verified.drain(..) {
			if verification.bad.contains(&block.header.parent_hash) {
				verification.bad.insert(block.header.hash());
			}
			else {
				new_verified.push_back(block);
			}
		}
		verification.verified = new_verified;
	}

	/// Removes up to `max` verified blocks from the queue
	pub fn drain(&mut self, max: usize) -> Vec<PreVerifiedBlock> {
		let mut verification = self.verification.lock().unwrap();
		let count = min(max, verification.verified.len());
		let mut result = Vec::with_capacity(count);
		for _ in 0..count {
			let block = verification.verified.pop_front().unwrap();
			self.processing.remove(&block.header.hash());
			result.push(block);
		}
		self.ready_signal.reset();
		if !verification.verified.is_empty() {
			self.ready_signal.set();
		}
		result
	}

	/// Get queue status.
	pub fn queue_info(&self) -> BlockQueueInfo {
		let verification = self.verification.lock().unwrap();
		BlockQueueInfo {
			full: false,
			verified_queue_size: verification.verified.len(),
			unverified_queue_size: verification.unverified.len(),
		}
	}
}

impl Drop for BlockQueue {
	fn drop(&mut self) {
		self.clear();
		self.deleting.store(true, AtomicOrdering::Relaxed);
		self.more_to_verify.notify_all();
		for t in self.verifiers.drain(..) {
			t.join().unwrap();
		}
	}
}

#[cfg(test)]
mod tests {
	use util::*;
	use spec::*;
	use block_queue::*;

	#[test]
	fn test_block_queue() {
		// TODO better test
		let spec = Spec::new_test();
		let engine = spec.to_engine().unwrap();
		let _ = BlockQueue::new(Arc::new(engine), IoChannel::disconnected());
	}
}
