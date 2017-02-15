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

//! Session handlers factory.

use std::path::{PathBuf, Path};
use std::sync::Arc;
use std::str::FromStr;

use authcode_store::AuthCodes;
use jsonrpc_core::{Metadata, Middleware};
use jsonrpc_core::reactor::RpcHandler;
use rpc::informant::RpcStats;
use util::{H256, version};
use ws;

#[cfg(feature = "parity-ui")]
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
#[cfg(not(feature = "parity-ui"))]
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

const HOME_DOMAIN: &'static str = "parity.web3.site";

fn origin_is_allowed(self_origin: &str, header: Option<&[u8]>) -> bool {
	match header.map(|h| String::from_utf8_lossy(h).into_owned()) {
		Some(ref origin) if origin.starts_with("chrome-extension://") => true,
		Some(ref origin) if origin.starts_with(self_origin) => true,
		Some(ref origin) if origin.starts_with(&format!("http://{}", self_origin)) => true,
		Some(ref origin) if origin.starts_with(HOME_DOMAIN) => true,
		Some(ref origin) if origin.starts_with(&format!("http://{}", HOME_DOMAIN)) => true,
		_ => false,
	}
}

fn auth_token_hash(codes_path: &Path, protocols: ws::Result<Vec<&str>>) -> Option<H256> {
	match protocols {
		Ok(ref protocols) if protocols.len() == 1 => {
			let protocol = protocols[0];
			let mut split = protocol.split('_');
			let auth = split.next().and_then(|v| H256::from_str(v).ok());
			let time = split.next().and_then(|v| u64::from_str_radix(v, 10).ok());

			if let (Some(auth), Some(time)) = (auth, time) {
				// Check if the code is valid
				AuthCodes::from_file(codes_path)
					.ok()
					.and_then(|mut codes| {
						// remove old tokens
						codes.clear_garbage();

						let res = codes.is_valid(&auth, time);
						// make sure to save back authcodes - it might have been modified
						if codes.to_file(codes_path).is_err() {
							warn!(target: "signer", "Couldn't save authorization codes to file.");
						}

						if res {
							Some(auth)
						} else {
							None
						}
					})
			} else {
				None
			}
		},
		_ => None,
	}
}

fn add_headers(mut response: ws::Response, mime: &str) -> ws::Response {
	let content_len = format!("{}", response.len());
	{
		let mut headers = response.headers_mut();
		headers.push(("X-Frame-Options".into(), b"SAMEORIGIN".to_vec()));
		headers.push(("X-XSS-Protection".into(), b"1; mode=block".to_vec()));
		headers.push(("X-Content-Type-Options".into(), b"nosniff".to_vec()));
		headers.push(("Server".into(), b"Parity/SignerUI".to_vec()));
		headers.push(("Content-Length".into(), content_len.as_bytes().to_vec()));
		headers.push(("Content-Type".into(), mime.as_bytes().to_vec()));
		headers.push(("Connection".into(), b"close".to_vec()));
	}
	response
}

/// Metadata extractor from session data.
pub trait MetaExtractor<M: Metadata>: Send + Clone + 'static {
	/// Extract metadata for given session
	fn extract_metadata(&self, _session_id: &H256) -> M {
		Default::default()
	}
}

pub struct Session<M: Metadata, S: Middleware<M>, T> {
	session_id: H256,
	out: ws::Sender,
	skip_origin_validation: bool,
	self_origin: String,
	self_port: u16,
	authcodes_path: PathBuf,
	handler: RpcHandler<M, S>,
	file_handler: Arc<ui::Handler>,
	stats: Option<Arc<RpcStats>>,
	meta_extractor: T,
}

impl<M: Metadata, S: Middleware<M>, T> Drop for Session<M, S, T> {
	fn drop(&mut self) {
		self.stats.as_ref().map(|stats| stats.close_session());
	}
}

impl<M: Metadata, S: Middleware<M>, T: MetaExtractor<M>> ws::Handler for Session<M, S, T> {
	fn on_request(&mut self, req: &ws::Request) -> ws::Result<(ws::Response)> {
		trace!(target: "signer", "Handling request: {:?}", req);

		// TODO [ToDr] ws server is not handling proxied requests correctly:
		// Trim domain name from resource part:
		let resource = req.resource().trim_left_matches(&format!("http://{}:{}", HOME_DOMAIN, self.self_port));
		let resource = resource.trim_left_matches(&format!("http://{}", HOME_DOMAIN));

		// Styles file is allowed for error pages to display nicely.
		let is_styles_file = resource == "/styles.css";

		// Check request origin and host header.
		if !self.skip_origin_validation {
			let origin = req.header("origin").or_else(|| req.header("Origin")).map(|x| &x[..]);
			let host = req.header("host").or_else(|| req.header("Host")).map(|x| &x[..]);

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

		// PROXY requests when running behind home.parity
		if req.method() == "CONNECT" {
			let mut res = ws::Response::ok("".into());
			res.headers_mut().push(("Content-Length".into(), b"0".to_vec()));
			res.headers_mut().push(("Connection".into(), b"keep-alive".to_vec()));
			return Ok(res);
		}

		// Detect if it's a websocket request
		// (styles file skips origin validation, so make sure to prevent WS connections on this resource)
		if req.header("sec-websocket-key").is_some() && !is_styles_file {
			// Check authorization
			let auth_token_hash = auth_token_hash(&self.authcodes_path, req.protocols());
			match auth_token_hash {
				None => {
					info!(target: "signer", "Unauthorized connection to Signer API blocked.");
					return Ok(error(ErrorType::Forbidden, "Not Authorized", "Request to this API was not authorized.", None));
				},
				Some(auth) => {
					self.session_id = auth;
				},
			}

			let protocols = req.protocols().expect("Existence checked by authorization.");
			let protocol = protocols.get(0).expect("Proved by authorization.");
			return ws::Response::from_request(req).map(|mut res| {
				// To make WebSockets connection successful we need to send back the protocol header.
				res.set_protocol(protocol);
				res
			});
		}

		debug!(target: "signer", "Requesting resource: {:?}", resource);
		// Otherwise try to serve a page.
		Ok(self.file_handler.handle(resource)
			.map_or_else(
				// return 404 not found
				|| error(ErrorType::NotFound, "Not found", "Requested file was not found.", None),
				// or serve the file
				|f| add_headers(ws::Response::ok_raw(f.content.to_vec()), f.content_type)
			))
	}

	fn on_message(&mut self, msg: ws::Message) -> ws::Result<()> {
		let req = msg.as_text()?;
		let out = self.out.clone();
		// TODO [ToDr] Move to on_connect
		let metadata = self.meta_extractor.extract_metadata(&self.session_id);

		self.handler.handle_request(req, metadata, move |response| {
			if let Some(result) = response {
				let res = out.send(result);
				if let Err(e) = res {
					warn!(target: "signer", "Error while sending response: {:?}", e);
				}
			}
		});
		Ok(())
	}
}

pub struct Factory<M: Metadata, S: Middleware<M>, T> {
	handler: RpcHandler<M, S>,
	skip_origin_validation: bool,
	self_origin: String,
	self_port: u16,
	authcodes_path: PathBuf,
	meta_extractor: T,
	file_handler: Arc<ui::Handler>,
	stats: Option<Arc<RpcStats>>,
}

impl<M: Metadata, S: Middleware<M>, T> Factory<M, S, T> {
	pub fn new(
		handler: RpcHandler<M, S>,
		self_origin: String,
		self_port: u16,
		authcodes_path: PathBuf,
		skip_origin_validation: bool,
		stats: Option<Arc<RpcStats>>,
		meta_extractor: T,
	) -> Self {
		Factory {
			handler: handler,
			skip_origin_validation: skip_origin_validation,
			self_origin: self_origin,
			self_port: self_port,
			authcodes_path: authcodes_path,
			meta_extractor: meta_extractor,
			file_handler: Arc::new(ui::Handler::default()),
			stats: stats,
		}
	}
}

impl<M: Metadata, S: Middleware<M>, T: MetaExtractor<M>> ws::Factory for Factory<M, S, T> {
	type Handler = Session<M, S, T>;

	fn connection_made(&mut self, sender: ws::Sender) -> Self::Handler {
		self.stats.as_ref().map(|stats| stats.open_session());

		Session {
			session_id: 0.into(),
			out: sender,
			handler: self.handler.clone(),
			skip_origin_validation: self.skip_origin_validation,
			self_origin: self.self_origin.clone(),
			self_port: self.self_port,
			authcodes_path: self.authcodes_path.clone(),
			meta_extractor: self.meta_extractor.clone(),
			file_handler: self.file_handler.clone(),
			stats: self.stats.clone(),
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
