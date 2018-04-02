// Copyright 2015-2018 Parity Technologies (UK) Ltd.
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

extern crate fetch;
extern crate hyper;
extern crate futures;

use hyper::StatusCode;
use futures::{future, future::FutureResult};
use fetch::{Fetch, Url, Method};


#[derive(Clone, Default)]
pub struct FakeFetch {
	success: bool,
}

impl FakeFetch {
	pub fn new(b: bool) -> Self {
		FakeFetch { success: b }
	}
}

impl Fetch for FakeFetch {
	type Result = FutureResult<fetch::Response, fetch::Error>;

	fn fetch(&self, url: &str, _method: Method, abort: fetch::Abort) -> Self::Result {
		let u = Url::parse(url).unwrap();
		future::ok(if self.success {
			let r = hyper::Response::new().with_body(&b"Some content"[..]);
			fetch::client::Response::new(u, r, abort)
		} else {
			fetch::client::Response::new(u, hyper::Response::new().with_status(StatusCode::NotFound), abort)
		})
	}

	fn get(&self, url: &str, abort: fetch::Abort) -> Self::Result {
		let url: Url = match url.parse() {
			Ok(u) => u,
			Err(e) => return future::err(e.into())
		};
		self.fetch(Request::get(url), abort)
	}

	fn post(&self, url: &str, abort: fetch::Abort) -> Self::Result {
		let url: Url = match url.parse() {
			Ok(u) => u,
			Err(e) => return future::err(e.into())
		};
		self.fetch(Request::post(url), abort)
	}
}
