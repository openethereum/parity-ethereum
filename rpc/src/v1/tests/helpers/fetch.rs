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

use std::thread;
use jsonrpc_core::futures::{self, Future};
use fetch::{self, Fetch, Url, Method};
use hyper;

/// Test implementation of fetcher. Will always return the same file.
#[derive(Default, Clone)]
pub struct TestFetch;

impl Fetch for TestFetch {
	type Result = Box<Future<Item = fetch::Response, Error = fetch::Error> + Send + 'static>;

	fn fetch(&self, url: &str, _method: Method, abort: fetch::Abort) -> Self::Result {
		let u = Url::parse(url).unwrap();
		let (tx, rx) = futures::oneshot();
		thread::spawn(move || {
			let r = hyper::Response::new().with_body(&b"Some content"[..]);
			tx.send(fetch::Response::new(u, r, abort)).unwrap();
		});

		Box::new(rx.map_err(|_| fetch::Error::Aborted))
	}
}
