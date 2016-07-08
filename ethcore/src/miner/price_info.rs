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

use rustc_serialize::json::Json;
use std::io::Read;
use std::time::Duration;
use hyper::client::{Handler, Request, Response, Client};
use hyper::{Next, Encoder, Decoder};
use hyper::net::HttpStream;
use std::str::FromStr;

#[derive(Debug)]
pub struct PriceInfo {
	pub ethusd: f32,
}

pub struct SetPriceHandler<F: Fn(PriceInfo) + Sync + Send + 'static> {
	set_price: F,
}

impl<F: Fn(PriceInfo) + Sync + Send + 'static> Handler<HttpStream> for SetPriceHandler<F> {
	fn on_request(&mut self, _: &mut Request) -> Next { trace!(target: "miner", "price_info: on_request"); Next::read().timeout(Duration::from_secs(3)) }
	fn on_request_writable(&mut self, _: &mut Encoder<HttpStream>) -> Next { trace!(target: "miner", "price_info: on_request_writable"); Next::read().timeout(Duration::from_secs(3)) }
	fn on_response(&mut self, _: Response) -> Next { trace!(target: "miner", "price_info: on_response"); Next::read().timeout(Duration::from_secs(3)) }
	fn on_response_readable(&mut self, r: &mut Decoder<HttpStream>) -> Next {
		trace!(target: "miner", "price_info: on_response_readable!"); 
		let mut body = String::new();
		let _ = r.read_to_string(&mut body).ok()
			.and_then(|_| Json::from_str(&body).ok())
			.and_then(|json| json.find_path(&["result", "ethusd"])
				.and_then(|obj| match *obj {
					Json::String(ref s) => Some((self.set_price)(PriceInfo {
						ethusd: FromStr::from_str(s).unwrap()
					})),
					_ => None,
				}));
		Next::end()
	}

}

impl PriceInfo {
	pub fn get<F: Fn(PriceInfo) + Sync + Send + 'static>(set_price: F) -> Result<(), ()> {
		// TODO: Handle each error type properly
		trace!(target: "miner", "Starting price info request...");
		Client::new().map_err(|_| ()).and_then(|client| {
			client.request(FromStr::from_str("http://api.etherscan.io/api?module=stats&action=ethprice").unwrap(), SetPriceHandler { set_price: set_price }).map_err(|_| ())
		})
	}
}
