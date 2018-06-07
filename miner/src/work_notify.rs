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

//! Sends HTTP notifications to a list of URLs every time new work is available.

extern crate ethash;
extern crate fetch;
extern crate parity_reactor;
extern crate url;
extern crate hyper;

use self::fetch::{Fetch, Request, Client as FetchClient, Method};
use self::parity_reactor::Remote;
use self::ethash::SeedHashCompute;
use self::url::Url;
use self::hyper::header::ContentType;

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
	remote: Remote,
	seed_compute: Mutex<SeedHashCompute>,
}

impl WorkPoster {
	/// Create new `WorkPoster`.
	pub fn new(urls: &[String], fetch: FetchClient, remote: Remote) -> Self {
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
			remote: remote,
			urls: urls,
			seed_compute: Mutex::new(SeedHashCompute::new()),
		}
	}
}

/// Convert an Ethash difficulty to the target boundary. Basically just `f(x) = 2^256 / x`.
fn difficulty_to_boundary(difficulty: &U256) -> H256 {
	if *difficulty <= U256::one() {
		U256::max_value().into()
	} else {
		(((U256::one() << 255) / *difficulty) << 1).into()
	}
}

impl NotifyWork for WorkPoster {
	fn notify(&self, pow_hash: H256, difficulty: U256, number: u64) {
		// TODO: move this to engine
		let target = difficulty_to_boundary(&difficulty);
		let seed_hash = &self.seed_compute.lock().hash_block_number(number);
		let seed_hash = H256::from_slice(&seed_hash[..]);
		let body = format!(
			r#"{{ "result": ["0x{:x}","0x{:x}","0x{:x}","0x{:x}"] }}"#,
			pow_hash, seed_hash, target, number
		);

		for u in &self.urls {
			let u = u.clone();
			self.remote.spawn(self.client.fetch(
				Request::new(u.clone(), Method::Post)
					.with_header(ContentType::json())
					.with_body(body.clone()), Default::default()
			).map_err(move |e| {
				warn!("Error sending HTTP notification to {} : {}, retrying", u, e);
			}).map(|_| ()));
		}
	}
}
