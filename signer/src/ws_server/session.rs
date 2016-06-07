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

//! Session handlers factory.

use ws;
use sysui;
use authcode_store::AuthCodes;
use std::path::{PathBuf, Path};
use std::sync::Arc;
use std::str::FromStr;
use jsonrpc_core::IoHandler;
use util::H256;

fn origin_is_allowed(self_origin: &str, header: Option<&Vec<u8>>) -> bool {
	match header {
		None => false,
		Some(h) => {
			let v = String::from_utf8(h.clone()).ok();
			match v {
				Some(ref origin) if origin.starts_with("chrome-extension://") => true,
				Some(ref origin) if origin.starts_with(self_origin) => true,
				Some(ref origin) if origin.starts_with(&format!("http://{}", self_origin)) => true,
				_ => false
			}
		}
	}
}

fn auth_is_valid(codes: &Path, protocols: ws::Result<Vec<&str>>) -> bool {
	match protocols {
		Ok(ref protocols) if protocols.len() == 1 => {
			protocols.iter().any(|protocol| {
				let mut split = protocol.split('_');
				let auth = split.next().and_then(|v| H256::from_str(v).ok());
				let time = split.next().and_then(|v| u64::from_str_radix(v, 10).ok());

				if let (Some(auth), Some(time)) = (auth, time) {
					// Check if the code is valid
					AuthCodes::from_file(codes)
						.map(|codes| codes.is_valid(&auth, time))
						.unwrap_or(false)
				} else {
					false
				}
			})
		},
		_ => false
	}
}

pub struct Session {
	out: ws::Sender,
	self_origin: String,
	authcodes_path: PathBuf,
	handler: Arc<IoHandler>,
}

impl ws::Handler for Session {
	fn on_request(&mut self, req: &ws::Request) -> ws::Result<(ws::Response)> {
		let origin = req.header("origin").or_else(|| req.header("Origin"));
		let host = req.header("host").or_else(|| req.header("Host"));

		// Check request origin and host header.
		if !origin_is_allowed(&self.self_origin, origin) && !origin_is_allowed(&self.self_origin, host) {
			warn!(target: "signer", "Blocked connection to Signer API from untrusted origin.");
			return Ok(ws::Response::forbidden("You are not allowed to access system ui.".into()));
		}

		// Detect if it's a websocket request.
		if req.header("sec-websocket-key").is_some() {
			// Check authorization
			if !auth_is_valid(&self.authcodes_path, req.protocols()) {
				info!(target: "signer", "Unauthorized connection to Signer API blocked.");
				return Ok(ws::Response::forbidden("You are not authorized.".into()));
			}

			let protocols = req.protocols().expect("Existence checked by authorization.");
			let protocol = protocols.get(0).expect("Proved by authorization.");
			return ws::Response::from_request(req).map(|mut res| {
				// To make WebSockets connection successful we need to send back the protocol header.
				res.set_protocol(protocol);
				res
			});
		}

		// Otherwise try to serve a page.
		sysui::handle(req.resource())
			.map_or_else(
				// return error
				|| Ok(ws::Response::not_found("Page not found".into())),
				// or serve the file
				|f| {
					let content_len = format!("{}", f.content.as_bytes().len());
					let mut res = ws::Response::ok(f.content.into());
					{
						let mut headers = res.headers_mut();
						headers.push(("Server".into(), b"Parity/SystemUI".to_vec()));
						headers.push(("Connection".into(), b"Closed".to_vec()));
						headers.push(("Content-Length".into(), content_len.as_bytes().to_vec()));
						headers.push(("Content-Type".into(), f.mime.as_bytes().to_vec()));
						if !f.safe_to_embed {
							headers.push(("X-Frame-Options".into(), b"SAMEORIGIN".to_vec()));
						}
					}
					Ok(res)
				})
	}

	fn on_message(&mut self, msg: ws::Message) -> ws::Result<()> {
		let req = try!(msg.as_text());
		match self.handler.handle_request(req) {
			Some(res) => self.out.send(res),
			None => Ok(()),
		}
	}
}

pub struct Factory {
	handler: Arc<IoHandler>,
	self_origin: String,
	authcodes_path: PathBuf,
}

impl Factory {
	pub fn new(handler: Arc<IoHandler>, self_origin: String, authcodes_path: PathBuf) -> Self {
		Factory {
			handler: handler,
			self_origin: self_origin,
			authcodes_path: authcodes_path,
		}
	}
}

impl ws::Factory for Factory {
	type Handler = Session;

	fn connection_made(&mut self, sender: ws::Sender) -> Self::Handler {
		Session {
			out: sender,
			handler: self.handler.clone(),
			self_origin: self.self_origin.clone(),
			authcodes_path: self.authcodes_path.clone(),
		}
	}
}
