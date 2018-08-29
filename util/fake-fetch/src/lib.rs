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

use hyper::{StatusCode, Body};
use futures::{future, future::FutureResult};
use fetch::{Fetch, Url, Request};

#[derive(Clone, Default)]
pub struct FakeFetch<T> where T: Clone + Send + Sync {
	val: Option<T>,
}

impl<T> FakeFetch<T> where T: Clone + Send + Sync {
	pub fn new(t: Option<T>) -> Self {
		FakeFetch { val : t }
	}
}

impl<T: 'static> Fetch for FakeFetch<T> where T: Clone + Send+ Sync {
	type Result = FutureResult<fetch::Response, fetch::Error>;

	fn fetch(&self, request: Request, abort: fetch::Abort) -> Self::Result {
		let u = request.url().clone();
		future::ok(if self.val.is_some() {
			let r = hyper::Response::new("Some content".into());
			fetch::client::Response::new(u, r, abort)
		} else {
			let r = hyper::Response::builder()
				.status(StatusCode::NOT_FOUND)
				.body(Body::empty()).expect("Nothing to parse, can not fail; qed");
			fetch::client::Response::new(u, r, abort)
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
