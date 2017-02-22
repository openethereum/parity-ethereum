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

use std::str::FromStr;
use std::sync::Arc;
use hyper::header;
use hyper::uri::RequestUri;
use hyper::method::Method as HttpMethod;
use hyper::status::StatusCode as HttpStatusCode;
use hyper::server::{Server as HttpServer, Request as HttpRequest, Response as HttpResponse, Handler as HttpHandler,
	Listening as HttpListening};
use url::percent_encoding::percent_decode;

use util::ToPretty;
use traits::KeyServer;
use types::all::{Error, ServiceConfiguration, RequestSignature, DocumentAddress, DocumentEncryptedKey};

/// Key server http-requests listener
pub struct KeyServerHttpListener<T: KeyServer + 'static> {
	_http_server: HttpListening,
	handler: Arc<KeyServerSharedHttpHandler<T>>,
}

/// Parsed http request
#[derive(Debug, Clone, PartialEq)]
enum Request {
	/// Invalid request
	Invalid,
	/// Request encryption key of given document for given requestor
	GetDocumentKey(DocumentAddress, RequestSignature),
}

/// Cloneable http handler
struct KeyServerHttpHandler<T: KeyServer + 'static> {
	handler: Arc<KeyServerSharedHttpHandler<T>>,
}

/// Shared http handler
struct KeyServerSharedHttpHandler<T: KeyServer + 'static> {
	key_server: T,
}

impl<T> KeyServerHttpListener<T> where T: KeyServer + 'static {
	/// Start KeyServer http listener
	pub fn start(config: ServiceConfiguration, key_server: T) -> Result<Self, Error> {
		let shared_handler = Arc::new(KeyServerSharedHttpHandler {
			key_server: key_server,
		});
		let handler = KeyServerHttpHandler {
			handler: shared_handler.clone(),
		};

		let listener_addr: &str = &format!("{}:{}", config.listener_addr, config.listener_port);
		let http_server = HttpServer::http(&listener_addr).unwrap();
		let http_server = http_server.handle(handler).unwrap();
		let listener = KeyServerHttpListener {
			_http_server: http_server,
			handler: shared_handler,
		};
		Ok(listener)
	}
}

impl<T> KeyServer for KeyServerHttpListener<T> where T: KeyServer + 'static {
	fn document_key(&self, signature: &RequestSignature, document: &DocumentAddress) -> Result<DocumentEncryptedKey, Error> {
		self.handler.key_server.document_key(signature, document)
	}
}

impl<T> HttpHandler for KeyServerHttpHandler<T> where T: KeyServer + 'static {
	fn handle(&self, req: HttpRequest, mut res: HttpResponse) {
		if req.method != HttpMethod::Get {
			warn!(target: "secretstore", "Ignoring {}-request {}", req.method, req.uri);
			*res.status_mut() = HttpStatusCode::NotFound;
			return;
		}

		if req.headers.has::<header::Origin>() {
			warn!(target: "secretstore", "Ignoring {}-request {} with Origin header", req.method, req.uri);
			*res.status_mut() = HttpStatusCode::NotFound;
			return;
		}

		match req.uri {
			RequestUri::AbsolutePath(ref path) => match parse_request(&path) {
				Request::GetDocumentKey(document, signature) => {
					let document_key = self.handler.key_server.document_key(&signature, &document)
						.map_err(|err| {
							warn!(target: "secretstore", "GetDocumentKey request {} has failed with: {}", req.uri, err);
							err
						});
					match document_key {
						Ok(document_key) => {
							let document_key = document_key.to_hex().into_bytes();
							res.headers_mut().set(header::ContentType::plaintext());
							if let Err(err) = res.send(&document_key) {
								// nothing to do, but log error
								warn!(target: "secretstore", "GetDocumentKey request {} response has failed with: {}", req.uri, err);
							}
						},
						Err(Error::BadSignature) => *res.status_mut() = HttpStatusCode::BadRequest,
						Err(Error::AccessDenied) => *res.status_mut() = HttpStatusCode::Forbidden,
						Err(Error::DocumentNotFound) => *res.status_mut() = HttpStatusCode::NotFound,
						Err(Error::Database(_)) => *res.status_mut() = HttpStatusCode::InternalServerError,
						Err(Error::Internal(_)) => *res.status_mut() = HttpStatusCode::InternalServerError,
					}
				},
				Request::Invalid => {
					warn!(target: "secretstore", "Ignoring invalid {}-request {}", req.method, req.uri);
					*res.status_mut() = HttpStatusCode::BadRequest;
				},
			},
			_ => {
				warn!(target: "secretstore", "Ignoring invalid {}-request {}", req.method, req.uri);
				*res.status_mut() = HttpStatusCode::NotFound;
			},
		};
	}
}

fn parse_request(uri_path: &str) -> Request {
	let uri_path = match percent_decode(uri_path.as_bytes()).decode_utf8() {
		Ok(path) => path,
		Err(_) => return Request::Invalid,
	};

	let path: Vec<String> = uri_path.trim_left_matches('/').split('/').map(Into::into).collect();
	if path.len() != 2 || path[0].is_empty() || path[1].is_empty() {
		return Request::Invalid;
	}

	let document = DocumentAddress::from_str(&path[0]);
	let signature = RequestSignature::from_str(&path[1]);
	match (document, signature) {
		(Ok(document), Ok(signature)) => Request::GetDocumentKey(document, signature),
		_ => Request::Invalid,
	}
}

#[cfg(test)]
mod tests {
	use std::str::FromStr;
	use super::super::RequestSignature;
	use super::{parse_request, Request};

	#[test]
	fn parse_request_successful() {
		assert_eq!(parse_request("/0000000000000000000000000000000000000000000000000000000000000001/a199fb39e11eefb61c78a4074a53c0d4424600a3e74aad4fb9d93a26c30d067e1d4d29936de0c73f19827394a1dd049480a0d581aee7ae7546968da7d3d1c2fd01"),
			Request::GetDocumentKey("0000000000000000000000000000000000000000000000000000000000000001".into(),
				RequestSignature::from_str("a199fb39e11eefb61c78a4074a53c0d4424600a3e74aad4fb9d93a26c30d067e1d4d29936de0c73f19827394a1dd049480a0d581aee7ae7546968da7d3d1c2fd01").unwrap()));
		assert_eq!(parse_request("/%30000000000000000000000000000000000000000000000000000000000000001/a199fb39e11eefb61c78a4074a53c0d4424600a3e74aad4fb9d93a26c30d067e1d4d29936de0c73f19827394a1dd049480a0d581aee7ae7546968da7d3d1c2fd01"),
			Request::GetDocumentKey("0000000000000000000000000000000000000000000000000000000000000001".into(),
				RequestSignature::from_str("a199fb39e11eefb61c78a4074a53c0d4424600a3e74aad4fb9d93a26c30d067e1d4d29936de0c73f19827394a1dd049480a0d581aee7ae7546968da7d3d1c2fd01").unwrap()));
	}

	#[test]
	fn parse_request_failed() {
		assert_eq!(parse_request("/0000000000000000000000000000000000000000000000000000000000000001"), Request::Invalid);
		assert_eq!(parse_request("/0000000000000000000000000000000000000000000000000000000000000001/"), Request::Invalid);
		assert_eq!(parse_request("/a/b"), Request::Invalid);
		assert_eq!(parse_request("/0000000000000000000000000000000000000000000000000000000000000001/a199fb39e11eefb61c78a4074a53c0d4424600a3e74aad4fb9d93a26c30d067e1d4d29936de0c73f19827394a1dd049480a0d581aee7ae7546968da7d3d1c2fd01/0000000000000000000000000000000000000000000000000000000000000002"), Request::Invalid);
	}
}
