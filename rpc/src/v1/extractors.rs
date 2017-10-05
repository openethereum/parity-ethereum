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

//! Parity-specific metadata extractors.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use authcodes;
use http_common::HttpMetaExtractor;
use ipc;
use jsonrpc_core as core;
use jsonrpc_pubsub::Session;
use ws;
use bigint::hash::H256;

use v1::{Metadata, Origin};
use v1::informant::RpcStats;

/// Common HTTP & IPC metadata extractor.
pub struct RpcExtractor;

impl HttpMetaExtractor for RpcExtractor {
	type Metadata = Metadata;

	fn read_metadata(&self, origin: Option<String>, user_agent: Option<String>, dapps_origin: Option<String>) -> Metadata {
		let mut metadata = Metadata::default();

		metadata.origin = match (origin.as_ref().map(|s| s.as_str()), user_agent, dapps_origin) {
			(Some("null"), _, Some(dapp)) => Origin::Dapps(dapp.into()),
			(Some(dapp), _, _) => Origin::Dapps(dapp.to_owned().into()),
			(None, Some(service), _) => Origin::Rpc(service.into()),
			(None, _, _) => Origin::Rpc("unknown".into()),
		};

		metadata
	}
}

impl ipc::MetaExtractor<Metadata> for RpcExtractor {
	fn extract(&self, req: &ipc::RequestContext) -> Metadata {
		let mut metadata = Metadata::default();
		metadata.origin = Origin::Ipc(req.session_id.into());
		metadata.session = Some(Arc::new(Session::new(req.sender.clone())));
		metadata
	}
}

/// WebSockets server metadata extractor and request middleware.
pub struct WsExtractor {
	authcodes_path: Option<PathBuf>,
}

impl WsExtractor {
	/// Creates new `WsExtractor` with given authcodes path.
	pub fn new(path: Option<&Path>) -> Self {
		WsExtractor {
			authcodes_path: path.map(|p| p.to_owned()),
		}
	}
}

impl ws::MetaExtractor<Metadata> for WsExtractor {
	fn extract(&self, req: &ws::RequestContext) -> Metadata {
		let mut metadata = Metadata::default();
		let id = req.session_id as u64;

		let dapp = req.origin.as_ref().map(|origin| (&**origin).into()).unwrap_or_default();
		metadata.origin = match self.authcodes_path {
			Some(ref path) => {
				let authorization = req.protocols.get(0).and_then(|p| auth_token_hash(&path, p, true));
				match authorization {
					Some(id) => Origin::Signer { session: id.into(), dapp: dapp },
					None => Origin::Ws { session: id.into(), dapp: dapp },
				}
			},
			None => Origin::Ws { session: id.into(), dapp: dapp },
		};
		metadata.session = Some(Arc::new(Session::new(req.sender())));
		metadata
	}
}

impl ws::RequestMiddleware for WsExtractor {
	fn process(&self, req: &ws::ws::Request) -> ws::MiddlewareAction {
		use self::ws::ws::Response;

		// Reply with 200 Ok to HEAD requests.
		if req.method() == "HEAD" {
			let mut response = Response::new(200, "Ok");
			add_security_headers(&mut response);
			return Some(response).into();
		}

		// Display WS info.
		if req.header("sec-websocket-key").is_none() {
			let mut response = Response::new(200, "Ok");
			response.set_body("WebSocket interface is active. Open WS connection to access RPC.");
			add_security_headers(&mut response);
			return Some(response).into();
		}

		// If protocol is provided it needs to be valid.
		let protocols = req.protocols().ok().unwrap_or_else(Vec::new);
		if let Some(ref path) = self.authcodes_path {
			if protocols.len() == 1 {
				let authorization = auth_token_hash(&path, protocols[0], false);
				if authorization.is_none() {
					warn!(
						"Blocked connection from {} using invalid token.",
						req.header("origin").and_then(|e| ::std::str::from_utf8(e).ok()).unwrap_or("Unknown Origin")
					);
					let mut response = Response::new(403, "Forbidden");
					add_security_headers(&mut response);
					return Some(response).into();
				}
			}
		}

		// Otherwise just proceed.
		ws::MiddlewareAction::Proceed
	}
}

fn add_security_headers(res: &mut ws::ws::Response) {
	let mut headers = res.headers_mut();
	headers.push(("X-Frame-Options".into(), b"SAMEORIGIN".to_vec()));
	headers.push(("X-XSS-Protection".into(), b"1; mode=block".to_vec()));
	headers.push(("X-Content-Type-Options".into(), b"nosniff".to_vec()));
	headers.push(("Content-Security-Policy".into(),
		b"default-src 'self';form-action 'none';block-all-mixed-content;sandbox allow-scripts;".to_vec()
	));
}

fn auth_token_hash(codes_path: &Path, protocol: &str, save_file: bool) -> Option<H256> {
	let mut split = protocol.split('_');
	let auth = split.next().and_then(|v| v.parse().ok());
	let time = split.next().and_then(|v| u64::from_str_radix(v, 10).ok());

	if let (Some(auth), Some(time)) = (auth, time) {
		// Check if the code is valid
		return authcodes::AuthCodes::from_file(codes_path)
			.ok()
			.and_then(|mut codes| {
				// remove old tokens
				codes.clear_garbage();

				let res = codes.is_valid(&auth, time);

				if save_file {
					// make sure to save back authcodes - it might have been modified
					if codes.to_file(codes_path).is_err() {
						warn!(target: "signer", "Couldn't save authorization codes to file.");
					}
				}

				if res {
					Some(auth)
				} else {
					None
				}
			})
	}

	None
}

/// WebSockets RPC usage statistics.
pub struct WsStats {
	stats: Arc<RpcStats>,
}

impl WsStats {
	/// Creates new WS usage tracker.
	pub fn new(stats: Arc<RpcStats>) -> Self {
		WsStats {
			stats: stats,
		}
	}
}

impl ws::SessionStats for WsStats {
	fn open_session(&self, _id: ws::SessionId) {
		self.stats.open_session()
	}

	fn close_session(&self, _id: ws::SessionId) {
		self.stats.close_session()
	}
}

/// WebSockets middleware dispatching requests to different handles dependning on metadata.
pub struct WsDispatcher<M: core::Middleware<Metadata>> {
	full_handler: core::MetaIoHandler<Metadata, M>,
}

impl<M: core::Middleware<Metadata>> WsDispatcher<M> {
	/// Create new `WsDispatcher` with given full handler.
	pub fn new(full_handler: core::MetaIoHandler<Metadata, M>) -> Self {
		WsDispatcher {
			full_handler: full_handler,
		}
	}
}

impl<M: core::Middleware<Metadata>> core::Middleware<Metadata> for WsDispatcher<M> {
	type Future = core::futures::future::Either<
		M::Future,
		core::FutureResponse,
	>;

	fn on_request<F, X>(&self, request: core::Request, meta: Metadata, process: F) -> Self::Future where
		F: FnOnce(core::Request, Metadata) -> X,
		X: core::futures::Future<Item=Option<core::Response>, Error=()> + Send + 'static,
	{
		use self::core::futures::future::Either::{A, B};

		let use_full = match &meta.origin {
			&Origin::Signer { .. } => true,
			_ => false,
		};

		if use_full {
			A(self.full_handler.handle_rpc_request(request, meta))
		} else {
			B(Box::new(process(request, meta)))
		}
	}
}

#[cfg(test)]
mod tests {
	use super::RpcExtractor;
	use {HttpMetaExtractor, Origin};

	#[test]
	fn should_extract_rpc_origin() {
		// given
		let extractor = RpcExtractor;

		// when
		let meta1 = extractor.read_metadata(None, None, None);
		let meta2 = extractor.read_metadata(None, Some("http://parity.io".to_owned()), None);
		let meta3 = extractor.read_metadata(None, Some("http://parity.io".to_owned()), Some("ignored".into()));

		// then
		assert_eq!(meta1.origin, Origin::Rpc("unknown".into()));
		assert_eq!(meta2.origin, Origin::Rpc("http://parity.io".into()));
		assert_eq!(meta3.origin, Origin::Rpc("http://parity.io".into()));
	}

	#[test]
	fn should_dapps_origin() {
		// given
		let extractor = RpcExtractor;
		let dapp = "https://wallet.ethereum.org".to_owned();

		// when
		let meta = extractor.read_metadata(Some("null".into()), None, Some(dapp.clone()));

		// then
		assert_eq!(meta.origin, Origin::Dapps(dapp.into()));
	}
}
