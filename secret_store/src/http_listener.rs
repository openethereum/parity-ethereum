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

use std::sync::Arc;
use hyper::header;
use hyper::uri::RequestUri;
use hyper::method::Method as HttpMethod;
use hyper::status::StatusCode as HttpStatusCode;
use hyper::server::{Server as HttpServer, Request as HttpRequest, Response as HttpResponse, Handler as HttpHandler,
	Listening as HttpListening};
use serde_json;
use url::percent_encoding::percent_decode;

use util::ToPretty;
use traits::KeyServer;
use serialization::SerializableDocumentEncryptedKeyShadow;
use types::all::{Error, ServiceConfiguration, RequestSignature, DocumentAddress, DocumentEncryptedKey, DocumentEncryptedKeyShadow};

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
	/// Generate encryption key.
	GenerateDocumentKey(DocumentAddress, RequestSignature, usize),
	/// Request encryption key of given document for given requestor.
	GetDocumentKey(DocumentAddress, RequestSignature),
	/// Request shadow of encryption key of given document for given requestor.
	GetDocumentKeyShadow(DocumentAddress, RequestSignature),
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

		let listener_addr: &str = &format!("{}:{}", config.listener_address.address, config.listener_address.port);
		let http_server = HttpServer::http(&listener_addr).expect("cannot start HttpServer");
		let http_server = http_server.handle(handler).expect("cannot start HttpServer");
		let listener = KeyServerHttpListener {
			_http_server: http_server,
			handler: shared_handler,
		};
		Ok(listener)
	}
}

impl<T> KeyServer for KeyServerHttpListener<T> where T: KeyServer + 'static {
	fn generate_document_key(&self, signature: &RequestSignature, document: &DocumentAddress, threshold: usize) -> Result<DocumentEncryptedKey, Error> {
		self.handler.key_server.generate_document_key(signature, document, threshold)
	}

	fn document_key(&self, signature: &RequestSignature, document: &DocumentAddress) -> Result<DocumentEncryptedKey, Error> {
		self.handler.key_server.document_key(signature, document)
	}

	fn document_key_shadow(&self, signature: &RequestSignature, document: &DocumentAddress) -> Result<DocumentEncryptedKeyShadow, Error> {
		self.handler.key_server.document_key_shadow(signature, document)
	}
}

impl<T> HttpHandler for KeyServerHttpHandler<T> where T: KeyServer + 'static {
	fn handle(&self, req: HttpRequest, mut res: HttpResponse) {
		if req.headers.has::<header::Origin>() {
			warn!(target: "secretstore", "Ignoring {}-request {} with Origin header", req.method, req.uri);
			*res.status_mut() = HttpStatusCode::NotFound;
			return;
		}

		let req_method = req.method.clone();
		let req_uri = req.uri.clone();
		match &req_uri {
			&RequestUri::AbsolutePath(ref path) => match parse_request(&req_method, &path) {
				Request::GenerateDocumentKey(document, signature, threshold) => {
					return_document_key(req, res, self.handler.key_server.generate_document_key(&signature, &document, threshold)
						.map_err(|err| {
							warn!(target: "secretstore", "GenerateDocumentKey request {} has failed with: {}", req_uri, err);
							err
						}));
				},
				Request::GetDocumentKey(document, signature) => {
					return_document_key(req, res, self.handler.key_server.document_key(&signature, &document)
						.map_err(|err| {
							warn!(target: "secretstore", "GetDocumentKey request {} has failed with: {}", req_uri, err);
							err
						}));
				},
				Request::GetDocumentKeyShadow(document, signature) => {
					match self.handler.key_server.document_key_shadow(&signature, &document)
						.map_err(|err| {
							warn!(target: "secretstore", "GetDocumentKeyShadow request {} has failed with: {}", req_uri, err);
							err
						}) {
						Ok(document_key_shadow) => {
							let document_key_shadow = SerializableDocumentEncryptedKeyShadow {
								decrypted_secret: document_key_shadow.decrypted_secret.into(),
								common_point: document_key_shadow.common_point.expect("always filled when requesting document_key_shadow; qed").into(),
								decrypt_shadows: document_key_shadow.decrypt_shadows.expect("always filled when requesting document_key_shadow; qed").into_iter().map(Into::into).collect(),
							};
							match serde_json::to_vec(&document_key_shadow) {
								Ok(document_key) => {
									res.headers_mut().set(header::ContentType::json());
									if let Err(err) = res.send(&document_key) {
										// nothing to do, but to log an error
										warn!(target: "secretstore", "response to request {} has failed with: {}", req.uri, err);
									}
								},
								Err(err) => {
									warn!(target: "secretstore", "response to request {} has failed with: {}", req.uri, err);
								}
							}
						},
						Err(err) => return_error(res, err),
					}
				},
				Request::Invalid => {
					warn!(target: "secretstore", "Ignoring invalid {}-request {}", req_method, req_uri);
					*res.status_mut() = HttpStatusCode::BadRequest;
				},
			},
			_ => {
				warn!(target: "secretstore", "Ignoring invalid {}-request {}", req_method, req_uri);
				*res.status_mut() = HttpStatusCode::NotFound;
			},
		};
	}
}

fn return_document_key(req: HttpRequest, mut res: HttpResponse, document_key: Result<DocumentEncryptedKey, Error>) {
	match document_key {
		Ok(document_key) => {
			let document_key = document_key.to_hex().into_bytes();
			res.headers_mut().set(header::ContentType::plaintext());
			if let Err(err) = res.send(&document_key) {
				// nothing to do, but to log an error
				warn!(target: "secretstore", "response to request {} has failed with: {}", req.uri, err);
			}
		},
		Err(err) => return_error(res, err),
	}
}

fn return_error(mut res: HttpResponse, err: Error) {
	match err {
		Error::BadSignature => *res.status_mut() = HttpStatusCode::BadRequest,
		Error::AccessDenied => *res.status_mut() = HttpStatusCode::Forbidden,
		Error::DocumentNotFound => *res.status_mut() = HttpStatusCode::NotFound,
		Error::Database(_) => *res.status_mut() = HttpStatusCode::InternalServerError,
		Error::Internal(_) => *res.status_mut() = HttpStatusCode::InternalServerError,
	}
}

fn parse_request(method: &HttpMethod, uri_path: &str) -> Request {
	let uri_path = match percent_decode(uri_path.as_bytes()).decode_utf8() {
		Ok(path) => path,
		Err(_) => return Request::Invalid,
	};

	let path: Vec<String> = uri_path.trim_left_matches('/').split('/').map(Into::into).collect();
	if path.len() == 0 {
		return Request::Invalid;
	}
	let (args_prefix, args_offset) = if &path[0] == "shadow" {
		("shadow", 1)
	} else {
		("", 0)
	};

	if path.len() < 2 + args_offset || path[args_offset].is_empty() || path[args_offset + 1].is_empty() {
		return Request::Invalid;
	}

	let args_len = path.len();
	let document = path[args_offset].parse();
	let signature = path[args_offset + 1].parse();
	let threshold = (if args_len > args_offset + 2 { &path[args_offset + 2] } else { "" }).parse();
	match (args_prefix, args_len, method, document, signature, threshold) {
		("",		3, &HttpMethod::Post, Ok(document), Ok(signature), Ok(threshold)) => Request::GenerateDocumentKey(document, signature, threshold),
		("",		2, &HttpMethod::Get, Ok(document), Ok(signature), _) => Request::GetDocumentKey(document, signature),
		("shadow",	3, &HttpMethod::Get, Ok(document), Ok(signature), _) => Request::GetDocumentKeyShadow(document, signature),
		_ => Request::Invalid,
	}
}

#[cfg(test)]
mod tests {
	use hyper::method::Method as HttpMethod;
	use super::{parse_request, Request};

	#[test]
	fn parse_request_successful() {
		assert_eq!(parse_request(&HttpMethod::Get, "/0000000000000000000000000000000000000000000000000000000000000001/a199fb39e11eefb61c78a4074a53c0d4424600a3e74aad4fb9d93a26c30d067e1d4d29936de0c73f19827394a1dd049480a0d581aee7ae7546968da7d3d1c2fd01"),
			Request::GetDocumentKey("0000000000000000000000000000000000000000000000000000000000000001".into(),
				"a199fb39e11eefb61c78a4074a53c0d4424600a3e74aad4fb9d93a26c30d067e1d4d29936de0c73f19827394a1dd049480a0d581aee7ae7546968da7d3d1c2fd01".parse().unwrap()));
		assert_eq!(parse_request(&HttpMethod::Get, "/%30000000000000000000000000000000000000000000000000000000000000001/a199fb39e11eefb61c78a4074a53c0d4424600a3e74aad4fb9d93a26c30d067e1d4d29936de0c73f19827394a1dd049480a0d581aee7ae7546968da7d3d1c2fd01"),
			Request::GetDocumentKey("0000000000000000000000000000000000000000000000000000000000000001".into(),
				"a199fb39e11eefb61c78a4074a53c0d4424600a3e74aad4fb9d93a26c30d067e1d4d29936de0c73f19827394a1dd049480a0d581aee7ae7546968da7d3d1c2fd01".parse().unwrap()));
	}

	#[test]
	fn parse_request_failed() {
		assert_eq!(parse_request(&HttpMethod::Get, "/0000000000000000000000000000000000000000000000000000000000000001"), Request::Invalid);
		assert_eq!(parse_request(&HttpMethod::Get, "/0000000000000000000000000000000000000000000000000000000000000001/"), Request::Invalid);
		assert_eq!(parse_request(&HttpMethod::Get, "/a/b"), Request::Invalid);
		assert_eq!(parse_request(&HttpMethod::Get, "/0000000000000000000000000000000000000000000000000000000000000001/a199fb39e11eefb61c78a4074a53c0d4424600a3e74aad4fb9d93a26c30d067e1d4d29936de0c73f19827394a1dd049480a0d581aee7ae7546968da7d3d1c2fd01/0000000000000000000000000000000000000000000000000000000000000002"), Request::Invalid);
	}
}
