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

use std::str;
use std::thread;
use std::sync::mpsc;
use std::io::{self, Write};
use std::collections::HashMap;

use mio;
use tlsclient::{TlsClient, TlsClientError};

use url::Url;

#[derive(Debug)]
pub enum FetchError {
	InvalidAddress,
	ReadingCaCertificates,
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
	Fetch(Url, Box<io::Write + Send>, mpsc::Sender<FetchResult>),
	Shutdown,
}

pub struct Client {
	channel: mio::Sender<ClientMessage>,
	thread: Option<thread::JoinHandle<()>>,
}

impl Drop for Client {
	fn drop(&mut self) {
		if let Err(e) = self.channel.send(ClientMessage::Shutdown) {
			warn!("Error while closing client: {:?}. Already stopped?", e);
		}
		if let Some(thread) = self.thread.take() {
			thread.join().expect("Clean shutdown.");
		}
	}
}

impl Client {
	pub fn new() -> Result<Self, FetchError> {
		let mut event_loop = try!(mio::EventLoop::new());
		let channel = event_loop.channel();

		let thread = thread::spawn(move || {
			let mut client = ClientLoop {
				next_token: 0,
				sessions: HashMap::new(),
			};
			event_loop.run(&mut client).unwrap();
		});

		Ok(Client {
			channel: channel,
			thread: Some(thread),
		})
	}

	pub fn fetch(&self, url: Url, writer: Box<io::Write + Send>) -> Result<mpsc::Receiver<FetchResult>, FetchError> {
		let (tx, rx) = mpsc::channel();
		try!(self.channel.send(ClientMessage::Fetch(url, writer, tx)));
		Ok(rx)
	}
}

pub struct ClientLoop {
	next_token: usize,
	sessions: HashMap<usize, TlsClient>,
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
			ClientMessage::Fetch(url, writer, sender) => {
				let token = self.next_token;
				self.next_token += 1;

				if let Ok(mut tlsclient) = TlsClient::new(mio::Token(token), &url, writer, sender) {
					let httpreq = format!(
						"GET {} HTTP/1.1\r\nHost: {}\r\nConnection: close\r\nAccept-Encoding: identity\r\n\r\n",
						url.path(),
						url.hostname()
					);
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
	use std::sync::Arc;
	use std::sync::atomic::{AtomicUsize, Ordering};

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
	let rx = client.fetch(Url::new("github.com", 443, "/").unwrap(), Box::new(writer)).unwrap();

	let result = rx.recv().unwrap();

	assert!(result.is_ok());
	assert!(wrote.load(Ordering::Relaxed) > 0);
}
