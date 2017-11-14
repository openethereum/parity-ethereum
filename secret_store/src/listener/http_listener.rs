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
use serde::Serialize;
use serde_json;
use url::percent_encoding::percent_decode;

use traits::KeyServer;
use serialization::{SerializableEncryptedDocumentKeyShadow, SerializableBytes, SerializablePublic};
use types::all::{Error, Public, MessageHash, NodeAddress, RequestSignature, ServerKeyId,
	EncryptedDocumentKey, EncryptedDocumentKeyShadow};

/// Key server http-requests listener. Available requests:
/// To generate server key:							POST		/shadow/{server_key_id}/{signature}/{threshold}
/// To store pregenerated encrypted document key: 	POST		/shadow/{server_key_id}/{signature}/{common_point}/{encrypted_key} 
/// To generate server && document key:				POST		/{server_key_id}/{signature}/{threshold} 
/// To get document key:							GET			/{server_key_id}/{signature}
/// To get document key shadow:						GET			/shadow/{server_key_id}/{signature} 
/// To sign message with server key:				GET			/{server_key_id}/{signature}/{message_hash}

pub struct KeyServerHttpListener {
	http_server: HttpListening,
	_handler: Arc<KeyServerSharedHttpHandler>,
}

/// Parsed http request
#[derive(Debug, Clone, PartialEq)]
enum Request {
	/// Invalid request
	Invalid,
	/// Generate server key.
	GenerateServerKey(ServerKeyId, RequestSignature, usize),
	/// Store document key.
	StoreDocumentKey(ServerKeyId, RequestSignature, Public, Public),
	/// Generate encryption key.
	GenerateDocumentKey(ServerKeyId, RequestSignature, usize),
	/// Request encryption key of given document for given requestor.
	GetDocumentKey(ServerKeyId, RequestSignature),
	/// Request shadow of encryption key of given document for given requestor.
	GetDocumentKeyShadow(ServerKeyId, RequestSignature),
	/// Sign message.
	SignMessage(ServerKeyId, RequestSignature, MessageHash),
}

/// Cloneable http handler
struct KeyServerHttpHandler {
	handler: Arc<KeyServerSharedHttpHandler>,
}

/// Shared http handler
struct KeyServerSharedHttpHandler {
	key_server: Arc<KeyServer>,
}

impl KeyServerHttpListener {
	/// Start KeyServer http listener
	pub fn start(listener_address: NodeAddress, key_server: Arc<KeyServer>) -> Result<Self, Error> {
		let shared_handler = Arc::new(KeyServerSharedHttpHandler {
			key_server: key_server,
		});

		let listener_address = format!("{}:{}", listener_address.address, listener_address.port);
		let http_server = HttpServer::http(&listener_address).expect("cannot start HttpServer");
		let http_server = http_server.handle(KeyServerHttpHandler {
				handler: shared_handler.clone(),
			}).expect("cannot start HttpServer");

		let listener = KeyServerHttpListener {
			http_server: http_server,
			_handler: shared_handler,
		};
		Ok(listener)
	}
}

impl Drop for KeyServerHttpListener {
	fn drop(&mut self) {
		// ignore error as we are dropping anyway
		self.http_server.take().map(|mut s| { let _ = s.close(); });
	}
}

impl HttpHandler for KeyServerHttpHandler {
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
				Request::GenerateServerKey(document, signature, threshold) => {
					return_server_public_key(req, res, self.handler.key_server.generate_key(&document, &signature, threshold)
						.map_err(|err| {
							warn!(target: "secretstore", "GenerateServerKey request {} has failed with: {}", req_uri, err);
							err
						}));
				},
				Request::StoreDocumentKey(document, signature, common_point, encrypted_document_key) => {
					return_empty(req, res, self.handler.key_server.store_document_key(&document, &signature, common_point, encrypted_document_key)
						.map_err(|err| {
							warn!(target: "secretstore", "StoreDocumentKey request {} has failed with: {}", req_uri, err);
							err
						}));
				},
				Request::GenerateDocumentKey(document, signature, threshold) => {
					return_document_key(req, res, self.handler.key_server.generate_document_key(&document, &signature, threshold)
						.map_err(|err| {
							warn!(target: "secretstore", "GenerateDocumentKey request {} has failed with: {}", req_uri, err);
							err
						}));
				},
				Request::GetDocumentKey(document, signature) => {
					return_document_key(req, res, self.handler.key_server.restore_document_key(&document, &signature)
						.map_err(|err| {
							warn!(target: "secretstore", "GetDocumentKey request {} has failed with: {}", req_uri, err);
							err
						}));
				},
				Request::GetDocumentKeyShadow(document, signature) => {
					return_document_key_shadow(req, res, self.handler.key_server.restore_document_key_shadow(&document, &signature)
						.map_err(|err| {
							warn!(target: "secretstore", "GetDocumentKeyShadow request {} has failed with: {}", req_uri, err);
							err
						}));
				},
				Request::SignMessage(document, signature, message_hash) => {
					return_message_signature(req, res, self.handler.key_server.sign_message(&document, &signature, message_hash)
						.map_err(|err| {
							warn!(target: "secretstore", "SignMessage request {} has failed with: {}", req_uri, err);
							err
						}));
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

fn return_empty(req: HttpRequest, res: HttpResponse, empty: Result<(), Error>) {
	return_bytes::<i32>(req, res, empty.map(|_| None))
}

fn return_server_public_key(req: HttpRequest, res: HttpResponse, server_public: Result<Public, Error>) {
	return_bytes(req, res, server_public.map(|k| Some(SerializablePublic(k))))
}

fn return_message_signature(req: HttpRequest, res: HttpResponse, signature: Result<EncryptedDocumentKey, Error>) {
	return_bytes(req, res, signature.map(|s| Some(SerializableBytes(s))))
}

fn return_document_key(req: HttpRequest, res: HttpResponse, document_key: Result<EncryptedDocumentKey, Error>) {
	return_bytes(req, res, document_key.map(|k| Some(SerializableBytes(k))))
}

fn return_document_key_shadow(req: HttpRequest, res: HttpResponse, document_key_shadow: Result<EncryptedDocumentKeyShadow, Error>) {
	return_bytes(req, res, document_key_shadow.map(|k| Some(SerializableEncryptedDocumentKeyShadow {
		decrypted_secret: k.decrypted_secret.into(),
		common_point: k.common_point.expect("always filled when requesting document_key_shadow; qed").into(),
		decrypt_shadows: k.decrypt_shadows.expect("always filled when requesting document_key_shadow; qed").into_iter().map(Into::into).collect(),
	})))
}

fn return_bytes<T: Serialize>(req: HttpRequest, mut res: HttpResponse, result: Result<Option<T>, Error>) {
	match result {
		Ok(Some(result)) => match serde_json::to_vec(&result) {
			Ok(result) => {
				res.headers_mut().set(header::ContentType::json());
				if let Err(err) = res.send(&result) {
					// nothing to do, but to log an error
					warn!(target: "secretstore", "response to request {} has failed with: {}", req.uri, err);
				}
			},
			Err(err) => {
				warn!(target: "secretstore", "response to request {} has failed with: {}", req.uri, err);
			}
		},
		Ok(None) => *res.status_mut() = HttpStatusCode::Ok,
		Err(err) => return_error(res, err),
	}
}

fn return_error(mut res: HttpResponse, err: Error) {
	match err {
		Error::BadSignature => *res.status_mut() = HttpStatusCode::BadRequest,
		Error::AccessDenied => *res.status_mut() = HttpStatusCode::Forbidden,
		Error::DocumentNotFound => *res.status_mut() = HttpStatusCode::NotFound,
		Error::Serde(_) => *res.status_mut() = HttpStatusCode::BadRequest,
		Error::Database(_) => *res.status_mut() = HttpStatusCode::InternalServerError,
		Error::Internal(_) => *res.status_mut() = HttpStatusCode::InternalServerError,
	}

	// return error text. ignore errors when returning error
	let error_text = format!("\"{}\"", err);
	if let Ok(error_text) = serde_json::to_vec(&error_text) {
		res.headers_mut().set(header::ContentType::json());
		let _ = res.send(&error_text);
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

	let (is_shadow_request, args_offset) = if &path[0] == "shadow" { (true, 1) } else { (false, 0) };
	let args_count = path.len() - args_offset;
	if args_count < 2 || path[args_offset].is_empty() || path[args_offset + 1].is_empty() {
		return Request::Invalid;
	}

	let document = match path[args_offset].parse() {
		Ok(document) => document,
		_ => return Request::Invalid,
	};
	let signature = match path[args_offset + 1].parse() {
		Ok(signature) => signature,
		_ => return Request::Invalid,
	};

	let threshold = path.get(args_offset + 2).map(|v| v.parse());
	let message_hash = path.get(args_offset + 2).map(|v| v.parse());
	let common_point = path.get(args_offset + 2).map(|v| v.parse());
	let encrypted_key = path.get(args_offset + 3).map(|v| v.parse());
	match (is_shadow_request, args_count, method, threshold, message_hash, common_point, encrypted_key) {
		(true, 3, &HttpMethod::Post, Some(Ok(threshold)), _, _, _) =>
			Request::GenerateServerKey(document, signature, threshold),
		(true, 4, &HttpMethod::Post, _, _, Some(Ok(common_point)), Some(Ok(encrypted_key))) =>
			Request::StoreDocumentKey(document, signature, common_point, encrypted_key),
		(false, 3, &HttpMethod::Post, Some(Ok(threshold)), _, _, _) =>
			Request::GenerateDocumentKey(document, signature, threshold),
		(false, 2, &HttpMethod::Get, _, _, _, _) =>
			Request::GetDocumentKey(document, signature),
		(true, 2, &HttpMethod::Get, _, _, _, _) =>
			Request::GetDocumentKeyShadow(document, signature),
		(false, 3, &HttpMethod::Get, _, Some(Ok(message_hash)), _, _) =>
			Request::SignMessage(document, signature, message_hash),
		_ => Request::Invalid,
	}
}

#[cfg(test)]
mod tests {
	use std::sync::Arc;
	use hyper::method::Method as HttpMethod;
	use key_server::tests::DummyKeyServer;
	use types::all::NodeAddress;
	use super::{parse_request, Request, KeyServerHttpListener};

	#[test]
	fn http_listener_successfully_drops() {
		let key_server = Arc::new(DummyKeyServer);
		let address = NodeAddress { address: "127.0.0.1".into(), port: 9000 };
		let listener = KeyServerHttpListener::start(Some(address), key_server).unwrap();
		drop(listener);
	}
 
	#[test]
	fn parse_request_successful() {
		// POST		/shadow/{server_key_id}/{signature}/{threshold}						=> generate server key
		assert_eq!(parse_request(&HttpMethod::Post, "/shadow/0000000000000000000000000000000000000000000000000000000000000001/a199fb39e11eefb61c78a4074a53c0d4424600a3e74aad4fb9d93a26c30d067e1d4d29936de0c73f19827394a1dd049480a0d581aee7ae7546968da7d3d1c2fd01/2"),
			Request::GenerateServerKey("0000000000000000000000000000000000000000000000000000000000000001".into(),
				"a199fb39e11eefb61c78a4074a53c0d4424600a3e74aad4fb9d93a26c30d067e1d4d29936de0c73f19827394a1dd049480a0d581aee7ae7546968da7d3d1c2fd01".parse().unwrap(),
				2));
		// POST		/shadow/{server_key_id}/{signature}/{common_point}/{encrypted_key}	=> store encrypted document key
		assert_eq!(parse_request(&HttpMethod::Post, "/shadow/0000000000000000000000000000000000000000000000000000000000000001/a199fb39e11eefb61c78a4074a53c0d4424600a3e74aad4fb9d93a26c30d067e1d4d29936de0c73f19827394a1dd049480a0d581aee7ae7546968da7d3d1c2fd01/b486d3840218837b035c66196ecb15e6b067ca20101e11bd5e626288ab6806ecc70b8307012626bd512bad1559112d11d21025cef48cc7a1d2f3976da08f36c8/1395568277679f7f583ab7c0992da35f26cde57149ee70e524e49bdae62db3e18eb96122501e7cbb798b784395d7bb5a499edead0706638ad056d886e56cf8fb"),
			Request::StoreDocumentKey("0000000000000000000000000000000000000000000000000000000000000001".into(),
				"a199fb39e11eefb61c78a4074a53c0d4424600a3e74aad4fb9d93a26c30d067e1d4d29936de0c73f19827394a1dd049480a0d581aee7ae7546968da7d3d1c2fd01".parse().unwrap(),
				"b486d3840218837b035c66196ecb15e6b067ca20101e11bd5e626288ab6806ecc70b8307012626bd512bad1559112d11d21025cef48cc7a1d2f3976da08f36c8".parse().unwrap(),
				"1395568277679f7f583ab7c0992da35f26cde57149ee70e524e49bdae62db3e18eb96122501e7cbb798b784395d7bb5a499edead0706638ad056d886e56cf8fb".parse().unwrap()));
		// POST		/{server_key_id}/{signature}/{threshold}							=> generate server && document key
		assert_eq!(parse_request(&HttpMethod::Post, "/0000000000000000000000000000000000000000000000000000000000000001/a199fb39e11eefb61c78a4074a53c0d4424600a3e74aad4fb9d93a26c30d067e1d4d29936de0c73f19827394a1dd049480a0d581aee7ae7546968da7d3d1c2fd01/2"),
			Request::GenerateDocumentKey("0000000000000000000000000000000000000000000000000000000000000001".into(),
				"a199fb39e11eefb61c78a4074a53c0d4424600a3e74aad4fb9d93a26c30d067e1d4d29936de0c73f19827394a1dd049480a0d581aee7ae7546968da7d3d1c2fd01".parse().unwrap(),
				2));
		// GET		/{server_key_id}/{signature}										=> get document key
		assert_eq!(parse_request(&HttpMethod::Get, "/0000000000000000000000000000000000000000000000000000000000000001/a199fb39e11eefb61c78a4074a53c0d4424600a3e74aad4fb9d93a26c30d067e1d4d29936de0c73f19827394a1dd049480a0d581aee7ae7546968da7d3d1c2fd01"),
			Request::GetDocumentKey("0000000000000000000000000000000000000000000000000000000000000001".into(),
				"a199fb39e11eefb61c78a4074a53c0d4424600a3e74aad4fb9d93a26c30d067e1d4d29936de0c73f19827394a1dd049480a0d581aee7ae7546968da7d3d1c2fd01".parse().unwrap()));
		assert_eq!(parse_request(&HttpMethod::Get, "/%30000000000000000000000000000000000000000000000000000000000000001/a199fb39e11eefb61c78a4074a53c0d4424600a3e74aad4fb9d93a26c30d067e1d4d29936de0c73f19827394a1dd049480a0d581aee7ae7546968da7d3d1c2fd01"),
			Request::GetDocumentKey("0000000000000000000000000000000000000000000000000000000000000001".into(),
				"a199fb39e11eefb61c78a4074a53c0d4424600a3e74aad4fb9d93a26c30d067e1d4d29936de0c73f19827394a1dd049480a0d581aee7ae7546968da7d3d1c2fd01".parse().unwrap()));
		// GET		/shadow/{server_key_id}/{signature}									=> get document key shadow
		assert_eq!(parse_request(&HttpMethod::Get, "/shadow/0000000000000000000000000000000000000000000000000000000000000001/a199fb39e11eefb61c78a4074a53c0d4424600a3e74aad4fb9d93a26c30d067e1d4d29936de0c73f19827394a1dd049480a0d581aee7ae7546968da7d3d1c2fd01"),
			Request::GetDocumentKeyShadow("0000000000000000000000000000000000000000000000000000000000000001".into(),
				"a199fb39e11eefb61c78a4074a53c0d4424600a3e74aad4fb9d93a26c30d067e1d4d29936de0c73f19827394a1dd049480a0d581aee7ae7546968da7d3d1c2fd01".parse().unwrap()));
		// GET		/{server_key_id}/{signature}/{message_hash}							=> sign message with server key
		assert_eq!(parse_request(&HttpMethod::Get, "/0000000000000000000000000000000000000000000000000000000000000001/a199fb39e11eefb61c78a4074a53c0d4424600a3e74aad4fb9d93a26c30d067e1d4d29936de0c73f19827394a1dd049480a0d581aee7ae7546968da7d3d1c2fd01/281b6bf43cb86d0dc7b98e1b7def4a80f3ce16d28d2308f934f116767306f06c"),
			Request::SignMessage("0000000000000000000000000000000000000000000000000000000000000001".into(),
				"a199fb39e11eefb61c78a4074a53c0d4424600a3e74aad4fb9d93a26c30d067e1d4d29936de0c73f19827394a1dd049480a0d581aee7ae7546968da7d3d1c2fd01".parse().unwrap(),
				"281b6bf43cb86d0dc7b98e1b7def4a80f3ce16d28d2308f934f116767306f06c".parse().unwrap()));
	}

	#[test]
	fn parse_request_failed() {
		assert_eq!(parse_request(&HttpMethod::Get, ""), Request::Invalid);
		assert_eq!(parse_request(&HttpMethod::Get, "/shadow"), Request::Invalid);
		assert_eq!(parse_request(&HttpMethod::Get, "///2"), Request::Invalid);
		assert_eq!(parse_request(&HttpMethod::Get, "/shadow///2"), Request::Invalid);
		assert_eq!(parse_request(&HttpMethod::Get, "/0000000000000000000000000000000000000000000000000000000000000001"), Request::Invalid);
		assert_eq!(parse_request(&HttpMethod::Get, "/0000000000000000000000000000000000000000000000000000000000000001/"), Request::Invalid);
		assert_eq!(parse_request(&HttpMethod::Get, "/a/b"), Request::Invalid);
		assert_eq!(parse_request(&HttpMethod::Get, "/0000000000000000000000000000000000000000000000000000000000000001/a199fb39e11eefb61c78a4074a53c0d4424600a3e74aad4fb9d93a26c30d067e1d4d29936de0c73f19827394a1dd049480a0d581aee7ae7546968da7d3d1c2fd01/0000000000000000000000000000000000000000000000000000000000000002/0000000000000000000000000000000000000000000000000000000000000002"), Request::Invalid);
	}
}
