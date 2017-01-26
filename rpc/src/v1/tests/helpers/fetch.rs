// Copyright 2015-2017 Parity Technologies (UK) Ltd.
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

//! Test implementation of fetch client.

use std::{io, thread};
use futures::{self, Future};
use fetch::{self, Fetch};

/// Test implementation of fetcher. Will always return the same file.
#[derive(Default, Clone)]
pub struct TestFetch;

impl Fetch for TestFetch {
	type Result = futures::BoxFuture<fetch::Response, fetch::Error>;

	fn new() -> Result<Self, fetch::Error> where Self: Sized {
		Ok(TestFetch)
	}

	fn fetch_with_abort(&self, _url: &str, _abort: fetch::Abort) -> Self::Result {
		let (tx, rx) = futures::oneshot();
		thread::spawn(move || {
			let cursor = io::Cursor::new(b"Some content");
			tx.complete(fetch::Response::from_reader(cursor));
		});

		rx.map_err(|_| fetch::Error::Aborted).boxed()
	}
}
