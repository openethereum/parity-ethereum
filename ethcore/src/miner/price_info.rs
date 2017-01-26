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

use rustc_serialize::json::Json;
use std::thread;
use std::io::Read;
use std::time::Duration;
use std::str::FromStr;
use std::sync::mpsc;
use hyper::client::{Handler, Request, Response, Client};
use hyper::{Url, Next, Encoder, Decoder};
use hyper::net::HttpStream;

#[derive(Debug)]
pub struct PriceInfo {
	pub ethusd: f32,
}

pub struct SetPriceHandler<F> {
	set_price: F,
	channel: mpsc::Sender<()>,
}

impl<F> Drop for SetPriceHandler<F> {
	fn drop(&mut self) {
		let _ = self.channel.send(());
	}
}

impl<F: Fn(PriceInfo) + Sync + Send + 'static> Handler<HttpStream> for SetPriceHandler<F> {
	fn on_request(&mut self, _: &mut Request) -> Next { Next::read().timeout(Duration::from_secs(3)) }
	fn on_request_writable(&mut self, _: &mut Encoder<HttpStream>) -> Next { Next::read().timeout(Duration::from_secs(3)) }
	fn on_response(&mut self, _: Response) -> Next { Next::read().timeout(Duration::from_secs(3)) }

	fn on_response_readable(&mut self, r: &mut Decoder<HttpStream>) -> Next {
		let mut body = String::new();
		let info = r.read_to_string(&mut body)
			.map_err(|e| format!("Unable to read response: {:?}", e))
			.and_then(|_| self.process_response(&body));

		if let Err(e) = info {
			warn!("Failed to auto-update latest ETH price: {:?}", e);
		}
		Next::end()
	}
}

impl<F: Fn(PriceInfo) + Sync + Send + 'static> SetPriceHandler<F> {
	fn process_response(&self, body: &str) -> Result<(), String> {
		let json = Json::from_str(body).map_err(|e| format!("Invalid JSON returned: {:?}", e))?;
		let obj = json.find_path(&["result", "ethusd"]).ok_or("USD price not found".to_owned())?;
		let ethusd = match *obj {
			Json::String(ref s) => FromStr::from_str(s).ok(),
			_ => None,
		}.ok_or("Unexpected price format.".to_owned())?;

		(self.set_price)(PriceInfo {
			ethusd: ethusd,
		});
		Ok(())
	}
}

impl PriceInfo {
	pub fn get<F: Fn(PriceInfo) + Sync + Send + 'static>(set_price: F) {
		thread::spawn(move || {
			let url = FromStr::from_str("http://api.etherscan.io/api?module=stats&action=ethprice")
				.expect("string known to be a valid URL; qed");

			if let Err(e) = Self::request(url, set_price) {
				warn!("Failed to auto-update latest ETH price: {:?}", e);
			}
		});
	}

	fn request<F: Fn(PriceInfo) + Send + Sync + 'static>(url: Url, set_price: F) -> Result<(), String> {
		let (tx, rx) = mpsc::channel();
		let client = Client::new().map_err(|e| format!("Unable to start client: {:?}", e))?;

		client.request(
			url,
			SetPriceHandler {
				set_price: set_price,
				channel: tx,
			},
		).map_err(|_| "Request failed.".to_owned())?;

		// Wait for exit
		let _ = rx.recv().map_err(|e| format!("Request interrupted: {:?}", e))?;
		client.close();

		Ok(())
	}
}

#[test] #[ignore]
fn should_get_price_info() {
	use std::sync::Arc;
	use std::time::Duration;
	use util::log::init_log;
	use util::{Condvar, Mutex};

	init_log();
	let done = Arc::new((Mutex::new(PriceInfo { ethusd: 0f32 }), Condvar::new()));
	let rdone = done.clone();

	PriceInfo::get(move |price| { let mut p = rdone.0.lock(); *p = price; rdone.1.notify_one(); });
	let mut p = done.0.lock();
	let t = done.1.wait_for(&mut p, Duration::from_millis(10000));
	assert!(!t.timed_out());
	assert!(p.ethusd != 0f32);
}
