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

use std::cell::RefCell;
use std::{fs, str, thread};
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::path::PathBuf;
use std::io::{self, Write};
use std::collections::HashMap;

use mio;
use tlsclient::{TlsClient, TlsClientError};

use url::Url;

#[derive(Debug)]
pub enum FetchError {
	InvalidAddress,
	ReadingCaCertificates,
	UnexpectedStatus(String),
	CaCertificates(io::Error),
	Io(io::Error),
	Notify(mio::NotifyError<ClientMessage>),
	Client(TlsClientError),
}

impl From<io::Error> for FetchError {
	fn from(e: io::Error) -> Self {
		FetchError::Io(e)
	}
}

impl From<mio::NotifyError<ClientMessage>> for FetchError {
	fn from(e: mio::NotifyError<ClientMessage>) -> Self {
		FetchError::Notify(e)
	}
}

impl From<TlsClientError> for FetchError {
	fn from(e: TlsClientError) -> Self {
		FetchError::Client(e)
	}
}

pub type FetchResult = Result<(), FetchError>;

pub enum ClientMessage {
	Fetch(Url, Box<io::Write + Send>, Arc<AtomicBool>, Box<FnMut(FetchResult) + Send>),
	Shutdown,
}

pub struct Client {
	channel: mio::Sender<ClientMessage>,
	thread: Option<thread::JoinHandle<()>>,
}

impl Drop for Client {
	fn drop(&mut self) {
		self.close_internal();
		if let Some(thread) = self.thread.take() {
			thread.join().expect("Clean shutdown.");
		}
	}
}

impl Client {
	pub fn new() -> Result<Self, FetchError> {
		Self::with_limit(None)
	}

	pub fn with_limit(size_limit: Option<usize>) -> Result<Self, FetchError> {
		let mut event_loop = try!(mio::EventLoop::new());
		let channel = event_loop.channel();

		let thread = thread::spawn(move || {
			let mut client = ClientLoop {
				next_token: 0,
				sessions: HashMap::new(),
				size_limit: size_limit,
			};
			event_loop.run(&mut client).unwrap();
		});

		Ok(Client {
			channel: channel,
			thread: Some(thread),
		})
	}

	pub fn fetch_to_file<F: FnOnce(FetchResult) + Send + 'static>(&self, url: Url, path: PathBuf, abort: Arc<AtomicBool>, callback: F) -> Result<(), FetchError> {
		let file = try!(fs::File::create(&path));
		self.fetch(url, Box::new(file), abort, move |result| {
			if let Err(_) = result {
				// remove temporary file
				let _ = fs::remove_file(&path);
			}
			callback(result);
		})
	}

	pub fn fetch<F: FnOnce(FetchResult) + Send + 'static>(&self, url: Url, writer: Box<io::Write + Send>, abort: Arc<AtomicBool>, callback: F) -> Result<(), FetchError> {
		let cell = RefCell::new(Some(callback));
		try!(self.channel.send(ClientMessage::Fetch(url, writer, abort, Box::new(move |res| {
			cell.borrow_mut().take().expect("Called only once.")(res);
		}))));
		Ok(())
	}

	pub fn close(mut self) {
		self.close_internal()
	}

	fn close_internal(&mut self) {
		if let Err(e) = self.channel.send(ClientMessage::Shutdown) {
			warn!("Error while closing client: {:?}. Already stopped?", e);
		}
	}
}

pub struct ClientLoop {
	next_token: usize,
	sessions: HashMap<usize, TlsClient>,
	size_limit: Option<usize>,
}

impl mio::Handler for ClientLoop {
	type Timeout = ();
	type Message = ClientMessage;

	fn ready(&mut self, event_loop: &mut mio::EventLoop<ClientLoop>, token: mio::Token, events: mio::EventSet) {
		let utoken = token.as_usize();
		let remove = if let Some(mut tlsclient) = self.sessions.get_mut(&utoken) {
			tlsclient.ready(event_loop, token, events)
		} else {
			false
		};

		if remove {
			self.sessions.remove(&utoken);
		}
	}

	fn notify(&mut self, event_loop: &mut mio::EventLoop<Self>, msg: Self::Message) {
		match msg {
			ClientMessage::Shutdown => event_loop.shutdown(),
			ClientMessage::Fetch(url, writer, abort, callback) => {
				let token = self.next_token;
				self.next_token += 1;

				if let Ok(mut tlsclient) = TlsClient::new(mio::Token(token), &url, writer, abort, callback, self.size_limit.clone()) {
					let httpreq = format!(
						"GET {} HTTP/1.1\r\nHost: {}\r\nConnection: close\r\nUser-Agent: {}/{}\r\nAccept-Encoding: identity\r\n\r\n",
						url.path(),
						url.hostname(),
						env!("CARGO_PKG_NAME"),
						env!("CARGO_PKG_VERSION")
					);
					debug!("Requesting content: {}", httpreq);
					let _ = tlsclient.write(httpreq.as_bytes());
					tlsclient.register(event_loop);

					self.sessions.insert(token, tlsclient);
				}
			}
		}
	}
}

#[test]
fn should_successfuly_fetch_a_page() {
	use std::io::{self, Cursor};
	use std::sync::{mpsc, Arc};
	use std::sync::atomic::{AtomicUsize, AtomicBool, Ordering};

	struct Writer {
		wrote: Arc<AtomicUsize>,
		data: Cursor<Vec<u8>>,
	}

	impl io::Write for Writer {
		fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
			let res = self.data.write(buf);
			if let Ok(count) = res {
				self.wrote.fetch_add(count, Ordering::Relaxed);
			}
			res
		}
		fn flush(&mut self) -> io::Result<()> { Ok(()) }
	}

	let client = Client::new().unwrap();

	let wrote = Arc::new(AtomicUsize::new(0));
	let writer = Writer {
		wrote: wrote.clone(),
		data: Cursor::new(Vec::new()),
	};
	let (tx, rx) = mpsc::channel();
	client.fetch(Url::new("github.com", 443, "/").unwrap(), Box::new(writer), Arc::new(AtomicBool::new(false)), move |result| {
		assert!(result.is_ok());
		assert!(wrote.load(Ordering::Relaxed) > 0);
		tx.send(result).unwrap();
	}).unwrap();
	let _ = rx.recv().unwrap();
}

