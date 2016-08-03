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

//! A queue of blocks. Sits between network or other I/O and the `BlockChain`.
//! Sorts them ready for blockchain insertion.
use std::thread::{JoinHandle, self};
use std::sync::atomic::{AtomicBool, Ordering as AtomicOrdering};
use std::sync::{Condvar as SCondvar, Mutex as SMutex};
use util::*;
use io::*;
use verification::*;
use error::*;
use engines::Engine;
use views::*;
use header::*;
use service::*;
use client::BlockStatus;
use util::panics::*;

pub use types::block_queue_info::BlockQueueInfo;

known_heap_size!(0, UnverifiedBlock, VerifyingBlock, PreverifiedBlock);

const MIN_MEM_LIMIT: usize = 16384;
const MIN_QUEUE_LIMIT: usize = 512;

/// Block queue configuration
#[derive(Debug, PartialEq)]
pub struct BlockQueueConfig {
	/// Maximum number of blocks to keep in unverified queue.
	/// When the limit is reached, is_full returns true.
	pub max_queue_size: usize,
	/// Maximum heap memory to use.
	/// When the limit is reached, is_full returns true.
	pub max_mem_use: usize,
}

impl Default for BlockQueueConfig {
	fn default() -> Self {
		BlockQueueConfig {
			max_queue_size: 30000,
			max_mem_use: 50 * 1024 * 1024,
		}
	}
}


impl BlockQueueInfo {
	/// The total size of the queues.
	pub fn total_queue_size(&self) -> usize { self.unverified_queue_size + self.verified_queue_size + self.verifying_queue_size }

	/// The size of the unverified and verifying queues.
	pub fn incomplete_queue_size(&self) -> usize { self.unverified_queue_size + self.verifying_queue_size }

	/// Indicates that queue is full
	pub fn is_full(&self) -> bool {
		self.unverified_queue_size + self.verified_queue_size + self.verifying_queue_size > self.max_queue_size ||
			self.mem_used > self.max_mem_use
	}

	/// Indicates that queue is empty
	pub fn is_empty(&self) -> bool {
		self.unverified_queue_size + self.verified_queue_size + self.verifying_queue_size == 0
	}
}

/// A queue of blocks. Sits between network or other I/O and the `BlockChain`.
/// Sorts them ready for blockchain insertion.
pub struct BlockQueue {
	panic_handler: Arc<PanicHandler>,
	engine: Arc<Box<Engine>>,
	more_to_verify: Arc<SCondvar>,
	verification: Arc<Verification>,
	verifiers: Vec<JoinHandle<()>>,
	deleting: Arc<AtomicBool>,
	ready_signal: Arc<QueueSignal>,
	empty: Arc<SCondvar>,
	processing: RwLock<HashSet<H256>>,
	max_queue_size: usize,
	max_mem_use: usize,
}

struct UnverifiedBlock {
	header: Header,
	bytes: Bytes,
}

struct VerifyingBlock {
	hash: H256,
	block: Option<PreverifiedBlock>,
}

struct QueueSignal {
	deleting: Arc<AtomicBool>,
	signalled: AtomicBool,
	message_channel: IoChannel<ClientIoMessage>,
}

impl QueueSignal {
	#[cfg_attr(feature="dev", allow(bool_comparison))]
	fn set(&self) {
		// Do not signal when we are about to close
		if self.deleting.load(AtomicOrdering::Relaxed) {
			return;
		}

		if self.signalled.compare_and_swap(false, true, AtomicOrdering::Relaxed) == false {
			if let Err(e) = self.message_channel.send(ClientIoMessage::BlockVerified) {
				debug!("Error sending BlockVerified message: {:?}", e);
			}
		}
	}

	fn reset(&self) {
		self.signalled.store(false, AtomicOrdering::Relaxed);
	}
}

struct Verification {
	// All locks must be captured in the order declared here.
	unverified: Mutex<VecDeque<UnverifiedBlock>>,
	verified: Mutex<VecDeque<PreverifiedBlock>>,
	verifying: Mutex<VecDeque<VerifyingBlock>>,
	bad: Mutex<HashSet<H256>>,
	more_to_verify: SMutex<()>,
	empty: SMutex<()>,
}

impl BlockQueue {
	/// Creates a new queue instance.
	pub fn new(config: BlockQueueConfig, engine: Arc<Box<Engine>>, message_channel: IoChannel<ClientIoMessage>) -> BlockQueue {
		let verification = Arc::new(Verification {
			unverified: Mutex::new(VecDeque::new()),
			verified: Mutex::new(VecDeque::new()),
			verifying: Mutex::new(VecDeque::new()),
			bad: Mutex::new(HashSet::new()),
			more_to_verify: SMutex::new(()),
			empty: SMutex::new(()),

		});
		let more_to_verify = Arc::new(SCondvar::new());
		let deleting = Arc::new(AtomicBool::new(false));
		let ready_signal = Arc::new(QueueSignal {
			deleting: deleting.clone(),
			signalled: AtomicBool::new(false),
			message_channel: message_channel
		});
		let empty = Arc::new(SCondvar::new());
		let panic_handler = PanicHandler::new_in_arc();

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
			max_queue_size: max(config.max_queue_size, MIN_QUEUE_LIMIT),
			max_mem_use: max(config.max_mem_use, MIN_MEM_LIMIT),
		}
	}

	fn verify(verification: Arc<Verification>, engine: Arc<Box<Engine>>, wait: Arc<SCondvar>, ready: Arc<QueueSignal>, deleting: Arc<AtomicBool>, empty: Arc<SCondvar>) {
		while !deleting.load(AtomicOrdering::Acquire) {
			{
				let mut more_to_verify = verification.more_to_verify.lock().unwrap();

				if verification.unverified.lock().is_empty() && verification.verifying.lock().is_empty() {
					empty.notify_all();
				}

				while verification.unverified.lock().is_empty() && !deleting.load(AtomicOrdering::Acquire) {
					more_to_verify = wait.wait(more_to_verify).unwrap();
				}

				if deleting.load(AtomicOrdering::Acquire) {
					return;
				}
			}

			let block = {
				let mut unverified = verification.unverified.lock();
				if unverified.is_empty() {
					continue;
				}
				let mut verifying = verification.verifying.lock();
				let block = unverified.pop_front().unwrap();
				verifying.push_back(VerifyingBlock{ hash: block.header.hash(), block: None });
				block
			};

			let block_hash = block.header.hash();
			match verify_block_unordered(block.header, block.bytes, engine.deref().deref()) {
				Ok(verified) => {
					let mut verifying = verification.verifying.lock();
					for e in verifying.iter_mut() {
						if e.hash == block_hash {
							e.block = Some(verified);
							break;
						}
					}
					if !verifying.is_empty() && verifying.front().unwrap().hash == block_hash {
						// we're next!
						let mut verified = verification.verified.lock();
						let mut bad = verification.bad.lock();
						BlockQueue::drain_verifying(&mut verifying, &mut verified, &mut bad);
						ready.set();
					}
				},
				Err(err) => {
					let mut verifying = verification.verifying.lock();
					let mut verified = verification.verified.lock();
					let mut bad = verification.bad.lock();
					warn!(target: "client", "Stage 2 block verification failed for {}\nError: {:?}", block_hash, err);
					bad.insert(block_hash.clone());
					verifying.retain(|e| e.hash != block_hash);
					BlockQueue::drain_verifying(&mut verifying, &mut verified, &mut bad);
					ready.set();
				}
			}
		}
	}

	fn drain_verifying(verifying: &mut VecDeque<VerifyingBlock>, verified: &mut VecDeque<PreverifiedBlock>, bad: &mut HashSet<H256>) {
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
	pub fn clear(&self) {
		let mut unverified = self.verification.unverified.lock();
		let mut verifying = self.verification.verifying.lock();
		let mut verified = self.verification.verified.lock();
		unverified.clear();
		verifying.clear();
		verified.clear();
		self.processing.write().clear();
	}

	/// Wait for unverified queue to be empty
	pub fn flush(&self) {
		let mut lock = self.verification.empty.lock().unwrap();
		while !self.verification.unverified.lock().is_empty() || !self.verification.verifying.lock().is_empty() {
			lock = self.empty.wait(lock).unwrap();
		}
	}

	/// Check if the block is currently in the queue
	pub fn block_status(&self, hash: &H256) -> BlockStatus {
		if self.processing.read().contains(hash) {
			return BlockStatus::Queued;
		}
		if self.verification.bad.lock().contains(hash) {
			return BlockStatus::Bad;
		}
		BlockStatus::Unknown
	}

	/// Add a block to the queue.
	pub fn import_block(&self, bytes: Bytes) -> ImportResult {
		let header = BlockView::new(&bytes).header();
		let h = header.hash();
		{
			if self.processing.read().contains(&h) {
				return Err(ImportError::AlreadyQueued.into());
			}

			let mut bad = self.verification.bad.lock();
			if bad.contains(&h) {
				return Err(ImportError::KnownBad.into());
			}

			if bad.contains(&header.parent_hash) {
				bad.insert(h.clone());
				return Err(ImportError::KnownBad.into());
			}
		}

		match verify_block_basic(&header, &bytes, self.engine.deref().deref()) {
			Ok(()) => {
				self.processing.write().insert(h.clone());
				self.verification.unverified.lock().push_back(UnverifiedBlock { header: header, bytes: bytes });
				self.more_to_verify.notify_all();
				Ok(h)
			},
			Err(err) => {
				warn!(target: "client", "Stage 1 block verification failed for {}\nError: {:?}", BlockView::new(&bytes).header_view().sha3(), err);
				self.verification.bad.lock().insert(h.clone());
				Err(err)
			}
		}
	}

	/// Mark given block and all its children as bad. Stops verification.
	pub fn mark_as_bad(&self, block_hashes: &[H256]) {
		if block_hashes.is_empty() {
			return;
		}
		let mut verified_lock = self.verification.verified.lock();
		let mut verified = verified_lock.deref_mut();
		let mut bad = self.verification.bad.lock();
		let mut processing = self.processing.write();
		bad.reserve(block_hashes.len());
		for hash in block_hashes {
			bad.insert(hash.clone());
			processing.remove(hash);
		}

		let mut new_verified = VecDeque::new();
		for block in verified.drain(..) {
			if bad.contains(&block.header.parent_hash) {
				bad.insert(block.header.hash());
				processing.remove(&block.header.hash());
			} else {
				new_verified.push_back(block);
			}
		}
		*verified = new_verified;
	}

	/// Mark given block as processed
	pub fn mark_as_good(&self, block_hashes: &[H256]) {
		if block_hashes.is_empty() {
			return;
		}
		let mut processing = self.processing.write();
		for hash in block_hashes {
			processing.remove(hash);
		}
	}

	/// Removes up to `max` verified blocks from the queue
	pub fn drain(&self, max: usize) -> Vec<PreverifiedBlock> {
		let mut verified = self.verification.verified.lock();
		let count = min(max, verified.len());
		let mut result = Vec::with_capacity(count);
		for _ in 0..count {
			let block = verified.pop_front().unwrap();
			result.push(block);
		}
		self.ready_signal.reset();
		if !verified.is_empty() {
			self.ready_signal.set();
		}
		result
	}

	/// Get queue status.
	pub fn queue_info(&self) -> BlockQueueInfo {
		let (unverified_len, unverified_bytes) = {
			let v = self.verification.unverified.lock();
			(v.len(), v.heap_size_of_children())
		};
		let (verifying_len, verifying_bytes) = {
			let v = self.verification.verifying.lock();
			(v.len(), v.heap_size_of_children())
		};
		let (verified_len, verified_bytes) = {
			let v = self.verification.verified.lock();
			(v.len(), v.heap_size_of_children())
		};
		BlockQueueInfo {
			unverified_queue_size: unverified_len,
			verifying_queue_size: verifying_len,
			verified_queue_size: verified_len,
			max_queue_size: self.max_queue_size,
			max_mem_use: self.max_mem_use,
			mem_used:
				unverified_bytes
				+ verifying_bytes
				+ verified_bytes
				// TODO: https://github.com/servo/heapsize/pull/50
				//+ self.processing.read().heap_size_of_children(),
		}
	}

	/// Optimise memory footprint of the heap fields.
	pub fn collect_garbage(&self) {
		{
			self.verification.unverified.lock().shrink_to_fit();
			self.verification.verifying.lock().shrink_to_fit();
			self.verification.verified.lock().shrink_to_fit();
		}
		self.processing.write().shrink_to_fit();
	}
}

impl MayPanic for BlockQueue {
	fn on_panic<F>(&self, closure: F) where F: OnPanicListener {
		self.panic_handler.on_panic(closure);
	}
}

impl Drop for BlockQueue {
	fn drop(&mut self) {
		trace!(target: "shutdown", "[BlockQueue] Closing...");
		self.clear();
		self.deleting.store(true, AtomicOrdering::Release);
		self.more_to_verify.notify_all();
		for t in self.verifiers.drain(..) {
			t.join().unwrap();
		}
		trace!(target: "shutdown", "[BlockQueue] Closed.");
	}
}

#[cfg(test)]
mod tests {
	use util::*;
	use io::*;
	use spec::*;
	use block_queue::*;
	use tests::helpers::*;
	use error::*;
	use views::*;

	fn get_test_queue() -> BlockQueue {
		let spec = get_test_spec();
		let engine = spec.engine;
		BlockQueue::new(BlockQueueConfig::default(), Arc::new(engine), IoChannel::disconnected())
	}

	#[test]
	fn can_be_created() {
		// TODO better test
		let spec = Spec::new_test();
		let engine = spec.engine;
		let _ = BlockQueue::new(BlockQueueConfig::default(), Arc::new(engine), IoChannel::disconnected());
	}

	#[test]
	fn can_import_blocks() {
		let queue = get_test_queue();
		if let Err(e) = queue.import_block(get_good_dummy_block()) {
			panic!("error importing block that is valid by definition({:?})", e);
		}
	}

	#[test]
	fn returns_error_for_duplicates() {
		let queue = get_test_queue();
		if let Err(e) = queue.import_block(get_good_dummy_block()) {
			panic!("error importing block that is valid by definition({:?})", e);
		}

		let duplicate_import = queue.import_block(get_good_dummy_block());
		match duplicate_import {
			Err(e) => {
				match e {
					Error::Import(ImportError::AlreadyQueued) => {},
					_ => { panic!("must return AlreadyQueued error"); }
				}
			}
			Ok(_) => { panic!("must produce error"); }
		}
	}

	#[test]
	fn returns_ok_for_drained_duplicates() {
		let queue = get_test_queue();
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
		let queue = get_test_queue();
		queue.import_block(get_good_dummy_block()).expect("error importing block that is valid by definition");
		queue.flush();
		queue.drain(1);

		assert!(queue.queue_info().is_empty());
	}

	#[test]
	fn test_mem_limit() {
		let spec = get_test_spec();
		let engine = spec.engine;
		let mut config = BlockQueueConfig::default();
		config.max_mem_use = super::MIN_MEM_LIMIT;  // empty queue uses about 15000
		let queue = BlockQueue::new(config, Arc::new(engine), IoChannel::disconnected());
		assert!(!queue.queue_info().is_full());
		let mut blocks = get_good_dummy_block_seq(50);
		for b in blocks.drain(..) {
			queue.import_block(b).unwrap();
		}
		assert!(queue.queue_info().is_full());
	}
}
