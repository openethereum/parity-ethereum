// Copyright 2015-2019 Parity Technologies (UK) Ltd.
// This file is part of Parity Ethereum.

// Parity Ethereum is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity Ethereum is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity Ethereum.  If not, see <http://www.gnu.org/licenses/>.

//! A queue of blocks. Sits between network or other I/O and the `BlockChain`.
//! Sorts them ready for blockchain insertion.

use std::thread::{self, JoinHandle};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering as AtomicOrdering};
use std::sync::Arc;
use std::cmp;
use std::collections::{VecDeque, HashSet, HashMap};
use parity_util_mem::{MallocSizeOf, MallocSizeOfExt};
use ethereum_types::{H256, U256};
use parking_lot::{Condvar, Mutex, RwLock};
use io::*;
use engine::Engine;
use client::ClientIoMessage;
use len_caching_lock::LenCachingMutex;
use types::errors::{BlockError, EthcoreError as Error, ImportError};

use self::kind::{BlockLike, Kind};

pub use types::verification::VerificationQueueInfo as QueueInfo;

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
	/// Settings for the number of verifiers and adaptation strategy.
	pub verifier_settings: VerifierSettings,
}

impl Default for Config {
	fn default() -> Self {
		Config {
			max_queue_size: 30000,
			max_mem_use: 50 * 1024 * 1024,
			verifier_settings: VerifierSettings::default(),
		}
	}
}

/// Verifier settings.
#[derive(Debug, PartialEq, Clone)]
pub struct VerifierSettings {
	/// Whether to scale amount of verifiers according to load.
	// Todo: replace w/ strategy enum?
	pub scale_verifiers: bool,
	/// Beginning amount of verifiers.
	pub num_verifiers: usize,
}

impl Default for VerifierSettings {
	fn default() -> Self {
		VerifierSettings {
			scale_verifiers: false,
			num_verifiers: ::num_cpus::get(),
		}
	}
}

// pool states
enum State {
	// all threads with id < inner value are to work.
	Work(usize),
	Exit,
}

/// An item which is in the process of being verified.
#[derive(MallocSizeOf)]
pub struct Verifying<K: Kind> {
	hash: H256,
	output: Option<K::Verified>,
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

impl Into<::types::block_status::BlockStatus> for Status {
	fn into(self) -> ::types::block_status::BlockStatus {
		use ::types::block_status::BlockStatus;
		match self {
			Status::Queued => BlockStatus::Queued,
			Status::Bad => BlockStatus::Bad,
			Status::Unknown => BlockStatus::Unknown,
		}
	}
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
	engine: Arc<dyn Engine>,
	more_to_verify: Arc<Condvar>,
	verification: Arc<Verification<K>>,
	deleting: Arc<AtomicBool>,
	ready_signal: Arc<QueueSignal>,
	empty: Arc<Condvar>,
	processing: RwLock<HashMap<H256, U256>>, // hash to difficulty
	ticks_since_adjustment: AtomicUsize,
	max_queue_size: usize,
	max_mem_use: usize,
	scale_verifiers: bool,
	verifier_handles: Vec<JoinHandle<()>>,
	state: Arc<(Mutex<State>, Condvar)>,
	total_difficulty: RwLock<U256>,
}

struct QueueSignal {
	deleting: Arc<AtomicBool>,
	signalled: AtomicBool,
	message_channel: Mutex<IoChannel<ClientIoMessage>>,
}

impl QueueSignal {
	fn set_sync(&self) {
		// Do not signal when we are about to close
		if self.deleting.load(AtomicOrdering::Relaxed) {
			return;
		}

		if self.signalled.compare_and_swap(false, true, AtomicOrdering::Relaxed) == false {
			let channel = self.message_channel.lock().clone();
			if let Err(e) = channel.send_sync(ClientIoMessage::BlockVerified) {
				debug!("Error sending BlockVerified message: {:?}", e);
			}
		}
	}

	fn set_async(&self) {
		// Do not signal when we are about to close
		if self.deleting.load(AtomicOrdering::Relaxed) {
			return;
		}

		if self.signalled.compare_and_swap(false, true, AtomicOrdering::Relaxed) == false {
			let channel = self.message_channel.lock().clone();
			if let Err(e) = channel.send(ClientIoMessage::BlockVerified) {
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
	unverified: LenCachingMutex<VecDeque<K::Unverified>>,
	verifying: LenCachingMutex<VecDeque<Verifying<K>>>,
	verified: LenCachingMutex<VecDeque<K::Verified>>,
	bad: Mutex<HashSet<H256>>,
	sizes: Sizes,
	check_seal: bool,
}

impl<K: Kind> VerificationQueue<K> {
	/// Creates a new queue instance.
	pub fn new(config: Config, engine: Arc<dyn Engine>, message_channel: IoChannel<ClientIoMessage>, check_seal: bool) -> Self {
		let verification = Arc::new(Verification {
			unverified: LenCachingMutex::new(VecDeque::new()),
			verifying: LenCachingMutex::new(VecDeque::new()),
			verified: LenCachingMutex::new(VecDeque::new()),
			bad: Mutex::new(HashSet::new()),
			sizes: Sizes {
				unverified: AtomicUsize::new(0),
				verifying: AtomicUsize::new(0),
				verified: AtomicUsize::new(0),
			},
			check_seal: check_seal,
		});
		let more_to_verify = Arc::new(Condvar::new());
		let deleting = Arc::new(AtomicBool::new(false));
		let ready_signal = Arc::new(QueueSignal {
			deleting: deleting.clone(),
			signalled: AtomicBool::new(false),
			message_channel: Mutex::new(message_channel),
		});
		let empty = Arc::new(Condvar::new());
		let scale_verifiers = config.verifier_settings.scale_verifiers;

		let max_verifiers = ::num_cpus::get();
		let default_amount = cmp::max(1, cmp::min(max_verifiers, config.verifier_settings.num_verifiers));

		// if `auto-scaling` is enabled spawn up extra threads as they might be needed
		// otherwise just spawn the number of threads specified by the config
		let number_of_threads = if scale_verifiers {
			max_verifiers
		} else {
			cmp::min(default_amount, max_verifiers)
		};

		let state = Arc::new((Mutex::new(State::Work(default_amount)), Condvar::new()));
		let mut verifier_handles = Vec::with_capacity(number_of_threads);

		debug!(target: "verification", "Allocating {} verifiers, {} initially active", number_of_threads, default_amount);
		debug!(target: "verification", "Verifier auto-scaling {}", if scale_verifiers { "enabled" } else { "disabled" });

		for i in 0..number_of_threads {
			debug!(target: "verification", "Adding verification thread #{}", i);

			let verification = verification.clone();
			let engine = engine.clone();
			let wait = more_to_verify.clone();
			let ready = ready_signal.clone();
			let empty = empty.clone();
			let state = state.clone();

			let handle = thread::Builder::new()
				.name(format!("Verifier #{}", i))
				.spawn(move || {
					VerificationQueue::verify(
						verification,
						engine,
						wait,
						ready,
						empty,
						state,
						i,
					)
				})
				.expect("Failed to create verifier thread.");
			verifier_handles.push(handle);
		}

		VerificationQueue {
			engine: engine,
			ready_signal: ready_signal,
			more_to_verify: more_to_verify,
			verification: verification,
			deleting: deleting,
			processing: RwLock::new(HashMap::new()),
			empty: empty,
			ticks_since_adjustment: AtomicUsize::new(0),
			max_queue_size: cmp::max(config.max_queue_size, MIN_QUEUE_LIMIT),
			max_mem_use: cmp::max(config.max_mem_use, MIN_MEM_LIMIT),
			scale_verifiers: scale_verifiers,
			verifier_handles: verifier_handles,
			state: state,
			total_difficulty: RwLock::new(0.into()),
		}
	}

	fn verify(
		verification: Arc<Verification<K>>,
		engine: Arc<dyn Engine>,
		wait: Arc<Condvar>,
		ready: Arc<QueueSignal>,
		empty: Arc<Condvar>,
		state: Arc<(Mutex<State>, Condvar)>,
		id: usize,
	) {
		loop {
			// check current state.
			{
				let mut cur_state = state.0.lock();
				while let State::Work(x) = *cur_state {
					// sleep until this thread is required.
					if id < x { break }

					debug!(target: "verification", "verifier {} sleeping", id);
					state.1.wait(&mut cur_state);
					debug!(target: "verification", "verifier {} waking up", id);
				}

				if let State::Exit = *cur_state {
					debug!(target: "verification", "verifier {} exiting", id);
					break;
				}
			}

			// wait for work if empty.
			{
				let mut unverified = verification.unverified.lock();

				if unverified.is_empty() && verification.verifying.lock().is_empty() {
					empty.notify_all();
				}

				while unverified.is_empty() {
					if let State::Exit = *state.0.lock() {
						debug!(target: "verification", "verifier {} exiting", id);
						return;
					}

					wait.wait(unverified.inner_mut());
				}

				if let State::Exit = *state.0.lock() {
					debug!(target: "verification", "verifier {} exiting", id);
					return;
				}
			}

			// do work.
			let item = {
				// acquire these locks before getting the item to verify.
				let mut unverified = verification.unverified.lock();
				let mut verifying = verification.verifying.lock();

				let item = match unverified.pop_front() {
					Some(item) => item,
					None => continue,
				};

				verification.sizes.unverified.fetch_sub(item.malloc_size_of(), AtomicOrdering::SeqCst);
				verifying.push_back(Verifying { hash: item.hash(), output: None });
				item
			};

			let hash = item.hash();
			let is_ready = match K::verify(item, &*engine, verification.check_seal) {
				Ok(verified) => {
					let mut verifying = verification.verifying.lock();
					let mut idx = None;
					for (i, e) in verifying.iter_mut().enumerate() {
						if e.hash == hash {
							idx = Some(i);

							verification.sizes.verifying.fetch_add(verified.malloc_size_of(), AtomicOrdering::SeqCst);
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

	fn drain_verifying(
		verifying: &mut VecDeque<Verifying<K>>,
		verified: &mut VecDeque<K::Verified>,
		bad: &mut HashSet<H256>,
		sizes: &Sizes,
	) {
		let mut removed_size = 0;
		let mut inserted_size = 0;

		while let Some(output) = verifying.front_mut().and_then(|x| x.output.take()) {
			assert!(verifying.pop_front().is_some());
			let size = output.malloc_size_of();
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
		*self.total_difficulty.write() = 0.into();

		self.processing.write().clear();
	}

	/// Wait for unverified queue to be empty
	pub fn flush(&self) {
		let mut unverified = self.verification.unverified.lock();
		while !unverified.is_empty() || !self.verification.verifying.lock().is_empty() {
			self.empty.wait(unverified.inner_mut());
		}
	}

	/// Check if the item is currently in the queue
	pub fn status(&self, hash: &H256) -> Status {
		if self.processing.read().contains_key(hash) {
			return Status::Queued;
		}
		if self.verification.bad.lock().contains(hash) {
			return Status::Bad;
		}
		Status::Unknown
	}

	/// Add a block to the queue.
	pub fn import(&self, input: K::Input) -> Result<H256, (K::Input, Error)> {
		let hash = input.hash();
		{
			if self.processing.read().contains_key(&hash) {
				return Err((input, Error::Import(ImportError::AlreadyQueued).into()));
			}

			let mut bad = self.verification.bad.lock();
			if bad.contains(&hash) {
				return Err((input, Error::Import(ImportError::KnownBad).into()));
			}

			if bad.contains(&input.parent_hash()) {
				bad.insert(hash);
				return Err((input, Error::Import(ImportError::KnownBad).into()));
			}
		}

		match K::create(input, &*self.engine, self.verification.check_seal) {
			Ok(item) => {
				self.verification.sizes.unverified.fetch_add(item.malloc_size_of(), AtomicOrdering::SeqCst);

				self.processing.write().insert(hash, item.difficulty());
				{
					let mut td = self.total_difficulty.write();
					*td = *td + item.difficulty();
				}
				self.verification.unverified.lock().push_back(item);
				self.more_to_verify.notify_all();
				Ok(hash)
			},
			Err((input, err)) => {
				match err {
					// Don't mark future blocks as bad.
					Error::Block(BlockError::TemporarilyInvalid(_)) => {},
					_ => {
						self.verification.bad.lock().insert(hash);
					}
				}
				Err((input, err))
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
		let verified = &mut *verified_lock;
		let mut bad = self.verification.bad.lock();
		let mut processing = self.processing.write();
		bad.reserve(hashes.len());
		for hash in hashes {
			bad.insert(hash.clone());
			if let Some(difficulty) = processing.remove(hash) {
				let mut td = self.total_difficulty.write();
				*td = *td - difficulty;
			}
		}

		let mut new_verified = VecDeque::new();
		let mut removed_size = 0;
		for output in verified.drain(..) {
			if bad.contains(&output.parent_hash()) {
				removed_size += output.malloc_size_of();
				bad.insert(output.hash());
				if let Some(difficulty) = processing.remove(&output.hash()) {
					let mut td = self.total_difficulty.write();
					*td = *td - difficulty;
				}
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
			if let Some(difficulty) = processing.remove(hash) {
				let mut td = self.total_difficulty.write();
				*td = *td - difficulty;
			}
		}
		processing.is_empty()
	}

	/// Removes up to `max` verified items from the queue
	pub fn drain(&self, max: usize) -> Vec<K::Verified> {
		let mut verified = self.verification.verified.lock();
		let count = cmp::min(max, verified.len());
		let result = verified.drain(..count).collect::<Vec<_>>();

		let drained_size = result.iter().map(MallocSizeOfExt::malloc_size_of).fold(0, |a, c| a + c);
		self.verification.sizes.verified.fetch_sub(drained_size, AtomicOrdering::SeqCst);

		self.ready_signal.reset();
		if !verified.is_empty() {
			self.ready_signal.set_async();
		}
		result
	}

	/// Returns true if there is nothing currently in the queue.
	pub fn is_empty(&self) -> bool {
		let v = &self.verification;

		v.unverified.load_len() == 0
			&& v.verifying.load_len() == 0
			&& v.verified.load_len() == 0
	}

	/// Get queue status.
	pub fn queue_info(&self) -> QueueInfo {
		use std::mem::size_of;

		let (unverified_len, unverified_bytes) = {
			let len = self.verification.unverified.load_len();
			let size = self.verification.sizes.unverified.load(AtomicOrdering::Acquire);

			(len, size + len * size_of::<K::Unverified>())
		};
		let (verifying_len, verifying_bytes) = {
			let len = self.verification.verifying.load_len();
			let size = self.verification.sizes.verifying.load(AtomicOrdering::Acquire);
			(len, size + len * size_of::<Verifying<K>>())
		};
		let (verified_len, verified_bytes) = {
			let len = self.verification.verified.load_len();
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

	/// Get the total difficulty of all the blocks in the queue.
	pub fn total_difficulty(&self) -> U256 {
		self.total_difficulty.read().clone()
	}

	/// Get the current number of working verifiers.
	pub fn num_verifiers(&self) -> usize {
		match *self.state.0.lock() {
			State::Work(x) => x,
			State::Exit => panic!("state only set to exit on drop; queue live now; qed"),
		}
	}

	/// Optimise memory footprint of the heap fields, and adjust the number of threads
	/// to better suit the workload.
	pub fn collect_garbage(&self) {
		// number of ticks to average queue stats over
		// when deciding whether to change the number of verifiers.
		#[cfg(not(test))]
		const READJUSTMENT_PERIOD: usize = 12;

		#[cfg(test)]
		const READJUSTMENT_PERIOD: usize = 1;

		let (u_len, v_len) = {
			let u_len = {
				let mut q = self.verification.unverified.lock();
				q.shrink_to_fit();
				q.len()
			};
			self.verification.verifying.lock().shrink_to_fit();

			let v_len = {
				let mut q = self.verification.verified.lock();
				q.shrink_to_fit();
				q.len()
			};

			(u_len as isize, v_len as isize)
		};

		self.processing.write().shrink_to_fit();

		if !self.scale_verifiers { return }

		if self.ticks_since_adjustment.fetch_add(1, AtomicOrdering::SeqCst) + 1 >= READJUSTMENT_PERIOD {
			self.ticks_since_adjustment.store(0, AtomicOrdering::SeqCst);
		} else {
			return;
		}

		let current = self.num_verifiers();

		let diff = (v_len - u_len).abs();
		let total = v_len + u_len;

		self.scale_verifiers(
			if u_len < 20 {
				1
			} else if diff <= total / 10 {
				current
			} else if v_len > u_len {
				current - 1
			} else {
				current + 1
			}
		);
	}

	// wake up or sleep verifiers to get as close to the target as
	// possible, never going over the amount of initially allocated threads
	// or below 1.
	fn scale_verifiers(&self, target: usize) {
		let current = self.num_verifiers();
		let target = cmp::min(self.verifier_handles.len(), target);
		let target = cmp::max(1, target);

		debug!(target: "verification", "Scaling from {} to {} verifiers", current, target);

		*self.state.0.lock() = State::Work(target);
		self.state.1.notify_all();
	}
}

impl<K: Kind> Drop for VerificationQueue<K> {
	fn drop(&mut self) {
		trace!(target: "shutdown", "[VerificationQueue] Closing...");
		self.clear();
		self.deleting.store(true, AtomicOrdering::SeqCst);

		// set exit state; should be done before `more_to_verify` notification.
		*self.state.0.lock() = State::Exit;
		self.state.1.notify_all();

		// acquire this lock to force threads to reach the waiting point
		// if they're in-between the exit check and the more_to_verify wait.
		{
			let _unverified = self.verification.unverified.lock();
			self.more_to_verify.notify_all();
		}

		// wait for all verifier threads to join.
		for thread in self.verifier_handles.drain(..) {
			thread.join().expect("Propagating verifier thread panic on shutdown");
		}

		trace!(target: "shutdown", "[VerificationQueue] Closed.");
	}
}

#[cfg(test)]
mod tests {
	use io::*;
	use super::{BlockQueue, Config, State};
	use test_helpers::{get_good_dummy_block_seq, get_good_dummy_block};
	use bytes::Bytes;
	use types::{
		errors::{EthcoreError, ImportError},
		verification::Unverified,
		view,
		views::BlockView,
	};
	use crate::spec;

	// create a test block queue.
	// auto_scaling enables verifier adjustment.
	fn get_test_queue(auto_scale: bool) -> BlockQueue {
		let spec = spec::new_test();
		let engine = spec.engine;

		let mut config = Config::default();
		config.verifier_settings.scale_verifiers = auto_scale;
		BlockQueue::new(config, engine, IoChannel::disconnected(), true)
	}

	fn get_test_config(num_verifiers: usize, is_auto_scale: bool) -> Config {
		let mut config = Config::default();
		config.verifier_settings.num_verifiers = num_verifiers;
		config.verifier_settings.scale_verifiers = is_auto_scale;
		config
	}

	fn new_unverified(bytes: Bytes) -> Unverified {
		Unverified::from_rlp(bytes).expect("Should be valid rlp")
	}

	#[test]
	fn can_be_created() {
		// TODO better test
		let spec = spec::new_test();
		let engine = spec.engine;
		let _ = BlockQueue::new(Config::default(), engine, IoChannel::disconnected(), true);
	}

	#[test]
	fn can_import_blocks() {
		let queue = get_test_queue(false);
		if let Err(e) = queue.import(new_unverified(get_good_dummy_block())) {
			panic!("error importing block that is valid by definition({:?})", e);
		}
	}

	#[test]
	fn returns_error_for_duplicates() {
		let queue = get_test_queue(false);
		if let Err(e) = queue.import(new_unverified(get_good_dummy_block())) {
			panic!("error importing block that is valid by definition({:?})", e);
		}

		let duplicate_import = queue.import(new_unverified(get_good_dummy_block()));
		match duplicate_import {
			Err((_, e)) => {
				match e {
					EthcoreError::Import(ImportError::AlreadyQueued) => {},
					_ => { panic!("must return AlreadyQueued error"); }
				}
			}
			Ok(_) => { panic!("must produce error"); }
		}
	}

	#[test]
	fn returns_total_difficulty() {
		let queue = get_test_queue(false);
		let block = get_good_dummy_block();
		let hash = view!(BlockView, &block).header().hash().clone();
		if let Err(e) = queue.import(new_unverified(block)) {
			panic!("error importing block that is valid by definition({:?})", e);
		}
		queue.flush();
		assert_eq!(queue.total_difficulty(), 131072.into());
		queue.drain(10);
		assert_eq!(queue.total_difficulty(), 131072.into());
		queue.mark_as_good(&[ hash ]);
		assert_eq!(queue.total_difficulty(), 0.into());
	}

	#[test]
	fn returns_ok_for_drained_duplicates() {
		let queue = get_test_queue(false);
		let block = get_good_dummy_block();
		let hash = view!(BlockView, &block).header().hash().clone();
		if let Err(e) = queue.import(new_unverified(block)) {
			panic!("error importing block that is valid by definition({:?})", e);
		}
		queue.flush();
		queue.drain(10);
		queue.mark_as_good(&[ hash ]);

		if let Err(e) = queue.import(new_unverified(get_good_dummy_block())) {
			panic!("error importing block that has already been drained ({:?})", e);
		}
	}

	#[test]
	fn returns_empty_once_finished() {
		let queue = get_test_queue(false);
		queue.import(new_unverified(get_good_dummy_block()))
			.expect("error importing block that is valid by definition");
		queue.flush();
		queue.drain(1);

		assert!(queue.queue_info().is_empty());
	}

	#[test]
	fn test_mem_limit() {
		let spec = spec::new_test();
		let engine = spec.engine;
		let mut config = Config::default();
		config.max_mem_use = super::MIN_MEM_LIMIT;  // empty queue uses about 15000
		let queue = BlockQueue::new(config, engine, IoChannel::disconnected(), true);
		assert!(!queue.queue_info().is_full());
		let mut blocks = get_good_dummy_block_seq(50);
		for b in blocks.drain(..) {
			queue.import(new_unverified(b)).unwrap();
		}
		assert!(queue.queue_info().is_full());
	}

	#[test]
	fn scaling_limits() {
		let max_verifiers = ::num_cpus::get();
		let queue = get_test_queue(true);
		queue.scale_verifiers(max_verifiers + 1);

		assert!(queue.num_verifiers() < max_verifiers + 1);

		queue.scale_verifiers(0);

		assert!(queue.num_verifiers() == 1);
	}

	#[test]
	fn readjust_verifiers() {
		let queue = get_test_queue(true);

		// put all the verifiers to sleep to ensure
		// the test isn't timing sensitive.
		*queue.state.0.lock() = State::Work(0);

		for block in get_good_dummy_block_seq(5000) {
			queue.import(new_unverified(block)).expect("Block good by definition; qed");
		}

		// almost all unverified == bump verifier count.
		queue.collect_garbage();
		assert_eq!(queue.num_verifiers(), 1);

		queue.flush();

		// nothing to verify == use minimum number of verifiers.
		queue.collect_garbage();
		assert_eq!(queue.num_verifiers(), 1);
	}

		#[test]
		fn worker_threads_honor_specified_number_without_scaling() {
			let spec = spec::new_test();
			let engine = spec.engine;
			let config = get_test_config(1, false);
			let queue = BlockQueue::new(config, engine, IoChannel::disconnected(), true);

			assert_eq!(queue.num_verifiers(), 1);
		}

		#[test]
		fn worker_threads_specified_to_zero_should_set_to_one() {
			let spec = spec::new_test();
			let engine = spec.engine;
			let config = get_test_config(0, false);
			let queue = BlockQueue::new(config, engine, IoChannel::disconnected(), true);

			assert_eq!(queue.num_verifiers(), 1);
		}

		#[test]
		fn worker_threads_should_only_accept_max_number_cpus() {
			let spec = spec::new_test();
			let engine = spec.engine;
			let config = get_test_config(10_000, false);
			let queue = BlockQueue::new(config, engine, IoChannel::disconnected(), true);
			let num_cpus = ::num_cpus::get();

			assert_eq!(queue.num_verifiers(), num_cpus);
		}

		#[test]
		fn worker_threads_scaling_with_specifed_num_of_workers() {
			let num_cpus = ::num_cpus::get();
			// only run the test with at least 2 CPUs
			if num_cpus > 1 {
				let spec = spec::new_test();
				let engine = spec.engine;
				let config = get_test_config(num_cpus - 1, true);
				let queue = BlockQueue::new(config, engine, IoChannel::disconnected(), true);
				queue.scale_verifiers(num_cpus);

				assert_eq!(queue.num_verifiers(), num_cpus);
			}
		}
}
