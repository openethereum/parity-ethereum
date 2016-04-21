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
use hyper::Client;
use hyper::header::Connection;
use std::str::FromStr;

pub struct PriceInfo {
	pub ethusd: f32,
}

impl PriceInfo {
	pub fn get() -> Option<PriceInfo> {
		let mut body = String::new();
		// TODO: Handle each error type properly
		Client::new()
			.get("http://api.etherscan.io/api?module=stats&action=ethprice")
			.header(Connection::close())
			.send().ok()
			.and_then(|mut s| s.read_to_string(&mut body).ok())
			.and_then(|_| Json::from_str(&body).ok())
			.and_then(|json| json.find_path(&["result", "ethusd"])
				.and_then(|obj| match *obj {
					Json::String(ref s) => Some(PriceInfo {
						ethusd: FromStr::from_str(&s).unwrap()
					}),
					_ => None
				}))
	}
}
