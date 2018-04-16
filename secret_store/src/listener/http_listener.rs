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

use std::collections::BTreeSet;
use std::sync::{Arc, Weak};
use hyper::{self, header, Chunk, Uri, Request as HttpRequest, Response as HttpResponse, Method as HttpMethod, StatusCode as HttpStatusCode};
use hyper::server::Http;
use serde::Serialize;
use serde_json;
use tokio;
use tokio::net::TcpListener;
use tokio::runtime::Runtime;
use tokio_service::Service;
use futures::{future, Future, Stream};
use url::percent_encoding::percent_decode;

use traits::KeyServer;
use serialization::{SerializableEncryptedDocumentKeyShadow, SerializableBytes, SerializablePublic};
use types::all::{Error, Public, MessageHash, NodeAddress, RequestSignature, ServerKeyId,
	EncryptedDocumentKey, EncryptedDocumentKeyShadow, NodeId};

/// Key server http-requests listener. Available requests:
/// To generate server key:							POST		/shadow/{server_key_id}/{signature}/{threshold}
/// To store pregenerated encrypted document key: 	POST		/shadow/{server_key_id}/{signature}/{common_point}/{encrypted_key}
/// To generate server && document key:				POST		/{server_key_id}/{signature}/{threshold}
/// To get document key:							GET			/{server_key_id}/{signature}
/// To get document key shadow:						GET			/shadow/{server_key_id}/{signature}
/// To generate Schnorr signature with server key:	GET			/schnorr/{server_key_id}/{signature}/{message_hash}
/// To generate ECDSA signature with server key:	GET			/ecdsa/{server_key_id}/{signature}/{message_hash}
/// To change servers set:							POST		/admin/servers_set_change/{old_signature}/{new_signature} + BODY: json array of hex-encoded nodes ids

pub struct KeyServerHttpListener {
	_runtime: Runtime,
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
	/// Generate Schnorr signature for the message.
	SchnorrSignMessage(ServerKeyId, RequestSignature, MessageHash),
	/// Generate ECDSA signature for the message.
	EcdsaSignMessage(ServerKeyId, RequestSignature, MessageHash),
	/// Change servers set.
	ChangeServersSet(RequestSignature, RequestSignature, BTreeSet<NodeId>),
}

/// Cloneable http handler
#[derive(Clone)]
struct KeyServerHttpHandler {
	handler: Arc<KeyServerSharedHttpHandler>,
}

/// Shared http handler
struct KeyServerSharedHttpHandler {
	key_server: Weak<KeyServer>,
}

impl KeyServerHttpListener {
	/// Start KeyServer http listener
	pub fn start(listener_address: NodeAddress, key_server: Weak<KeyServer>) -> Result<Self, Error> {
		let shared_handler = Arc::new(KeyServerSharedHttpHandler {
			key_server: key_server,
		});

		let mut runtime = Runtime::new()?;
		let listener_address = format!("{}:{}", listener_address.address, listener_address.port).parse()?;
		let listener = TcpListener::bind(&listener_address)?;

		let shared_handler2 = shared_handler.clone();

		let server = listener.incoming()
			.map_err(|e| warn!("Key server listener error: {:?}", e))
			.for_each(move |socket| {
				let http: Http<Chunk> = Http::new();
				let serve = http.serve_connection(socket, KeyServerHttpHandler {
					handler: shared_handler2.clone(),
				}).map(|_| ()).map_err(|e| {
					warn!("Key server handler error: {:?}", e);
				});

				tokio::spawn(serve)
			});

		runtime.spawn(server);

		let listener = KeyServerHttpListener {
			_runtime: runtime,
			_handler: shared_handler,
		};

		Ok(listener)
	}
}

impl KeyServerHttpHandler {
	fn process(self, req_method: HttpMethod, req_uri: Uri, path: &str, req_body: &[u8]) -> HttpResponse {
		match parse_request(&req_method, &path, &req_body) {
			Request::GenerateServerKey(document, signature, threshold) => {
				return_server_public_key(&req_uri, self.handler.key_server.upgrade()
					.map(|key_server| key_server.generate_key(&document, &signature.into(), threshold))
					.unwrap_or(Err(Error::Internal("KeyServer is already destroyed".into())))
					.map_err(|err| {
						warn!(target: "secretstore", "GenerateServerKey request {} has failed with: {}", req_uri, err);
						err
					}))
			},
			Request::StoreDocumentKey(document, signature, common_point, encrypted_document_key) => {
				return_empty(&req_uri, self.handler.key_server.upgrade()
					.map(|key_server| key_server.store_document_key(&document, &signature.into(), common_point, encrypted_document_key))
					.unwrap_or(Err(Error::Internal("KeyServer is already destroyed".into())))
					.map_err(|err| {
						warn!(target: "secretstore", "StoreDocumentKey request {} has failed with: {}", req_uri, err);
						err
					}))
			},
			Request::GenerateDocumentKey(document, signature, threshold) => {
				return_document_key(&req_uri, self.handler.key_server.upgrade()
					.map(|key_server| key_server.generate_document_key(&document, &signature.into(), threshold))
					.unwrap_or(Err(Error::Internal("KeyServer is already destroyed".into())))
					.map_err(|err| {
						warn!(target: "secretstore", "GenerateDocumentKey request {} has failed with: {}", req_uri, err);
						err
					}))
			},
			Request::GetDocumentKey(document, signature) => {
				return_document_key(&req_uri, self.handler.key_server.upgrade()
					.map(|key_server| key_server.restore_document_key(&document, &signature.into()))
					.unwrap_or(Err(Error::Internal("KeyServer is already destroyed".into())))
					.map_err(|err| {
						warn!(target: "secretstore", "GetDocumentKey request {} has failed with: {}", req_uri, err);
						err
					}))
			},
			Request::GetDocumentKeyShadow(document, signature) => {
				return_document_key_shadow(&req_uri, self.handler.key_server.upgrade()
					.map(|key_server| key_server.restore_document_key_shadow(&document, &signature.into()))
					.unwrap_or(Err(Error::Internal("KeyServer is already destroyed".into())))
					.map_err(|err| {
						warn!(target: "secretstore", "GetDocumentKeyShadow request {} has failed with: {}", req_uri, err);
						err
					}))
			},
			Request::SchnorrSignMessage(document, signature, message_hash) => {
				return_message_signature(&req_uri, self.handler.key_server.upgrade()
					.map(|key_server| key_server.sign_message_schnorr(&document, &signature.into(), message_hash))
					.unwrap_or(Err(Error::Internal("KeyServer is already destroyed".into())))
					.map_err(|err| {
						warn!(target: "secretstore", "SchnorrSignMessage request {} has failed with: {}", req_uri, err);
						err
					}))
				},
			Request::EcdsaSignMessage(document, signature, message_hash) => {
				return_message_signature(&req_uri, self.handler.key_server.upgrade()
					.map(|key_server| key_server.sign_message_ecdsa(&document, &signature.into(), message_hash))
					.unwrap_or(Err(Error::Internal("KeyServer is already destroyed".into())))
					.map_err(|err| {
						warn!(target: "secretstore", "EcdsaSignMessage request {} has failed with: {}", req_uri, err);
						err
					}))
			},
			Request::ChangeServersSet(old_set_signature, new_set_signature, new_servers_set) => {
				return_empty(&req_uri, self.handler.key_server.upgrade()
					.map(|key_server| key_server.change_servers_set(old_set_signature, new_set_signature, new_servers_set))
					.unwrap_or(Err(Error::Internal("KeyServer is already destroyed".into())))
					.map_err(|err| {
						warn!(target: "secretstore", "ChangeServersSet request {} has failed with: {}", req_uri, err);
						err
					}))
				},
			Request::Invalid => {
				warn!(target: "secretstore", "Ignoring invalid {}-request {}", req_method, req_uri);
				HttpResponse::new().with_status(HttpStatusCode::BadRequest)
			},
		}
	}
}

impl Service for KeyServerHttpHandler {
	type Request = HttpRequest;
	type Response = HttpResponse;
	type Error = hyper::Error;
	type Future = Box<Future<Item=Self::Response, Error=Self::Error> + Send>;

	fn call(&self, req: HttpRequest) -> Self::Future {
		if req.headers().has::<header::Origin>() {
			warn!(target: "secretstore", "Ignoring {}-request {} with Origin header", req.method(), req.uri());
			return Box::new(future::ok(HttpResponse::new().with_status(HttpStatusCode::NotFound)));
		}

		let req_method = req.method().clone();
		let req_uri = req.uri().clone();
		// We cannot consume Self because of the Service trait requirement.
		let this = self.clone();

		Box::new(req.body().concat2().map(move |body| {
			let path = req_uri.path().to_string();
			if path.starts_with("/") {
				this.process(req_method, req_uri, &path, &body)
			} else {
				warn!(target: "secretstore", "Ignoring invalid {}-request {}", req_method, req_uri);
				HttpResponse::new().with_status(HttpStatusCode::NotFound)
			}
		}))
	}
}

fn return_empty(req_uri: &Uri, empty: Result<(), Error>) -> HttpResponse {
	return_bytes::<i32>(req_uri, empty.map(|_| None))
}

fn return_server_public_key(req_uri: &Uri, server_public: Result<Public, Error>) -> HttpResponse {
	return_bytes(req_uri, server_public.map(|k| Some(SerializablePublic(k))))
}

fn return_message_signature(req_uri: &Uri, signature: Result<EncryptedDocumentKey, Error>) -> HttpResponse {
	return_bytes(req_uri, signature.map(|s| Some(SerializableBytes(s))))
}

fn return_document_key(req_uri: &Uri, document_key: Result<EncryptedDocumentKey, Error>) -> HttpResponse {
	return_bytes(req_uri, document_key.map(|k| Some(SerializableBytes(k))))
}

fn return_document_key_shadow(req_uri: &Uri, document_key_shadow: Result<EncryptedDocumentKeyShadow, Error>) -> HttpResponse {
	return_bytes(req_uri, document_key_shadow.map(|k| Some(SerializableEncryptedDocumentKeyShadow {
		decrypted_secret: k.decrypted_secret.into(),
		common_point: k.common_point.expect("always filled when requesting document_key_shadow; qed").into(),
		decrypt_shadows: k.decrypt_shadows.expect("always filled when requesting document_key_shadow; qed").into_iter().map(Into::into).collect(),
	})))
}

fn return_bytes<T: Serialize>(req_uri: &Uri, result: Result<Option<T>, Error>) -> HttpResponse {
	match result {
		Ok(Some(result)) => match serde_json::to_vec(&result) {
			Ok(result) => HttpResponse::new()
				.with_header(header::ContentType::json())
				.with_body(result),
			Err(err) => {
				warn!(target: "secretstore", "response to request {} has failed with: {}", req_uri, err);
				HttpResponse::new().with_status(HttpStatusCode::InternalServerError)
			}
		},
		Ok(None) => HttpResponse::new().with_status(HttpStatusCode::Ok),
		Err(err) => return_error(err),
	}
}

fn return_error(err: Error) -> HttpResponse {
	let mut res = match err {
		Error::InsufficientRequesterData(_) => HttpResponse::new().with_status(HttpStatusCode::BadRequest),
		Error::AccessDenied => HttpResponse::new().with_status(HttpStatusCode::Forbidden),
		Error::DocumentNotFound => HttpResponse::new().with_status(HttpStatusCode::NotFound),
		Error::Hyper(_) => HttpResponse::new().with_status(HttpStatusCode::BadRequest),
		Error::Serde(_) => HttpResponse::new().with_status(HttpStatusCode::BadRequest),
		Error::Database(_) => HttpResponse::new().with_status(HttpStatusCode::InternalServerError),
		Error::Internal(_) => HttpResponse::new().with_status(HttpStatusCode::InternalServerError),
	};

	// return error text. ignore errors when returning error
	let error_text = format!("\"{}\"", err);
	if let Ok(error_text) = serde_json::to_vec(&error_text) {
		res.headers_mut().set(header::ContentType::json());
		res.set_body(error_text);
	}

	res
}

fn parse_request(method: &HttpMethod, uri_path: &str, body: &[u8]) -> Request {
	let uri_path = match percent_decode(uri_path.as_bytes()).decode_utf8() {
		Ok(path) => path,
		Err(_) => return Request::Invalid,
	};

	let path: Vec<String> = uri_path.trim_left_matches('/').split('/').map(Into::into).collect();
	if path.len() == 0 {
		return Request::Invalid;
	}

	if path[0] == "admin" {
		return parse_admin_request(method, path, body);
	}

	let (prefix, args_offset) = if &path[0] == "shadow" || &path[0] == "schnorr" || &path[0] == "ecdsa"
		{ (&*path[0], 1) } else { ("", 0) };
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
	match (prefix, args_count, method, threshold, message_hash, common_point, encrypted_key) {
		("shadow", 3, &HttpMethod::Post, Some(Ok(threshold)), _, _, _) =>
			Request::GenerateServerKey(document, signature, threshold),
		("shadow", 4, &HttpMethod::Post, _, _, Some(Ok(common_point)), Some(Ok(encrypted_key))) =>
			Request::StoreDocumentKey(document, signature, common_point, encrypted_key),
		("", 3, &HttpMethod::Post, Some(Ok(threshold)), _, _, _) =>
			Request::GenerateDocumentKey(document, signature, threshold),
		("", 2, &HttpMethod::Get, _, _, _, _) =>
			Request::GetDocumentKey(document, signature),
		("shadow", 2, &HttpMethod::Get, _, _, _, _) =>
			Request::GetDocumentKeyShadow(document, signature),
		("schnorr", 3, &HttpMethod::Get, _, Some(Ok(message_hash)), _, _) =>
			Request::SchnorrSignMessage(document, signature, message_hash),
		("ecdsa", 3, &HttpMethod::Get, _, Some(Ok(message_hash)), _, _) =>
			Request::EcdsaSignMessage(document, signature, message_hash),
		_ => Request::Invalid,
	}
}

fn parse_admin_request(method: &HttpMethod, path: Vec<String>, body: &[u8]) -> Request {
	let args_count = path.len();
	if *method != HttpMethod::Post || args_count != 4 || path[1] != "servers_set_change" {
		return Request::Invalid;
	}

	let old_set_signature = match path[2].parse() {
		Ok(signature) => signature,
		_ => return Request::Invalid,
	};

	let new_set_signature = match path[3].parse() {
		Ok(signature) => signature,
		_ => return Request::Invalid,
	};

	let new_servers_set: BTreeSet<SerializablePublic> = match serde_json::from_slice(body) {
		Ok(new_servers_set) => new_servers_set,
		_ => return Request::Invalid,
	};

	Request::ChangeServersSet(old_set_signature, new_set_signature,
		new_servers_set.into_iter().map(Into::into).collect())
}

#[cfg(test)]
mod tests {
	use std::sync::Arc;
	use hyper::Method as HttpMethod;
	use ethkey::Public;
	use traits::KeyServer;
	use key_server::tests::DummyKeyServer;
	use types::all::NodeAddress;
	use super::{parse_request, Request, KeyServerHttpListener};

	#[test]
	fn http_listener_successfully_drops() {
		let key_server: Arc<KeyServer> = Arc::new(DummyKeyServer::default());
		let address = NodeAddress { address: "127.0.0.1".into(), port: 9000 };
		let listener = KeyServerHttpListener::start(address, Arc::downgrade(&key_server)).unwrap();
		drop(listener);
	}

	#[test]
	fn parse_request_successful() {
		// POST		/shadow/{server_key_id}/{signature}/{threshold}						=> generate server key
		assert_eq!(parse_request(&HttpMethod::Post, "/shadow/0000000000000000000000000000000000000000000000000000000000000001/a199fb39e11eefb61c78a4074a53c0d4424600a3e74aad4fb9d93a26c30d067e1d4d29936de0c73f19827394a1dd049480a0d581aee7ae7546968da7d3d1c2fd01/2", Default::default()),
			Request::GenerateServerKey("0000000000000000000000000000000000000000000000000000000000000001".into(),
				"a199fb39e11eefb61c78a4074a53c0d4424600a3e74aad4fb9d93a26c30d067e1d4d29936de0c73f19827394a1dd049480a0d581aee7ae7546968da7d3d1c2fd01".parse().unwrap(),
				2));
		// POST		/shadow/{server_key_id}/{signature}/{common_point}/{encrypted_key}	=> store encrypted document key
		assert_eq!(parse_request(&HttpMethod::Post, "/shadow/0000000000000000000000000000000000000000000000000000000000000001/a199fb39e11eefb61c78a4074a53c0d4424600a3e74aad4fb9d93a26c30d067e1d4d29936de0c73f19827394a1dd049480a0d581aee7ae7546968da7d3d1c2fd01/b486d3840218837b035c66196ecb15e6b067ca20101e11bd5e626288ab6806ecc70b8307012626bd512bad1559112d11d21025cef48cc7a1d2f3976da08f36c8/1395568277679f7f583ab7c0992da35f26cde57149ee70e524e49bdae62db3e18eb96122501e7cbb798b784395d7bb5a499edead0706638ad056d886e56cf8fb", Default::default()),
			Request::StoreDocumentKey("0000000000000000000000000000000000000000000000000000000000000001".into(),
				"a199fb39e11eefb61c78a4074a53c0d4424600a3e74aad4fb9d93a26c30d067e1d4d29936de0c73f19827394a1dd049480a0d581aee7ae7546968da7d3d1c2fd01".parse().unwrap(),
				"b486d3840218837b035c66196ecb15e6b067ca20101e11bd5e626288ab6806ecc70b8307012626bd512bad1559112d11d21025cef48cc7a1d2f3976da08f36c8".parse().unwrap(),
				"1395568277679f7f583ab7c0992da35f26cde57149ee70e524e49bdae62db3e18eb96122501e7cbb798b784395d7bb5a499edead0706638ad056d886e56cf8fb".parse().unwrap()));
		// POST		/{server_key_id}/{signature}/{threshold}							=> generate server && document key
		assert_eq!(parse_request(&HttpMethod::Post, "/0000000000000000000000000000000000000000000000000000000000000001/a199fb39e11eefb61c78a4074a53c0d4424600a3e74aad4fb9d93a26c30d067e1d4d29936de0c73f19827394a1dd049480a0d581aee7ae7546968da7d3d1c2fd01/2", Default::default()),
			Request::GenerateDocumentKey("0000000000000000000000000000000000000000000000000000000000000001".into(),
				"a199fb39e11eefb61c78a4074a53c0d4424600a3e74aad4fb9d93a26c30d067e1d4d29936de0c73f19827394a1dd049480a0d581aee7ae7546968da7d3d1c2fd01".parse().unwrap(),
				2));
		// GET		/{server_key_id}/{signature}										=> get document key
		assert_eq!(parse_request(&HttpMethod::Get, "/0000000000000000000000000000000000000000000000000000000000000001/a199fb39e11eefb61c78a4074a53c0d4424600a3e74aad4fb9d93a26c30d067e1d4d29936de0c73f19827394a1dd049480a0d581aee7ae7546968da7d3d1c2fd01", Default::default()),
			Request::GetDocumentKey("0000000000000000000000000000000000000000000000000000000000000001".into(),
				"a199fb39e11eefb61c78a4074a53c0d4424600a3e74aad4fb9d93a26c30d067e1d4d29936de0c73f19827394a1dd049480a0d581aee7ae7546968da7d3d1c2fd01".parse().unwrap()));
		assert_eq!(parse_request(&HttpMethod::Get, "/%30000000000000000000000000000000000000000000000000000000000000001/a199fb39e11eefb61c78a4074a53c0d4424600a3e74aad4fb9d93a26c30d067e1d4d29936de0c73f19827394a1dd049480a0d581aee7ae7546968da7d3d1c2fd01", Default::default()),
			Request::GetDocumentKey("0000000000000000000000000000000000000000000000000000000000000001".into(),
				"a199fb39e11eefb61c78a4074a53c0d4424600a3e74aad4fb9d93a26c30d067e1d4d29936de0c73f19827394a1dd049480a0d581aee7ae7546968da7d3d1c2fd01".parse().unwrap()));
		// GET		/shadow/{server_key_id}/{signature}									=> get document key shadow
		assert_eq!(parse_request(&HttpMethod::Get, "/shadow/0000000000000000000000000000000000000000000000000000000000000001/a199fb39e11eefb61c78a4074a53c0d4424600a3e74aad4fb9d93a26c30d067e1d4d29936de0c73f19827394a1dd049480a0d581aee7ae7546968da7d3d1c2fd01", Default::default()),
			Request::GetDocumentKeyShadow("0000000000000000000000000000000000000000000000000000000000000001".into(),
				"a199fb39e11eefb61c78a4074a53c0d4424600a3e74aad4fb9d93a26c30d067e1d4d29936de0c73f19827394a1dd049480a0d581aee7ae7546968da7d3d1c2fd01".parse().unwrap()));
		// GET		/schnorr/{server_key_id}/{signature}/{message_hash}					=> schnorr-sign message with server key
		assert_eq!(parse_request(&HttpMethod::Get, "/schnorr/0000000000000000000000000000000000000000000000000000000000000001/a199fb39e11eefb61c78a4074a53c0d4424600a3e74aad4fb9d93a26c30d067e1d4d29936de0c73f19827394a1dd049480a0d581aee7ae7546968da7d3d1c2fd01/281b6bf43cb86d0dc7b98e1b7def4a80f3ce16d28d2308f934f116767306f06c", Default::default()),
			Request::SchnorrSignMessage("0000000000000000000000000000000000000000000000000000000000000001".into(),
				"a199fb39e11eefb61c78a4074a53c0d4424600a3e74aad4fb9d93a26c30d067e1d4d29936de0c73f19827394a1dd049480a0d581aee7ae7546968da7d3d1c2fd01".parse().unwrap(),
				"281b6bf43cb86d0dc7b98e1b7def4a80f3ce16d28d2308f934f116767306f06c".parse().unwrap()));
		// GET		/ecdsa/{server_key_id}/{signature}/{message_hash}					=> ecdsa-sign message with server key
		assert_eq!(parse_request(&HttpMethod::Get, "/ecdsa/0000000000000000000000000000000000000000000000000000000000000001/a199fb39e11eefb61c78a4074a53c0d4424600a3e74aad4fb9d93a26c30d067e1d4d29936de0c73f19827394a1dd049480a0d581aee7ae7546968da7d3d1c2fd01/281b6bf43cb86d0dc7b98e1b7def4a80f3ce16d28d2308f934f116767306f06c", Default::default()),
			Request::EcdsaSignMessage("0000000000000000000000000000000000000000000000000000000000000001".into(),
				"a199fb39e11eefb61c78a4074a53c0d4424600a3e74aad4fb9d93a26c30d067e1d4d29936de0c73f19827394a1dd049480a0d581aee7ae7546968da7d3d1c2fd01".parse().unwrap(),
				"281b6bf43cb86d0dc7b98e1b7def4a80f3ce16d28d2308f934f116767306f06c".parse().unwrap()));
		// POST		/admin/servers_set_change/{old_set_signature}/{new_set_signature} + body
		let node1: Public = "843645726384530ffb0c52f175278143b5a93959af7864460f5a4fec9afd1450cfb8aef63dec90657f43f55b13e0a73c7524d4e9a13c051b4e5f1e53f39ecd91".parse().unwrap();
		let node2: Public = "07230e34ebfe41337d3ed53b186b3861751f2401ee74b988bba55694e2a6f60c757677e194be2e53c3523cc8548694e636e6acb35c4e8fdc5e29d28679b9b2f3".parse().unwrap();
		let nodes = vec![node1, node2].into_iter().collect();
		assert_eq!(parse_request(&HttpMethod::Post, "/admin/servers_set_change/a199fb39e11eefb61c78a4074a53c0d4424600a3e74aad4fb9d93a26c30d067e1d4d29936de0c73f19827394a1dd049480a0d581aee7ae7546968da7d3d1c2fd01/b199fb39e11eefb61c78a4074a53c0d4424600a3e74aad4fb9d93a26c30d067e1d4d29936de0c73f19827394a1dd049480a0d581aee7ae7546968da7d3d1c2fd01",
			&r#"["0x843645726384530ffb0c52f175278143b5a93959af7864460f5a4fec9afd1450cfb8aef63dec90657f43f55b13e0a73c7524d4e9a13c051b4e5f1e53f39ecd91",
				"0x07230e34ebfe41337d3ed53b186b3861751f2401ee74b988bba55694e2a6f60c757677e194be2e53c3523cc8548694e636e6acb35c4e8fdc5e29d28679b9b2f3"]"#.as_bytes()),
			Request::ChangeServersSet(
				"a199fb39e11eefb61c78a4074a53c0d4424600a3e74aad4fb9d93a26c30d067e1d4d29936de0c73f19827394a1dd049480a0d581aee7ae7546968da7d3d1c2fd01".parse().unwrap(),
				"b199fb39e11eefb61c78a4074a53c0d4424600a3e74aad4fb9d93a26c30d067e1d4d29936de0c73f19827394a1dd049480a0d581aee7ae7546968da7d3d1c2fd01".parse().unwrap(),
				nodes,
			));
	}

	#[test]
	fn parse_request_failed() {
		assert_eq!(parse_request(&HttpMethod::Get, "", Default::default()), Request::Invalid);
		assert_eq!(parse_request(&HttpMethod::Get, "/shadow", Default::default()), Request::Invalid);
		assert_eq!(parse_request(&HttpMethod::Get, "///2", Default::default()), Request::Invalid);
		assert_eq!(parse_request(&HttpMethod::Get, "/shadow///2", Default::default()), Request::Invalid);
		assert_eq!(parse_request(&HttpMethod::Get, "/0000000000000000000000000000000000000000000000000000000000000001", Default::default()), Request::Invalid);
		assert_eq!(parse_request(&HttpMethod::Get, "/0000000000000000000000000000000000000000000000000000000000000001/", Default::default()), Request::Invalid);
		assert_eq!(parse_request(&HttpMethod::Get, "/a/b", Default::default()), Request::Invalid);
		assert_eq!(parse_request(&HttpMethod::Get, "/schnorr/0000000000000000000000000000000000000000000000000000000000000001/a199fb39e11eefb61c78a4074a53c0d4424600a3e74aad4fb9d93a26c30d067e1d4d29936de0c73f19827394a1dd049480a0d581aee7ae7546968da7d3d1c2fd01/0000000000000000000000000000000000000000000000000000000000000002/0000000000000000000000000000000000000000000000000000000000000002", Default::default()), Request::Invalid);
		assert_eq!(parse_request(&HttpMethod::Get, "/ecdsa/0000000000000000000000000000000000000000000000000000000000000001/a199fb39e11eefb61c78a4074a53c0d4424600a3e74aad4fb9d93a26c30d067e1d4d29936de0c73f19827394a1dd049480a0d581aee7ae7546968da7d3d1c2fd01/0000000000000000000000000000000000000000000000000000000000000002/0000000000000000000000000000000000000000000000000000000000000002", Default::default()), Request::Invalid);
		assert_eq!(parse_request(&HttpMethod::Post, "/admin/servers_set_change/xxx/yyy",
			&r#"["0x843645726384530ffb0c52f175278143b5a93959af7864460f5a4fec9afd1450cfb8aef63dec90657f43f55b13e0a73c7524d4e9a13c051b4e5f1e53f39ecd91",
				"0x07230e34ebfe41337d3ed53b186b3861751f2401ee74b988bba55694e2a6f60c757677e194be2e53c3523cc8548694e636e6acb35c4e8fdc5e29d28679b9b2f3"]"#.as_bytes()),
			Request::Invalid);
		assert_eq!(parse_request(&HttpMethod::Post, "/admin/servers_set_change/a199fb39e11eefb61c78a4074a53c0d4424600a3e74aad4fb9d93a26c30d067e1d4d29936de0c73f19827394a1dd049480a0d581aee7ae7546968da7d3d1c2fd01/a199fb39e11eefb61c78a4074a53c0d4424600a3e74aad4fb9d93a26c30d067e1d4d29936de0c73f19827394a1dd049480a0d581aee7ae7546968da7d3d1c2fd01", "".as_bytes()),
			Request::Invalid);
	}
}
