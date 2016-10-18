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
use authcode_store::AuthCodes;
use std::path::{PathBuf, Path};
use std::sync::Arc;
use std::str::FromStr;
use jsonrpc_core::IoHandler;
use util::{H256, Mutex, version};

#[cfg(feature = "ui")]
mod ui {
	extern crate parity_ui as ui;
	extern crate parity_dapps_glue as dapps;

	use self::dapps::WebApp;

	#[derive(Default)]
	pub struct Handler {
		ui: ui::App,
	}

	impl Handler {
		pub fn handle(&self, req: &str) -> Option<&dapps::File> {
			let file = match req {
				"" | "/" => "index.html",
				path => &path[1..],
			};
			self.ui.file(file)
		}
	}
}
#[cfg(not(feature = "ui"))]
mod ui {
	pub struct File {
		pub content: &'static [u8],
		pub content_type: &'static str,
	}

	#[derive(Default)]
	pub struct Handler;

	impl Handler {
		pub fn handle(&self, _req: &str) -> Option<&File> {
			None
		}
	}
}

fn origin_is_allowed(self_origin: &str, header: Option<&[u8]>) -> bool {
	match header {
		None => false,
		Some(h) => {
			let v = String::from_utf8(h.to_owned()).ok();
			match v {
				Some(ref origin) if origin.starts_with("chrome-extension://") => true,
				Some(ref origin) if origin.starts_with(self_origin) => true,
				Some(ref origin) if origin.starts_with(&format!("http://{}", self_origin)) => true,
				_ => false
			}
		}
	}
}

fn auth_is_valid(codes_path: &Path, protocols: ws::Result<Vec<&str>>) -> bool {
	match protocols {
		Ok(ref protocols) if protocols.len() == 1 => {
			protocols.iter().any(|protocol| {
				let mut split = protocol.split('_');
				let auth = split.next().and_then(|v| H256::from_str(v).ok());
				let time = split.next().and_then(|v| u64::from_str_radix(v, 10).ok());

				if let (Some(auth), Some(time)) = (auth, time) {
					// Check if the code is valid
					AuthCodes::from_file(codes_path)
						.map(|mut codes| {
							let res = codes.is_valid(&auth, time);
							// make sure to save back authcodes - it might have been modified
							if let Err(_) = codes.to_file(codes_path) {
								warn!(target: "signer", "Couldn't save authorization codes to file.");
							}
							res
						})
						.unwrap_or(false)
				} else {
					false
				}
			})
		},
		_ => false
	}
}

fn add_headers(mut response: ws::Response, mime: &str) -> ws::Response {
	let content_len = format!("{}", response.len());
	{
		let mut headers = response.headers_mut();
		headers.push(("X-Frame-Options".into(), b"SAMEORIGIN".to_vec()));
		headers.push(("Server".into(), b"Parity/SignerUI".to_vec()));
		headers.push(("Content-Length".into(), content_len.as_bytes().to_vec()));
		headers.push(("Content-Type".into(), mime.as_bytes().to_vec()));
		headers.push(("Connection".into(), b"close".to_vec()));
	}
	response
}

pub struct Session {
	out: Arc<Mutex<ws::Sender>>,
	skip_origin_validation: bool,
	self_origin: String,
	authcodes_path: PathBuf,
	handler: Arc<IoHandler>,
	file_handler: Arc<ui::Handler>,
}

impl ws::Handler for Session {
	#[cfg_attr(feature="dev", allow(collapsible_if))]
	fn on_request(&mut self, req: &ws::Request) -> ws::Result<(ws::Response)> {
		let origin = req.header("origin").or_else(|| req.header("Origin")).map(|x| &x[..]);
		let host = req.header("host").or_else(|| req.header("Host")).map(|x| &x[..]);
		// Styles file is allowed for error pages to display nicely.
		let is_styles_file = req.resource() == "/styles.css";

		// Check request origin and host header.
		if !self.skip_origin_validation {
			let is_valid = origin_is_allowed(&self.self_origin, origin) || (origin.is_none() && origin_is_allowed(&self.self_origin, host));
			let is_valid = is_styles_file || is_valid;

			if !is_valid {
				warn!(target: "signer", "Blocked connection to Signer API from untrusted origin.");
				return Ok(error(
						ErrorType::Forbidden,
						"URL Blocked",
						"You are not allowed to access Trusted Signer using this URL.",
						Some(&format!("Use: http://{}", self.self_origin)),
				));
			}
		}

		// Detect if it's a websocket request
		// (styles file skips origin validation, so make sure to prevent WS connections on this resource)
		if req.header("sec-websocket-key").is_some() && !is_styles_file {
			// Check authorization
			if !auth_is_valid(&self.authcodes_path, req.protocols()) {
				info!(target: "signer", "Unauthorized connection to Signer API blocked.");
				return Ok(error(ErrorType::Forbidden, "Not Authorized", "Request to this API was not authorized.", None));
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
		Ok(self.file_handler.handle(req.resource())
			.map_or_else(
				// return 404 not found
				|| error(ErrorType::NotFound, "Not found", "Requested file was not found.", None),
				// or serve the file
				|f| add_headers(ws::Response::ok_raw(f.content.to_vec()), f.content_type)
			))
	}

	fn on_message(&mut self, msg: ws::Message) -> ws::Result<()> {
		let req = try!(msg.as_text());
		if let Some(async) = self.handler.handle_request(req) {
			let out = self.out.clone();
			async.on_result(move |result| {
				let res = out.lock().send(result);
				if let Err(e) = res {
					warn!(target: "signer", "Error while sending response: {:?}", e);
				}
			});
		}
		Ok(())
	}
}

pub struct Factory {
	handler: Arc<IoHandler>,
	skip_origin_validation: bool,
	self_origin: String,
	authcodes_path: PathBuf,
	file_handler: Arc<ui::Handler>,
}

impl Factory {
	pub fn new(handler: Arc<IoHandler>, self_origin: String, authcodes_path: PathBuf, skip_origin_validation: bool) -> Self {
		Factory {
			handler: handler,
			skip_origin_validation: skip_origin_validation,
			self_origin: self_origin,
			authcodes_path: authcodes_path,
			file_handler: Arc::new(ui::Handler::default()),
		}
	}
}

impl ws::Factory for Factory {
	type Handler = Session;

	fn connection_made(&mut self, sender: ws::Sender) -> Self::Handler {
		Session {
			out: Arc::new(Mutex::new(sender)),
			handler: self.handler.clone(),
			skip_origin_validation: self.skip_origin_validation,
			self_origin: self.self_origin.clone(),
			authcodes_path: self.authcodes_path.clone(),
			file_handler: self.file_handler.clone(),
		}
	}
}

enum ErrorType {
	NotFound,
	Forbidden,
}

fn error(error: ErrorType, title: &str, message: &str, details: Option<&str>) -> ws::Response {
	let content = format!(
		include_str!("./error_tpl.html"),
		title=title,
		meta="",
		message=message,
		details=details.unwrap_or(""),
		version=version(),
	);
	let res = match error {
		ErrorType::NotFound => ws::Response::not_found(content),
		ErrorType::Forbidden => ws::Response::forbidden(content),
	};
	add_headers(res, "text/html")
}
