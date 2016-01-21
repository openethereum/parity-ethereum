//! A queue of state changes that are written to database in background.
use std::thread::{JoinHandle, self};
use std::sync::atomic::{AtomicBool, Ordering as AtomicOrdering};
use util::*;
use engine::Engine;
use client::Client;

/// State DB commit params
pub struct StateDBCommit {
	/// Database to commit
	pub db: JournalDB,
	/// Starting block number
	pub now: u64,
	/// Block ahash
	pub hash: H256,
	/// End block number + hash
	pub end: Option<(u64, H256)>,
}

/// A queue of state changes that are written to database in background.
pub struct DbQueue {
	more_to_write: Arc<Condvar>,
	queue: Arc<Mutex<VecDeque<StateDBCommit>>>,
	writer: Mutex<Option<JoinHandle<()>>>,
	deleting: Arc<AtomicBool>,
}

impl DbQueue {
	/// Creates a new queue instance.
	pub fn new() -> DbQueue {
		let queue = Arc::new(Mutex::new(VecDeque::new()));
		let more_to_write = Arc::new(Condvar::new());
		let deleting = Arc::new(AtomicBool::new(false));

		DbQueue {
			more_to_write: more_to_write.clone(),
			queue: queue.clone(),
			writer: Mutex::new(None),
			deleting: deleting.clone(),
		}
	}

	/// Start processing the queue
	pub fn start(&self, client: Weak<Client>) {
		let writer = {
			let queue = self.queue.clone();
			let client = client.clone();
			let more_to_write = self.more_to_write.clone();
			let deleting = self.deleting.clone();
			thread::Builder::new().name("DB Writer".to_string()).spawn(move || DbQueue::writer_loop(client, queue, more_to_write, deleting)).expect("Error creating db writer thread")
		};
		mem::replace(self.writer.lock().unwrap().deref_mut(), Some(writer));
	}

	fn writer_loop(client: Weak<Client>, queue: Arc<Mutex<VecDeque<StateDBCommit>>>, wait: Arc<Condvar>, deleting: Arc<AtomicBool>) {
		while !deleting.load(AtomicOrdering::Relaxed) {
			let mut batch = {
				let mut locked = queue.lock().unwrap();
				while locked.is_empty() && !deleting.load(AtomicOrdering::Relaxed) {
					locked = wait.wait(locked).unwrap();
				}
				
				if deleting.load(AtomicOrdering::Relaxed) {
					return;
				}
				mem::replace(locked.deref_mut(), VecDeque::new())
			};

			for mut state in batch.drain(..) { //TODO: make this a single write transaction
				match state.db.commit(state.now, &state.hash, state.end.clone()) {
					Ok(_) => (),
					Err(e) => {
						warn!(target: "client", "State DB commit failed: {:?}", e);
					}
				}
				client.upgrade().unwrap().clear_state(&state.hash);
			}

		}
	}

	/// Add a state to the queue
	pub fn queue(&self, state: StateDBCommit) {
		let mut queue = self.queue.lock().unwrap();
		queue.push_back(state);
		self.more_to_write.notify_all();
	}
}

impl Drop for DbQueue {
	fn drop(&mut self) {
		self.deleting.store(true, AtomicOrdering::Relaxed);
		self.more_to_write.notify_all();
		mem::replace(self.writer.lock().unwrap().deref_mut(), None).unwrap().join().unwrap();
	}
}

#[cfg(test)]
mod tests {
	use util::*;
	use spec::*;
	use queue::*;

	#[test]
	fn test_block_queue() {
		// TODO better test
		let spec = Spec::new_test();
		let engine = spec.to_engine().unwrap();
		let _ = BlockQueue::new(Arc::new(engine), IoChannel::disconnected());
	}
}
