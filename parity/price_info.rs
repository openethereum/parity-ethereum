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
		// TODO: actually bother dicking around with the errors and make it proper.
		match match match Client::new()
			.get("http://api.etherscan.io/api?module=stats&action=ethprice")
			.header(Connection::close())
			.send() {
			Err(_) => { return None; },
			Ok(mut s) => s.read_to_string(&mut body),
		} {
			Err(_) => { return None; },
			_ => { Json::from_str(&body) }
		} {
			Err(_) => { return None; },
			Ok(json) => {
				let ethusd: f32 = if let Some(&Json::String(ref s)) = json.find_path(&["result", "ethusd"]) {
					FromStr::from_str(&s).unwrap()
				} else {
					return None;
				};
				Some(PriceInfo {
					ethusd: ethusd,
				})
			}
		}
	}
}

