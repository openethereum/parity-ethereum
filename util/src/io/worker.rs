use std::sync::*;
use std::mem;
use std::thread::{JoinHandle, self};
use std::sync::atomic::{AtomicBool, Ordering as AtomicOrdering};
use crossbeam::sync::chase_lev;
use io::service::{HandlerId, IoChannel, IoContext};
use io::{IoHandler};

pub enum WorkType<Message> {
	Readable,
	Writable,
	Hup,
	Timeout,
	Message(Message)
}

pub struct Work<Message> {
	pub work_type: WorkType<Message>,
	pub token: usize,
	pub handler_id: HandlerId,
	pub handler: Arc<IoHandler<Message>>,
}

/// An IO worker thread
/// Sorts them ready for blockchain insertion.
pub struct Worker {
	thread: Option<JoinHandle<()>>,
	wait: Arc<Condvar>,
	deleting: Arc<AtomicBool>,
}

impl Worker {
	/// Creates a new worker instance.
	pub fn new<Message>(index: usize, 
						stealer: chase_lev::Stealer<Work<Message>>, 
						channel: IoChannel<Message>,
						wait: Arc<Condvar>,
						wait_mutex: Arc<Mutex<bool>>) -> Worker 
						where Message: Send + Sync + Clone + 'static {
		let deleting = Arc::new(AtomicBool::new(false));
		let mut worker = Worker {
			thread: None,
			wait: wait.clone(),
			deleting: deleting.clone(),
		};
		worker.thread = Some(thread::Builder::new().name(format!("IO Worker #{}", index)).spawn(
			move || Worker::work_loop(stealer, channel.clone(), wait, wait_mutex.clone(), deleting))
			.expect("Error creating worker thread"));
		worker
	}

	fn work_loop<Message>(stealer: chase_lev::Stealer<Work<Message>>,
						channel: IoChannel<Message>, wait: Arc<Condvar>, 
						wait_mutex: Arc<Mutex<bool>>, 
						deleting: Arc<AtomicBool>) 
						where Message: Send + Sync + Clone + 'static {
		while !deleting.load(AtomicOrdering::Relaxed) {
			{
				let lock = wait_mutex.lock().unwrap();
				let _ = wait.wait(lock).unwrap();
				if deleting.load(AtomicOrdering::Relaxed) {
					return;
				}
			}
			loop {
				match stealer.steal() {
					chase_lev::Steal::Data(work) => {
						Worker::do_work(work, channel.clone());
					}
					_ => break
				}
			}
		}
	}

	fn do_work<Message>(work: Work<Message>, channel: IoChannel<Message>) where Message: Send + Sync + Clone + 'static {
		match work.work_type {
			WorkType::Readable => {
				work.handler.stream_readable(&mut IoContext::new(channel, work.handler_id), work.token);
			},
			WorkType::Writable => {
				work.handler.stream_writable(&mut IoContext::new(channel, work.handler_id), work.token);
			}
			WorkType::Hup => {
				work.handler.stream_hup(&mut IoContext::new(channel, work.handler_id), work.token);
			}
			WorkType::Timeout => {
				work.handler.timeout(&mut IoContext::new(channel, work.handler_id), work.token);
			}
			WorkType::Message(message) => {
				work.handler.message(&mut IoContext::new(channel, work.handler_id), &message);
			}
		}
	}
}

impl Drop for Worker {
	fn drop(&mut self) {
		self.deleting.store(true, AtomicOrdering::Relaxed);
		self.wait.notify_all();
		let thread = mem::replace(&mut self.thread, None).unwrap();
		thread.join().unwrap();
	}
}
