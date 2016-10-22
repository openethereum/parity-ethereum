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
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering as AtomicOrdering};
use std::sync::{Condvar as SCondvar, Mutex as SMutex};
use util::*;
use io::*;
use error::*;
use engines::Engine;
use service::*;

use self::kind::{HasHash, Kind};

pub use types::verification_queue_info::VerificationQueueInfo as QueueInfo;

pub mod kind;

const MIN_MEM_LIMIT: usize = 16384;
const MIN_QUEUE_LIMIT: usize = 512;

/// Type alias for block queue convenience.
pub type BlockQueue = VerificationQueue<self::kind::Blocks>;

/// Type alias for header queue convenience.
pub type HeaderQueue = VerificationQueue<self::kind::Headers>;

/// Verification queue configuration
#[derive(Debug, PartialEq, Clone)]
pub struct Config {
	/// Maximum number of items to keep in unverified queue.
	/// When the limit is reached, is_full returns true.
	pub max_queue_size: usize,
	/// Maximum heap memory to use.
	/// When the limit is reached, is_full returns true.
	pub max_mem_use: usize,
}

impl Default for Config {
	fn default() -> Self {
		Config {
			max_queue_size: 30000,
			max_mem_use: 50 * 1024 * 1024,
		}
	}
}

/// An item which is in the process of being verified.
pub struct Verifying<K: Kind> {
	hash: H256,
	output: Option<K::Verified>,
}

impl<K: Kind> HeapSizeOf for Verifying<K> {
	fn heap_size_of_children(&self) -> usize {
		self.output.heap_size_of_children()
	}
}

/// Status of items in the queue.
pub enum Status {
	/// Currently queued.
	Queued,
	/// Known to be bad.
	Bad,
	/// Unknown.
	Unknown,
}

// the internal queue sizes.
struct Sizes {
	unverified: AtomicUsize,
	verifying: AtomicUsize,
	verified: AtomicUsize,
}

/// A queue of items to be verified. Sits between network or other I/O and the `BlockChain`.
/// Keeps them in the same order as inserted, minus invalid items.
pub struct VerificationQueue<K: Kind> {
	panic_handler: Arc<PanicHandler>,
	engine: Arc<Engine>,
	more_to_verify: Arc<SCondvar>,
	verification: Arc<Verification<K>>,
	verifiers: Vec<JoinHandle<()>>,
	deleting: Arc<AtomicBool>,
	ready_signal: Arc<QueueSignal>,
	empty: Arc<SCondvar>,
	processing: RwLock<HashSet<H256>>,
	max_queue_size: usize,
	max_mem_use: usize,
}

struct QueueSignal {
	deleting: Arc<AtomicBool>,
	signalled: AtomicBool,
	message_channel: IoChannel<ClientIoMessage>,
}

impl QueueSignal {
	#[cfg_attr(feature="dev", allow(bool_comparison))]
	fn set_sync(&self) {
		// Do not signal when we are about to close
		if self.deleting.load(AtomicOrdering::Relaxed) {
			return;
		}

		if self.signalled.compare_and_swap(false, true, AtomicOrdering::Relaxed) == false {
			if let Err(e) = self.message_channel.send_sync(ClientIoMessage::BlockVerified) {
				debug!("Error sending BlockVerified message: {:?}", e);
			}
		}
	}

	#[cfg_attr(feature="dev", allow(bool_comparison))]
	fn set_async(&self) {
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

struct Verification<K: Kind> {
	// All locks must be captured in the order declared here.
	unverified: Mutex<VecDeque<K::Unverified>>,
	verifying: Mutex<VecDeque<Verifying<K>>>,
	verified: Mutex<VecDeque<K::Verified>>,
	bad: Mutex<HashSet<H256>>,
	more_to_verify: SMutex<()>,
	empty: SMutex<()>,
	sizes: Sizes,
}

impl<K: Kind> VerificationQueue<K> {
	/// Creates a new queue instance.
	pub fn new(config: Config, engine: Arc<Engine>, message_channel: IoChannel<ClientIoMessage>) -> Self {
		let verification = Arc::new(Verification {
			unverified: Mutex::new(VecDeque::new()),
			verifying: Mutex::new(VecDeque::new()),
			verified: Mutex::new(VecDeque::new()),
			bad: Mutex::new(HashSet::new()),
			more_to_verify: SMutex::new(()),
			empty: SMutex::new(()),
			sizes: Sizes {
				unverified: AtomicUsize::new(0),
				verifying: AtomicUsize::new(0),
				verified: AtomicUsize::new(0),
			}
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
						VerificationQueue::verify(verification, engine, more_to_verify, ready_signal, deleting, empty)
					}).unwrap()
				})
				.expect("Error starting block verification thread")
			);
		}
		VerificationQueue {
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

	fn verify(verification: Arc<Verification<K>>, engine: Arc<Engine>, wait: Arc<SCondvar>, ready: Arc<QueueSignal>, deleting: Arc<AtomicBool>, empty: Arc<SCondvar>) {
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

			let item = {
				// acquire these locks before getting the item to verify.
				let mut unverified = verification.unverified.lock();
				let mut verifying = verification.verifying.lock();

				let item = match unverified.pop_front() {
					Some(item) => item,
					None => continue,
				};

				verification.sizes.unverified.fetch_sub(item.heap_size_of_children(), AtomicOrdering::SeqCst);
				verifying.push_back(Verifying { hash: item.hash(), output: None });
				item
			};

			let hash = item.hash();
			let is_ready = match K::verify(item, &*engine) {
				Ok(verified) => {
					let mut verifying = verification.verifying.lock();
					let mut idx = None;
					for (i, e) in verifying.iter_mut().enumerate() {
						if e.hash == hash {
							idx = Some(i);

							verification.sizes.verifying.fetch_add(verified.heap_size_of_children(), AtomicOrdering::SeqCst);
							e.output = Some(verified);
							break;
						}
					}

					if idx == Some(0) {
						// we're next!
						let mut verified = verification.verified.lock();
						let mut bad = verification.bad.lock();
						VerificationQueue::drain_verifying(&mut verifying, &mut verified, &mut bad, &verification.sizes);
						true
					} else {
						false
					}
				},
				Err(_) => {
					let mut verifying = verification.verifying.lock();
					let mut verified = verification.verified.lock();
					let mut bad = verification.bad.lock();

					bad.insert(hash.clone());
					verifying.retain(|e| e.hash != hash);

					if verifying.front().map_or(false, |x| x.output.is_some()) {
						VerificationQueue::drain_verifying(&mut verifying, &mut verified, &mut bad, &verification.sizes);
						true
					} else {
						false
					}
				}
			};
			if is_ready {
				// Import the block immediately
				ready.set_sync();
			}
		}
	}

	fn drain_verifying(verifying: &mut VecDeque<Verifying<K>>, verified: &mut VecDeque<K::Verified>, bad: &mut HashSet<H256>, sizes: &Sizes) {
		let mut removed_size = 0;
		let mut inserted_size = 0;
		while let Some(output) = verifying.front_mut().and_then(|x| x.output.take()) {
			assert!(verifying.pop_front().is_some());
			let size = output.heap_size_of_children();
			removed_size += size;

			if bad.contains(&output.parent_hash()) {
				bad.insert(output.hash());
			} else {
				inserted_size += size;
				verified.push_back(output);
			}
		}

		sizes.verifying.fetch_sub(removed_size, AtomicOrdering::SeqCst);
		sizes.verified.fetch_add(inserted_size, AtomicOrdering::SeqCst);
	}

	/// Clear the queue and stop verification activity.
	pub fn clear(&self) {
		let mut unverified = self.verification.unverified.lock();
		let mut verifying = self.verification.verifying.lock();
		let mut verified = self.verification.verified.lock();
		unverified.clear();
		verifying.clear();
		verified.clear();

		let sizes = &self.verification.sizes;
		sizes.unverified.store(0, AtomicOrdering::Release);
		sizes.verifying.store(0, AtomicOrdering::Release);
		sizes.verified.store(0, AtomicOrdering::Release);

		self.processing.write().clear();
	}

	/// Wait for unverified queue to be empty
	pub fn flush(&self) {
		let mut lock = self.verification.empty.lock().unwrap();
		while !self.verification.unverified.lock().is_empty() || !self.verification.verifying.lock().is_empty() {
			lock = self.empty.wait(lock).unwrap();
		}
	}

	/// Check if the item is currently in the queue
	pub fn status(&self, hash: &H256) -> Status {
		if self.processing.read().contains(hash) {
			return Status::Queued;
		}
		if self.verification.bad.lock().contains(hash) {
			return Status::Bad;
		}
		Status::Unknown
	}

	/// Add a block to the queue.
	pub fn import(&self, input: K::Input) -> ImportResult {
		let h = input.hash();
		{
			if self.processing.read().contains(&h) {
				return Err(ImportError::AlreadyQueued.into());
			}

			let mut bad = self.verification.bad.lock();
			if bad.contains(&h) {
				return Err(ImportError::KnownBad.into());
			}

			if bad.contains(&input.parent_hash()) {
				bad.insert(h.clone());
				return Err(ImportError::KnownBad.into());
			}
		}

		match K::create(input, &*self.engine) {
			Ok(item) => {
				self.verification.sizes.unverified.fetch_add(item.heap_size_of_children(), AtomicOrdering::SeqCst);

				self.processing.write().insert(h.clone());
				self.verification.unverified.lock().push_back(item);
				self.more_to_verify.notify_all();
				Ok(h)
			},
			Err(err) => {
				self.verification.bad.lock().insert(h.clone());
				Err(err)
			}
		}
	}

	/// Mark given item and all its children as bad. pauses verification
	/// until complete.
	pub fn mark_as_bad(&self, hashes: &[H256]) {
		if hashes.is_empty() {
			return;
		}
		let mut verified_lock = self.verification.verified.lock();
		let mut verified = &mut *verified_lock;
		let mut bad = self.verification.bad.lock();
		let mut processing = self.processing.write();
		bad.reserve(hashes.len());
		for hash in hashes {
			bad.insert(hash.clone());
			processing.remove(hash);
		}

		let mut new_verified = VecDeque::new();
		let mut removed_size = 0;
		for output in verified.drain(..) {
			if bad.contains(&output.parent_hash()) {
				removed_size += output.heap_size_of_children();
				bad.insert(output.hash());
				processing.remove(&output.hash());
			} else {
				new_verified.push_back(output);
			}
		}

		self.verification.sizes.verified.fetch_sub(removed_size, AtomicOrdering::SeqCst);
		*verified = new_verified;
	}

	/// Mark given item as processed.
	/// Returns true if the queue becomes empty.
	pub fn mark_as_good(&self, hashes: &[H256]) -> bool {
		if hashes.is_empty() {
			return self.processing.read().is_empty();
		}
		let mut processing = self.processing.write();
		for hash in hashes {
			processing.remove(hash);
		}
		processing.is_empty()
	}

	/// Removes up to `max` verified items from the queue
	pub fn drain(&self, max: usize) -> Vec<K::Verified> {
		let mut verified = self.verification.verified.lock();
		let count = min(max, verified.len());
		let result = verified.drain(..count).collect::<Vec<_>>();

		let drained_size = result.iter().map(HeapSizeOf::heap_size_of_children).fold(0, |a, c| a + c);
		self.verification.sizes.verified.fetch_sub(drained_size, AtomicOrdering::SeqCst);

		self.ready_signal.reset();
		if !verified.is_empty() {
			self.ready_signal.set_async();
		}
		result
	}

	/// Get queue status.
	pub fn queue_info(&self) -> QueueInfo {
		use std::mem::size_of;

		let (unverified_len, unverified_bytes) = {
			let len = self.verification.unverified.lock().len();
			let size = self.verification.sizes.unverified.load(AtomicOrdering::Acquire);

			(len, size + len * size_of::<K::Unverified>())
		};
		let (verifying_len, verifying_bytes) = {
			let len = self.verification.verifying.lock().len();
			let size = self.verification.sizes.verifying.load(AtomicOrdering::Acquire);
			(len, size + len * size_of::<Verifying<K>>())
		};
		let (verified_len, verified_bytes) = {
			let len = self.verification.verified.lock().len();
			let size = self.verification.sizes.verified.load(AtomicOrdering::Acquire);
			(len, size + len * size_of::<K::Verified>())
		};

		QueueInfo {
			unverified_queue_size: unverified_len,
			verifying_queue_size: verifying_len,
			verified_queue_size: verified_len,
			max_queue_size: self.max_queue_size,
			max_mem_use: self.max_mem_use,
			mem_used: unverified_bytes
					   + verifying_bytes
					   + verified_bytes
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

impl<K: Kind> MayPanic for VerificationQueue<K> {
	fn on_panic<F>(&self, closure: F) where F: OnPanicListener {
		self.panic_handler.on_panic(closure);
	}
}

impl<K: Kind> Drop for VerificationQueue<K> {
	fn drop(&mut self) {
		trace!(target: "shutdown", "[VerificationQueue] Closing...");
		self.clear();
		self.deleting.store(true, AtomicOrdering::Release);
		self.more_to_verify.notify_all();
		for t in self.verifiers.drain(..) {
			t.join().unwrap();
		}
		trace!(target: "shutdown", "[VerificationQueue] Closed.");
	}
}

#[cfg(test)]
mod tests {
	use util::*;
	use io::*;
	use spec::*;
	use super::{BlockQueue, Config};
	use super::kind::blocks::Unverified;
	use tests::helpers::*;
	use error::*;
	use views::*;

	fn get_test_queue() -> BlockQueue {
		let spec = get_test_spec();
		let engine = spec.engine;
		BlockQueue::new(Config::default(), engine, IoChannel::disconnected())
	}

	#[test]
	fn can_be_created() {
		// TODO better test
		let spec = Spec::new_test();
		let engine = spec.engine;
		let _ = BlockQueue::new(Config::default(), engine, IoChannel::disconnected());
	}

	#[test]
	fn can_import_blocks() {
		let queue = get_test_queue();
		if let Err(e) = queue.import(Unverified::new(get_good_dummy_block())) {
			panic!("error importing block that is valid by definition({:?})", e);
		}
	}

	#[test]
	fn returns_error_for_duplicates() {
		let queue = get_test_queue();
		if let Err(e) = queue.import(Unverified::new(get_good_dummy_block())) {
			panic!("error importing block that is valid by definition({:?})", e);
		}

		let duplicate_import = queue.import(Unverified::new(get_good_dummy_block()));
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
		if let Err(e) = queue.import(Unverified::new(block)) {
			panic!("error importing block that is valid by definition({:?})", e);
		}
		queue.flush();
		queue.drain(10);
		queue.mark_as_good(&[ hash ]);

		if let Err(e) = queue.import(Unverified::new(get_good_dummy_block())) {
			panic!("error importing block that has already been drained ({:?})", e);
		}
	}

	#[test]
	fn returns_empty_once_finished() {
		let queue = get_test_queue();
		queue.import(Unverified::new(get_good_dummy_block()))
			.expect("error importing block that is valid by definition");
		queue.flush();
		queue.drain(1);

		assert!(queue.queue_info().is_empty());
	}

	#[test]
	fn test_mem_limit() {
		let spec = get_test_spec();
		let engine = spec.engine;
		let mut config = Config::default();
		config.max_mem_use = super::MIN_MEM_LIMIT;  // empty queue uses about 15000
		let queue = BlockQueue::new(config, engine, IoChannel::disconnected());
		assert!(!queue.queue_info().is_full());
		let mut blocks = get_good_dummy_block_seq(50);
		for b in blocks.drain(..) {
			queue.import(Unverified::new(b)).unwrap();
		}
		assert!(queue.queue_info().is_full());
	}
}
