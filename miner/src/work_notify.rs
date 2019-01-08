// Copyright 2015-2019 Parity Technologies (UK) Ltd.
// This file is part of Parity Ethereum.

// Parity Ethereum is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity Ethereum is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity Ethereum.  If not, see <http://www.gnu.org/licenses/>.

//! Sends HTTP notifications to a list of URLs every time new work is available.

extern crate ethash;
extern crate fetch;
extern crate parity_runtime;
extern crate url;
extern crate hyper;

use self::fetch::{Fetch, Request, Client as FetchClient, Method};
use self::parity_runtime::Executor;
use self::ethash::SeedHashCompute;
use self::url::Url;
use self::hyper::header::{self, HeaderValue};

use ethereum_types::{H256, U256};
use parking_lot::Mutex;

use futures::Future;

/// Trait for notifying about new mining work
pub trait NotifyWork : Send + Sync {
	/// Fired when new mining job available
	fn notify(&self, pow_hash: H256, difficulty: U256, number: u64);
}

/// POSTs info about new work to given urls.
pub struct WorkPoster {
	urls: Vec<Url>,
	client: FetchClient,
	executor: Executor,
	seed_compute: Mutex<SeedHashCompute>,
}

impl WorkPoster {
	/// Create new `WorkPoster`.
	pub fn new(urls: &[String], fetch: FetchClient, executor: Executor) -> Self {
		let urls = urls.into_iter().filter_map(|u| {
			match Url::parse(u) {
				Ok(url) => Some(url),
				Err(e) => {
					warn!("Error parsing URL {} : {}", u, e);
					None
				}
			}
		}).collect();
		WorkPoster {
			client: fetch,
			executor: executor,
			urls: urls,
			seed_compute: Mutex::new(SeedHashCompute::default()),
		}
	}
}

impl NotifyWork for WorkPoster {
	fn notify(&self, pow_hash: H256, difficulty: U256, number: u64) {
		// TODO: move this to engine
		let target = ethash::difficulty_to_boundary(&difficulty);
		let seed_hash = &self.seed_compute.lock().hash_block_number(number);
		let seed_hash = H256::from_slice(&seed_hash[..]);
		let body = format!(
			r#"{{ "result": ["0x{:x}","0x{:x}","0x{:x}","0x{:x}"] }}"#,
			pow_hash, seed_hash, target, number
		);

		for u in &self.urls {
			let u = u.clone();
			self.executor.spawn(self.client.fetch(
				Request::new(u.clone(), Method::POST)
					.with_header(header::CONTENT_TYPE, HeaderValue::from_static("application/json"))
					.with_body(body.clone()), Default::default()
			).map_err(move |e| {
				warn!("Error sending HTTP notification to {} : {}, retrying", u, e);
			}).map(|_| ()));
		}
	}
}
