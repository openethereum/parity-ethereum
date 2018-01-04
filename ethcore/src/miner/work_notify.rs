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

extern crate hyper;

use std::io::Write;
use hyper::header::ContentType;
use hyper::method::Method;
use hyper::client::{Request, Response, Client};
use hyper::{Next};
use hyper::net::HttpStream;
use ethash::SeedHashCompute;
use hyper::Url;
use ethereum_types::{H256, U256};
use parking_lot::Mutex;
use ethereum::ethash::Ethash;

/// Trait for notifying about new mining work
pub trait NotifyWork : Send + Sync {
	/// Fired when new mining job available
	fn notify(&self, pow_hash: H256, difficulty: U256, number: u64);
}

pub struct WorkPoster {
	urls: Vec<Url>,
	client: Mutex<Client<PostHandler>>,
	seed_compute: Mutex<SeedHashCompute>,
}

impl WorkPoster {
	pub fn new(urls: &[String]) -> Self {
		let urls = urls.into_iter().filter_map(|u| {
			match Url::parse(u) {
				Ok(url) => Some(url),
				Err(e) => {
					warn!("Error parsing URL {} : {}", u, e);
					None
				}
			}
		}).collect();
		let client = WorkPoster::create_client();
		WorkPoster {
			client: Mutex::new(client),
			urls: urls,
			seed_compute: Mutex::new(SeedHashCompute::new()),
		}
	}

	fn create_client() -> Client<PostHandler> {
		Client::<PostHandler>::configure()
			.keep_alive(true)
			.build()
			.expect("Error creating HTTP client")
	}
}

impl NotifyWork for WorkPoster {
	fn notify(&self, pow_hash: H256, difficulty: U256, number: u64) {
		// TODO: move this to engine
		let target = Ethash::difficulty_to_boundary(&difficulty);
		let seed_hash = &self.seed_compute.lock().hash_block_number(number);
		let seed_hash = H256::from_slice(&seed_hash[..]);
		let body = format!(
			r#"{{ "result": ["0x{}","0x{}","0x{}","0x{:x}"] }}"#,
			pow_hash.hex(), seed_hash.hex(), target.hex(), number
		);
		let mut client = self.client.lock();
		for u in &self.urls {
			if let Err(e) = client.request(u.clone(), PostHandler { body: body.clone() }) {
				warn!("Error sending HTTP notification to {} : {}, retrying", u, e);
				// TODO: remove this once https://github.com/hyperium/hyper/issues/848 is fixed
				*client = WorkPoster::create_client();
				if let Err(e) = client.request(u.clone(), PostHandler { body: body.clone() }) {
					warn!("Error sending HTTP notification to {} : {}", u, e);
				}
			}
		}
	}
}

struct PostHandler {
	body: String,
}

impl hyper::client::Handler<HttpStream> for PostHandler {
	fn on_request(&mut self, request: &mut Request) -> Next {
		request.set_method(Method::Post);
		request.headers_mut().set(ContentType::json());
		Next::write()
	}

	fn on_request_writable(&mut self, encoder: &mut hyper::Encoder<HttpStream>) -> Next {
		if let Err(e) = encoder.write_all(self.body.as_bytes()) {
			trace!("Error posting work data: {}", e);
		}
		encoder.close();
		Next::read()

	}

	fn on_response(&mut self, _response: Response) -> Next {
		Next::end()
	}

	fn on_response_readable(&mut self, _decoder: &mut hyper::Decoder<HttpStream>) -> Next {
		Next::end()
	}

	fn on_error(&mut self, err: hyper::Error) -> Next {
		trace!("Error posting work data: {}", err);
		Next::end()
	}
}

