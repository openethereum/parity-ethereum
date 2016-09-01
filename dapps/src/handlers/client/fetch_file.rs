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

//! Hyper Client Handler to Fetch File

use std::{env, io, fs, fmt};
use std::path::PathBuf;
use std::sync::{mpsc, Arc};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use random_filename;

use hyper::status::StatusCode;
use hyper::client::{Request, Response, DefaultTransport as HttpStream};
use hyper::header::Connection;
use hyper::{self, Decoder, Encoder, Next};

#[derive(Debug)]
pub enum Error {
	Aborted,
	NotStarted,
	UnexpectedStatus(StatusCode),
	IoError(io::Error),
	HyperError(hyper::Error),
}

pub type FetchResult = Result<PathBuf, Error>;
pub type OnDone = Box<Fn() + Send>;

pub struct Fetch {
	path: PathBuf,
	abort: Arc<AtomicBool>,
	file: Option<fs::File>,
	result: Option<FetchResult>,
	sender: mpsc::Sender<FetchResult>,
	on_done: Option<OnDone>,
}

impl fmt::Debug for Fetch {
	fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
		write!(f, "Fetch {{ path: {:?}, file: {:?}, result: {:?} }}", self.path, self.file, self.result)
	}
}

impl Drop for Fetch {
    fn drop(&mut self) {
		let res = self.result.take().unwrap_or(Err(Error::NotStarted));
		// Remove file if there was an error
		if res.is_err() || self.is_aborted() {
			if let Some(file) = self.file.take() {
				drop(file);
				// Remove file
				let _ = fs::remove_file(&self.path);
			}
		}
		// send result
		let _ = self.sender.send(res);
		if let Some(f) = self.on_done.take() {
			f();
		}
    }
}

impl Fetch {
	pub fn new(sender: mpsc::Sender<FetchResult>, abort: Arc<AtomicBool>, on_done: OnDone) -> Self {
		let mut dir = env::temp_dir();
		dir.push(random_filename());

		Fetch {
			path: dir,
			abort: abort,
			file: None,
			result: None,
			sender: sender,
			on_done: Some(on_done),
		}
	}
}

impl Fetch {
	fn is_aborted(&self) -> bool {
		self.abort.load(Ordering::Relaxed)
	}
	fn mark_aborted(&mut self) -> Next {
		self.result = Some(Err(Error::Aborted));
		Next::end()
	}
}

impl hyper::client::Handler<HttpStream> for Fetch {
    fn on_request(&mut self, req: &mut Request) -> Next {
		if self.is_aborted() {
			return self.mark_aborted();
		}
        req.headers_mut().set(Connection::close());
        read()
    }

    fn on_request_writable(&mut self, _encoder: &mut Encoder<HttpStream>) -> Next {
		if self.is_aborted() {
			return self.mark_aborted();
		}
        read()
    }

    fn on_response(&mut self, res: Response) -> Next {
		if self.is_aborted() {
			return self.mark_aborted();
		}
		if *res.status() != StatusCode::Ok {
			self.result = Some(Err(Error::UnexpectedStatus(*res.status())));
			return Next::end();
		}

		// Open file to write
		match fs::File::create(&self.path) {
			Ok(file) => {
				self.file = Some(file);
				self.result = Some(Ok(self.path.clone()));
				read()
			},
			Err(err) => {
				self.result = Some(Err(Error::IoError(err)));
				Next::end()
			},
		}
    }

    fn on_response_readable(&mut self, decoder: &mut Decoder<HttpStream>) -> Next {
		if self.is_aborted() {
			return self.mark_aborted();
		}
        match io::copy(decoder, self.file.as_mut().expect("File is there because on_response has created it.")) {
            Ok(0) => Next::end(),
            Ok(_) => read(),
            Err(e) => match e.kind() {
                io::ErrorKind::WouldBlock => Next::read(),
                _ => {
					self.result = Some(Err(Error::IoError(e)));
                    Next::end()
                }
            }
        }
    }

    fn on_error(&mut self, err: hyper::Error) -> Next {
		self.result = Some(Err(Error::HyperError(err)));
        Next::remove()
    }
}

fn read() -> Next {
    Next::read().timeout(Duration::from_secs(15))
}
