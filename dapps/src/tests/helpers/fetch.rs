// Copyright 2015, 2016 Parity Technologies (UK) Ltd.
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

use std::{io, thread};
use std::sync::Arc;
use std::sync::atomic::{self, AtomicUsize};
use util::Mutex;

use futures::{self, Future};
use fetch::{self, Fetch};

#[derive(Clone, Default)]
pub struct FakeFetch {
	asserted: Arc<AtomicUsize>,
	requested: Arc<Mutex<Vec<String>>>,
}

impl FakeFetch {
	pub fn assert_requested(&self, url: &str) {
		let requests = self.requested.lock();
		let idx = self.asserted.fetch_add(1, atomic::Ordering::SeqCst);

		assert_eq!(requests.get(idx), Some(&url.to_owned()), "Expected fetch from specific URL.");
	}

	pub fn assert_no_more_requests(&self) {
		let requests = self.requested.lock();
		let len = self.asserted.load(atomic::Ordering::SeqCst);
		assert_eq!(requests.len(), len, "Didn't expect any more requests, got: {:?}", &requests[len..]);
	}
}

impl Fetch for FakeFetch {
	type Result = futures::BoxFuture<fetch::Response, fetch::Error>;

	fn new() -> Result<Self, fetch::Error> where Self: Sized {
		Ok(FakeFetch::default())
	}

	fn fetch_with_abort(&self, url: &str, _abort: fetch::Abort) -> Self::Result {
		self.requested.lock().push(url.into());

		let (tx, rx) = futures::oneshot();
		thread::spawn(move || {
			let cursor = io::Cursor::new(b"Some content");
			tx.complete(fetch::Response::from_reader(cursor));
		});

		rx.map_err(|_| fetch::Error::Aborted).boxed()
	}
}
