// Copyright 2015, 2016 Ethcore (UK) Ltd.
// This file is part of Parity.

// Parity is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity.  If not, see <http://www.gnu.org/licenses/>.

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
use client::BlockStatus;
use util::panics::*;

/// Block queue status
#[derive(Debug)]
pub struct BlockQueueInfo {
	/// Number of queued blocks pending verification
	pub unverified_queue_size: usize,
	/// Number of verified queued blocks pending import
	pub verified_queue_size: usize,
	/// Number of blocks being verified
	pub verifying_queue_size: usize,
}

impl BlockQueueInfo {
	/// The total size of the queues.
	pub fn total_queue_size(&self) -> usize { self.unverified_queue_size + self.verified_queue_size + self.verifying_queue_size }

	/// The size of the unverified and verifying queues.
	pub fn incomplete_queue_size(&self) -> usize { self.unverified_queue_size + self.verifying_queue_size }

	/// Indicates that queue is full
	pub fn is_full(&self) -> bool {
		self.unverified_queue_size + self.verified_queue_size + self.verifying_queue_size > MAX_UNVERIFIED_QUEUE_SIZE
	}

	/// Indicates that queue is empty
	pub fn is_empty(&self) -> bool {
		self.unverified_queue_size + self.verified_queue_size + self.verifying_queue_size == 0
	}
}

/// A queue of blocks. Sits between network or other I/O and the BlockChain.
/// Sorts them ready for blockchain insertion.
pub struct BlockQueue {
	panic_handler: Arc<PanicHandler>,
	engine: Arc<Box<Engine>>,
	more_to_verify: Arc<Condvar>,
	verification: Arc<Mutex<Verification>>,
	verifiers: Vec<JoinHandle<()>>,
	deleting: Arc<AtomicBool>,
	ready_signal: Arc<QueueSignal>,
	empty: Arc<Condvar>,
	processing: RwLock<HashSet<H256>>
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

const MAX_UNVERIFIED_QUEUE_SIZE: usize = 50000;

impl BlockQueue {
	/// Creates a new queue instance.
	pub fn new(engine: Arc<Box<Engine>>, message_channel: IoChannel<NetSyncMessage>) -> BlockQueue {
		let verification = Arc::new(Mutex::new(Verification::default()));
		let more_to_verify = Arc::new(Condvar::new());
		let ready_signal = Arc::new(QueueSignal { signalled: AtomicBool::new(false), message_channel: message_channel });
		let deleting = Arc::new(AtomicBool::new(false));
		let empty = Arc::new(Condvar::new());
		let panic_handler = PanicHandler::new_arc();

		let mut verifiers: Vec<JoinHandle<()>> = Vec::new();
		let thread_count = max(::num_cpus::get(), 3) - 2;
		for i in 0..thread_count {
			let verification = verification.clone();
			let engine = engine.clone();
			let more_to_verify = more_to_verify.clone();
			let ready_signal = ready_signal.clone();
			let empty = empty.clone();
			let deleting = deleting.clone();
			let panic_handler = panic_handler.clone();
			verifiers.push(
				thread::Builder::new()
				.name(format!("Verifier #{}", i))
				.spawn(move || {
					panic_handler.catch_panic(move || {
					  BlockQueue::verify(verification, engine, more_to_verify, ready_signal, deleting, empty)
					}).unwrap()
				})
				.expect("Error starting block verification thread")
			);
		}
		BlockQueue {
			engine: engine,
			panic_handler: panic_handler,
			ready_signal: ready_signal.clone(),
			more_to_verify: more_to_verify.clone(),
			verification: verification.clone(),
			verifiers: verifiers,
			deleting: deleting.clone(),
			processing: RwLock::new(HashSet::new()),
			empty: empty.clone(),
		}
	}

	fn verify(verification: Arc<Mutex<Verification>>, engine: Arc<Box<Engine>>, wait: Arc<Condvar>, ready: Arc<QueueSignal>, deleting: Arc<AtomicBool>, empty: Arc<Condvar>) {
		while !deleting.load(AtomicOrdering::Relaxed) {
			{
				let mut lock = verification.lock().unwrap();

				if lock.unverified.is_empty() && lock.verifying.is_empty() {
					empty.notify_all();
				}

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
		verification.verified.clear();
		self.processing.write().unwrap().clear();
	}

	/// Wait for queue to be empty
	pub fn flush(&mut self) {
		let mut verification = self.verification.lock().unwrap();
		while !verification.unverified.is_empty() || !verification.verifying.is_empty() {
			verification = self.empty.wait(verification).unwrap();
		}
	}

	/// Check if the block is currently in the queue
	pub fn block_status(&self, hash: &H256) -> BlockStatus {
		if self.processing.read().unwrap().contains(&hash) {
			return BlockStatus::Queued;
		}
		if self.verification.lock().unwrap().bad.contains(&hash) {
			return BlockStatus::Bad;
		}
		BlockStatus::Unknown
	}

	/// Add a block to the queue.
	pub fn import_block(&mut self, bytes: Bytes) -> ImportResult {
		let header = BlockView::new(&bytes).header();
		let h = header.hash();
		if self.processing.read().unwrap().contains(&h) {
			return Err(ImportError::AlreadyQueued);
		}
		{
			let mut verification = self.verification.lock().unwrap();
			if verification.bad.contains(&h) {
				return Err(ImportError::Bad(None));
			}

			if verification.bad.contains(&header.parent_hash) {
				verification.bad.insert(h.clone());
				return Err(ImportError::Bad(None));
			}
		}

		match verify_block_basic(&header, &bytes, self.engine.deref().deref()) {
			Ok(()) => {
				self.processing.write().unwrap().insert(h.clone());
				self.verification.lock().unwrap().unverified.push_back(UnVerifiedBlock { header: header, bytes: bytes });
				self.more_to_verify.notify_all();
				Ok(h)
			},
			Err(err) => {
				warn!(target: "client", "Stage 1 block verification failed for {}\nError: {:?}", BlockView::new(&bytes).header_view().sha3(), err);
				self.verification.lock().unwrap().bad.insert(h.clone());
				Err(From::from(err))
			}
		}
	}

	/// Mark given block and all its children as bad. Stops verification.
	pub fn mark_as_bad(&mut self, hash: &H256) {
		let mut verification_lock = self.verification.lock().unwrap();
		let mut verification = verification_lock.deref_mut();
		verification.bad.insert(hash.clone());
		self.processing.write().unwrap().remove(&hash);
		let mut new_verified = VecDeque::new();
		for block in verification.verified.drain(..) {
			if verification.bad.contains(&block.header.parent_hash) {
				verification.bad.insert(block.header.hash());
				self.processing.write().unwrap().remove(&block.header.hash());
			}
			else {
				new_verified.push_back(block);
			}
		}
		verification.verified = new_verified;
	}

	/// Mark given block as processed
	pub fn mark_as_good(&mut self, hashes: &[H256]) {
		let mut processing = self.processing.write().unwrap();
		for h in hashes {
			processing.remove(&h);
		}
	}

	/// Removes up to `max` verified blocks from the queue
	pub fn drain(&mut self, max: usize) -> Vec<PreVerifiedBlock> {
		let mut verification = self.verification.lock().unwrap();
		let count = min(max, verification.verified.len());
		let mut result = Vec::with_capacity(count);
		for _ in 0..count {
			let block = verification.verified.pop_front().unwrap();
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
			verified_queue_size: verification.verified.len(),
			unverified_queue_size: verification.unverified.len(),
			verifying_queue_size: verification.verifying.len(),
		}
	}
}

impl MayPanic for BlockQueue {
	fn on_panic<F>(&self, closure: F) where F: OnPanicListener {
		self.panic_handler.on_panic(closure);
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
	use tests::helpers::*;
	use error::*;
	use views::*;

	fn get_test_queue() -> BlockQueue {
		let spec = get_test_spec();
		let engine = spec.to_engine().unwrap();
		BlockQueue::new(Arc::new(engine), IoChannel::disconnected())
	}

	#[test]
	fn can_be_created() {
		// TODO better test
		let spec = Spec::new_test();
		let engine = spec.to_engine().unwrap();
		let _ = BlockQueue::new(Arc::new(engine), IoChannel::disconnected());
	}

	#[test]
	fn can_import_blocks() {
		let mut queue = get_test_queue();
		if let Err(e) = queue.import_block(get_good_dummy_block()) {
			panic!("error importing block that is valid by definition({:?})", e);
		}
	}

	#[test]
	fn returns_error_for_duplicates() {
		let mut queue = get_test_queue();
		if let Err(e) = queue.import_block(get_good_dummy_block()) {
			panic!("error importing block that is valid by definition({:?})", e);
		}

		let duplicate_import = queue.import_block(get_good_dummy_block());
		match duplicate_import {
			Err(e) => {
				match e {
					ImportError::AlreadyQueued => {},
					_ => { panic!("must return AlreadyQueued error"); }
				}
			}
			Ok(_) => { panic!("must produce error"); }
		}
	}

	#[test]
	fn returns_ok_for_drained_duplicates() {
		let mut queue = get_test_queue();
		let block = get_good_dummy_block();
		let hash = BlockView::new(&block).header().hash().clone();
		if let Err(e) = queue.import_block(block) {
			panic!("error importing block that is valid by definition({:?})", e);
		}
		queue.flush();
		queue.drain(10);
		queue.mark_as_good(&[ hash ]);

		if let Err(e) = queue.import_block(get_good_dummy_block()) {
			panic!("error importing block that has already been drained ({:?})", e);
		}
	}

	#[test]
	fn returns_empty_once_finished() {
		let mut queue = get_test_queue();
		queue.import_block(get_good_dummy_block()).expect("error importing block that is valid by definition");
		queue.flush();
		queue.drain(1);

		assert!(queue.queue_info().is_empty());
	}
}
