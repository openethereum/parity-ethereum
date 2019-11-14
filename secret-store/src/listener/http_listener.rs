// Copyright 2015-2019 Parity Technologies (UK) Ltd.
// This file is part of Parity Ethereum.

// Parity Ethereum is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity Ethereum is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity Ethereum.  If not, see <http://www.gnu.org/licenses/>.

use std::collections::BTreeSet;
use std::sync::{Arc, Weak};
use futures::future::{ok, result};
use hyper::{self, Uri, Request as HttpRequest, Response as HttpResponse, Method as HttpMethod,
	StatusCode as HttpStatusCode, Body,
	header::{self, HeaderValue},
	server::conn::Http,
	service::Service,
};
use serde::Serialize;
use serde_json;
use tokio;
use tokio::net::TcpListener;
use parity_runtime::Executor;
use futures::{future, Future, Stream};
use percent_encoding::percent_decode;

use traits::KeyServer;
use serialization::{SerializableEncryptedDocumentKeyShadow, SerializableBytes, SerializablePublic};
use types::{Error, Public, MessageHash, NodeAddress, RequestSignature, ServerKeyId,
	EncryptedDocumentKey, EncryptedDocumentKeyShadow, NodeId};
use jsonrpc_server_utils::cors::{self, AllowCors, AccessControlAllowOrigin};

/// Key server http-requests listener. Available requests:
/// To generate server key:							POST		/shadow/{server_key_id}/{signature}/{threshold}
/// To store pregenerated encrypted document key: 	POST		/shadow/{server_key_id}/{signature}/{common_point}/{encrypted_key}
/// To generate server && document key:				POST		/{server_key_id}/{signature}/{threshold}
/// To get public portion of server key:			GET			/server/{server_key_id}/{signature}
/// To get document key:							GET			/{server_key_id}/{signature}
/// To get document key shadow:						GET			/shadow/{server_key_id}/{signature}
/// To generate Schnorr signature with server key:	GET			/schnorr/{server_key_id}/{signature}/{message_hash}
/// To generate ECDSA signature with server key:	GET			/ecdsa/{server_key_id}/{signature}/{message_hash}
/// To change servers set:							POST		/admin/servers_set_change/{old_signature}/{new_signature} + BODY: json array of hex-encoded nodes ids

type CorsDomains = Option<Vec<AccessControlAllowOrigin>>;

pub struct KeyServerHttpListener {
	_executor: Executor,
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
	/// Request public portion of server key.
	GetServerKey(ServerKeyId, RequestSignature),
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
	cors: CorsDomains,
}

/// Shared http handler
struct KeyServerSharedHttpHandler {
	key_server: Weak<dyn KeyServer>,
}


impl KeyServerHttpListener {
	/// Start KeyServer http listener
	pub fn start(listener_address: NodeAddress, cors_domains: Option<Vec<String>>, key_server: Weak<dyn KeyServer>, executor: Executor) -> Result<Self, Error> {
		let shared_handler = Arc::new(KeyServerSharedHttpHandler {
			key_server: key_server,
		});
		let cors: CorsDomains = cors_domains.map(|domains| domains.into_iter().map(AccessControlAllowOrigin::from).collect());
		let listener_address = format!("{}:{}", listener_address.address, listener_address.port).parse()?;
		let listener = TcpListener::bind(&listener_address)?;

		let shared_handler2 = shared_handler.clone();

		let server = listener.incoming()
			.map_err(|e| warn!("Key server listener error: {:?}", e))
			.for_each(move |socket| {
				let http = Http::new();
				let serve = http.serve_connection(socket,
					KeyServerHttpHandler { handler: shared_handler2.clone(), cors: cors.clone() }
				).map(|_| ()).map_err(|e| {
					warn!("Key server handler error: {:?}", e);
				});

				tokio::spawn(serve)
			});

		executor.spawn(server);

		let listener = KeyServerHttpListener {
			_executor: executor,
			_handler: shared_handler,
		};

		Ok(listener)
	}
}

impl KeyServerHttpHandler {
	fn key_server(&self) -> Result<Arc<dyn KeyServer>, Error> {
		self.handler.key_server.upgrade()
			.ok_or_else(|| Error::Internal("KeyServer is already destroyed".into()))
	}

	fn process(
		self,
		req_method: HttpMethod,
		req_uri: Uri,
		path: &str,
		req_body: &[u8],
		cors: AllowCors<AccessControlAllowOrigin>,
	) -> Box<dyn Future<Item=HttpResponse<Body>, Error=hyper::Error> + Send> {
		match parse_request(&req_method, &path, &req_body) {
			Request::GenerateServerKey(document, signature, threshold) =>
				Box::new(result(self.key_server())
					.and_then(move |key_server| key_server.generate_key(document, signature.into(), threshold))
					.then(move |result| ok(return_server_public_key("GenerateServerKey", &req_uri, cors, result)))),
			Request::StoreDocumentKey(document, signature, common_point, encrypted_document_key) =>
				Box::new(result(self.key_server())
					.and_then(move |key_server| key_server.store_document_key(
						document,
						signature.into(),
						common_point,
						encrypted_document_key,
					))
					.then(move |result| ok(return_empty("StoreDocumentKey", &req_uri, cors, result)))),
			Request::GenerateDocumentKey(document, signature, threshold) =>
				Box::new(result(self.key_server())
					.and_then(move |key_server| key_server.generate_document_key(
						document,
						signature.into(),
						threshold,
					))
					.then(move |result| ok(return_document_key("GenerateDocumentKey", &req_uri, cors, result)))),
			Request::GetServerKey(document, signature) =>
				Box::new(result(self.key_server())
					.and_then(move |key_server| key_server.restore_key_public(
						document,
						signature.into(),
					))
					.then(move |result| ok(return_server_public_key("GetServerKey", &req_uri, cors, result)))),
			Request::GetDocumentKey(document, signature) =>
				Box::new(result(self.key_server())
					.and_then(move |key_server| key_server.restore_document_key(document, signature.into()))
					.then(move |result| ok(return_document_key("GetDocumentKey", &req_uri, cors, result)))),
			Request::GetDocumentKeyShadow(document, signature) =>
				Box::new(result(self.key_server())
					.and_then(move |key_server| key_server.restore_document_key_shadow(document, signature.into()))
					.then(move |result| ok(return_document_key_shadow("GetDocumentKeyShadow", &req_uri, cors, result)))),
			Request::SchnorrSignMessage(document, signature, message_hash) =>
				Box::new(result(self.key_server())
					.and_then(move |key_server| key_server.sign_message_schnorr(
						document,
						signature.into(),
						message_hash,
					))
					.then(move |result| ok(return_message_signature("SchnorrSignMessage", &req_uri, cors, result)))),
			Request::EcdsaSignMessage(document, signature, message_hash) =>
				Box::new(result(self.key_server())
					.and_then(move |key_server| key_server.sign_message_ecdsa(
						document,
						signature.into(),
						message_hash,
					))
					.then(move |result| ok(return_message_signature("EcdsaSignMessage", &req_uri, cors, result)))),
			Request::ChangeServersSet(old_set_signature, new_set_signature, new_servers_set) =>
				Box::new(result(self.key_server())
					.and_then(move |key_server| key_server.change_servers_set(
						old_set_signature,
						new_set_signature,
						new_servers_set,
					))
					.then(move |result| ok(return_empty("ChangeServersSet", &req_uri, cors, result)))),
			Request::Invalid => {
				warn!(target: "secretstore", "Ignoring invalid {}-request {}", req_method, req_uri);
				Box::new(ok(HttpResponse::builder()
					.status(HttpStatusCode::BAD_REQUEST)
					.body(Body::empty())
					.expect("Nothing to parse, cannot fail; qed")))
			},
		}
	}
}

impl Service for KeyServerHttpHandler {
	type ReqBody = Body;
	type ResBody = Body;
	type Error = hyper::Error;
	type Future = Box<dyn Future<Item = HttpResponse<Self::ResBody>, Error=Self::Error> + Send>;

	fn call(&mut self, req: HttpRequest<Body>) -> Self::Future {
		let cors = cors::get_cors_allow_origin(
			req.headers().get(header::ORIGIN).and_then(|value| value.to_str().ok()),
			req.headers().get(header::HOST).and_then(|value| value.to_str().ok()),
			&self.cors
		);
		match cors {
			AllowCors::Invalid => {
				warn!(target: "secretstore", "Ignoring {}-request {} with unauthorized Origin header", req.method(), req.uri());
				Box::new(future::ok(HttpResponse::builder()
					.status(HttpStatusCode::NOT_FOUND)
					.body(Body::empty())
					.expect("Nothing to parse, cannot fail; qed")))
			},
			_ => {
				let req_method = req.method().clone();
				let req_uri = req.uri().clone();
				let path = req_uri.path().to_string();
				// We cannot consume Self because of the Service trait requirement.
				let this = self.clone();

				Box::new(req.into_body().concat2()
					.and_then(move |body| this.process(req_method, req_uri, &path, &body, cors)))
			}
		}
	}
}

fn return_empty(req_type: &str, req_uri: &Uri, cors: AllowCors<AccessControlAllowOrigin>, empty: Result<(), Error>) -> HttpResponse<Body> {
	return_bytes::<i32>(req_type, req_uri, cors, empty.map(|_| None))
}

fn return_server_public_key(
	req_type: &str,
	req_uri: &Uri,
	cors: AllowCors<AccessControlAllowOrigin>,
	server_public: Result<Public, Error>,
) -> HttpResponse<Body> {
	return_bytes(req_type, req_uri, cors, server_public.map(|k| Some(SerializablePublic(k))))
}

fn return_message_signature(
	req_type: &str,
	req_uri: &Uri,
	cors: AllowCors<AccessControlAllowOrigin>,
	signature: Result<EncryptedDocumentKey, Error>,
) -> HttpResponse<Body> {
	return_bytes(req_type, req_uri, cors, signature.map(|s| Some(SerializableBytes(s))))
}

fn return_document_key(
	req_type: &str,
	req_uri: &Uri,
	cors: AllowCors<AccessControlAllowOrigin>,
	document_key: Result<EncryptedDocumentKey, Error>,
) -> HttpResponse<Body> {
	return_bytes(req_type, req_uri, cors, document_key.map(|k| Some(SerializableBytes(k))))
}

fn return_document_key_shadow(
	req_type: &str,
	req_uri: &Uri,
	cors: AllowCors<AccessControlAllowOrigin>,
	document_key_shadow: Result<EncryptedDocumentKeyShadow, Error>,
) -> HttpResponse<Body> {
	return_bytes(req_type, req_uri, cors, document_key_shadow.map(|k| Some(SerializableEncryptedDocumentKeyShadow {
		decrypted_secret: k.decrypted_secret.into(),
		common_point: k.common_point.expect("always filled when requesting document_key_shadow; qed").into(),
		decrypt_shadows: k.decrypt_shadows.expect("always filled when requesting document_key_shadow; qed").into_iter().map(Into::into).collect()
	})))
}

fn return_bytes<T: Serialize>(
	req_type: &str,
	req_uri: &Uri,
	cors: AllowCors<AccessControlAllowOrigin>,
	result: Result<Option<T>, Error>,
) -> HttpResponse<Body> {
	match result {
		Ok(Some(result)) => match serde_json::to_vec(&result) {
			Ok(result) => {
				let body: Body = result.into();
				let mut builder = HttpResponse::builder();
				builder.header(header::CONTENT_TYPE, HeaderValue::from_static("application/json; charset=utf-8"));
				if let AllowCors::Ok(AccessControlAllowOrigin::Value(origin)) = cors {
					builder.header(header::ACCESS_CONTROL_ALLOW_ORIGIN, origin.to_string());
				}
				builder.body(body).expect("Error creating http response")
			},
			Err(err) => {
				warn!(target: "secretstore", "response to request {} has failed with: {}", req_uri, err);
				HttpResponse::builder()
					.status(HttpStatusCode::INTERNAL_SERVER_ERROR)
					.body(Body::empty())
					.expect("Nothing to parse, cannot fail; qed")
			}
		},
		Ok(None) => {
			let mut builder = HttpResponse::builder();
			builder.status(HttpStatusCode::OK);
			if let AllowCors::Ok(AccessControlAllowOrigin::Value(origin)) = cors {
				builder.header(header::ACCESS_CONTROL_ALLOW_ORIGIN, origin.to_string());
			}
			builder.body(Body::empty()).expect("Nothing to parse, cannot fail; qed")
		},
		Err(err) => {
			warn!(target: "secretstore", "{} request {} has failed with: {}", req_type, req_uri, err);
			return_error(err)
		},
	}
}

fn return_error(err: Error) -> HttpResponse<Body> {
	let status = match err {
		| Error::AccessDenied
		| Error::ConsensusUnreachable
		| Error::ConsensusTemporaryUnreachable =>
			HttpStatusCode::FORBIDDEN,
		| Error::ServerKeyIsNotFound
		| Error::DocumentKeyIsNotFound =>
			HttpStatusCode::NOT_FOUND,
		| Error::InsufficientRequesterData(_)
		| Error::Hyper(_)
		| Error::Serde(_)
		| Error::DocumentKeyAlreadyStored
		| Error::ServerKeyAlreadyGenerated =>
			HttpStatusCode::BAD_REQUEST,
		_ => HttpStatusCode::INTERNAL_SERVER_ERROR,
	};

	let mut res = HttpResponse::builder();
	res.status(status);

	// return error text. ignore errors when returning error
	let error_text = format!("\"{}\"", err);
	if let Ok(error_text) = serde_json::to_vec(&error_text) {
		res.header(header::CONTENT_TYPE, HeaderValue::from_static("application/json; charset=utf-8"));
		res.body(error_text.into())
			.expect("`error_text` is a formatted string, parsing cannot fail; qed")
	} else {
		res.body(Body::empty())
			.expect("Nothing to parse, cannot fail; qed")
	}
}

fn parse_request(method: &HttpMethod, uri_path: &str, body: &[u8]) -> Request {
	let uri_path = match percent_decode(uri_path.as_bytes()).decode_utf8() {
		Ok(path) => path,
		Err(_) => return Request::Invalid,
	};

	let path: Vec<String> = uri_path.trim_start_matches('/').split('/').map(Into::into).collect();
	if path.len() == 0 {
		return Request::Invalid;
	}

	if path[0] == "admin" {
		return parse_admin_request(method, path, body);
	}

	let is_known_prefix = &path[0] == "shadow" || &path[0] == "schnorr" || &path[0] == "ecdsa" || &path[0] == "server";
	let (prefix, args_offset) = if is_known_prefix { (&*path[0], 1) } else { ("", 0) };
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
		("shadow", 3, &HttpMethod::POST, Some(Ok(threshold)), _, _, _) =>
			Request::GenerateServerKey(document, signature, threshold),
		("shadow", 4, &HttpMethod::POST, _, _, Some(Ok(common_point)), Some(Ok(encrypted_key))) =>
			Request::StoreDocumentKey(document, signature, common_point, encrypted_key),
		("", 3, &HttpMethod::POST, Some(Ok(threshold)), _, _, _) =>
			Request::GenerateDocumentKey(document, signature, threshold),
		("server", 2, &HttpMethod::GET, _, _, _, _) =>
			Request::GetServerKey(document, signature),
		("", 2, &HttpMethod::GET, _, _, _, _) =>
			Request::GetDocumentKey(document, signature),
		("shadow", 2, &HttpMethod::GET, _, _, _, _) =>
			Request::GetDocumentKeyShadow(document, signature),
		("schnorr", 3, &HttpMethod::GET, _, Some(Ok(message_hash)), _, _) =>
			Request::SchnorrSignMessage(document, signature, message_hash),
		("ecdsa", 3, &HttpMethod::GET, _, Some(Ok(message_hash)), _, _) =>
			Request::EcdsaSignMessage(document, signature, message_hash),
		_ => Request::Invalid,
	}
}

fn parse_admin_request(method: &HttpMethod, path: Vec<String>, body: &[u8]) -> Request {
	let args_count = path.len();
	if *method != HttpMethod::POST || args_count != 4 || path[1] != "servers_set_change" {
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
	use std::str::FromStr;
	use hyper::Method as HttpMethod;
	use crypto::publickey::Public;
	use traits::KeyServer;
	use key_server::tests::DummyKeyServer;
	use types::NodeAddress;
	use parity_runtime::Runtime;
	use ethereum_types::H256;
	use super::{parse_request, Request, KeyServerHttpListener};

	#[test]
	fn http_listener_successfully_drops() {
		let key_server: Arc<dyn KeyServer> = Arc::new(DummyKeyServer::default());
		let address = NodeAddress { address: "127.0.0.1".into(), port: 9000 };
		let runtime = Runtime::with_thread_count(1);
		let listener = KeyServerHttpListener::start(address, None, Arc::downgrade(&key_server),
			runtime.executor()).unwrap();
		drop(listener);
	}

	#[test]
	fn parse_request_successful() {
		// POST		/shadow/{server_key_id}/{signature}/{threshold}						=> generate server key
		assert_eq!(parse_request(&HttpMethod::POST, "/shadow/0000000000000000000000000000000000000000000000000000000000000001/a199fb39e11eefb61c78a4074a53c0d4424600a3e74aad4fb9d93a26c30d067e1d4d29936de0c73f19827394a1dd049480a0d581aee7ae7546968da7d3d1c2fd01/2", Default::default()),
			Request::GenerateServerKey(H256::from_str("0000000000000000000000000000000000000000000000000000000000000001").unwrap(),
				"a199fb39e11eefb61c78a4074a53c0d4424600a3e74aad4fb9d93a26c30d067e1d4d29936de0c73f19827394a1dd049480a0d581aee7ae7546968da7d3d1c2fd01".parse().unwrap(),
				2));
		// POST		/shadow/{server_key_id}/{signature}/{common_point}/{encrypted_key}	=> store encrypted document key
		assert_eq!(parse_request(&HttpMethod::POST, "/shadow/0000000000000000000000000000000000000000000000000000000000000001/a199fb39e11eefb61c78a4074a53c0d4424600a3e74aad4fb9d93a26c30d067e1d4d29936de0c73f19827394a1dd049480a0d581aee7ae7546968da7d3d1c2fd01/b486d3840218837b035c66196ecb15e6b067ca20101e11bd5e626288ab6806ecc70b8307012626bd512bad1559112d11d21025cef48cc7a1d2f3976da08f36c8/1395568277679f7f583ab7c0992da35f26cde57149ee70e524e49bdae62db3e18eb96122501e7cbb798b784395d7bb5a499edead0706638ad056d886e56cf8fb", Default::default()),
			Request::StoreDocumentKey(H256::from_str("0000000000000000000000000000000000000000000000000000000000000001").unwrap(),
				"a199fb39e11eefb61c78a4074a53c0d4424600a3e74aad4fb9d93a26c30d067e1d4d29936de0c73f19827394a1dd049480a0d581aee7ae7546968da7d3d1c2fd01".parse().unwrap(),
				"b486d3840218837b035c66196ecb15e6b067ca20101e11bd5e626288ab6806ecc70b8307012626bd512bad1559112d11d21025cef48cc7a1d2f3976da08f36c8".parse().unwrap(),
				"1395568277679f7f583ab7c0992da35f26cde57149ee70e524e49bdae62db3e18eb96122501e7cbb798b784395d7bb5a499edead0706638ad056d886e56cf8fb".parse().unwrap()));
		// POST		/{server_key_id}/{signature}/{threshold}							=> generate server && document key
		assert_eq!(parse_request(&HttpMethod::POST, "/0000000000000000000000000000000000000000000000000000000000000001/a199fb39e11eefb61c78a4074a53c0d4424600a3e74aad4fb9d93a26c30d067e1d4d29936de0c73f19827394a1dd049480a0d581aee7ae7546968da7d3d1c2fd01/2", Default::default()),
			Request::GenerateDocumentKey(H256::from_str("0000000000000000000000000000000000000000000000000000000000000001").unwrap(),
				"a199fb39e11eefb61c78a4074a53c0d4424600a3e74aad4fb9d93a26c30d067e1d4d29936de0c73f19827394a1dd049480a0d581aee7ae7546968da7d3d1c2fd01".parse().unwrap(),
				2));
		// GET		/server/{server_key_id}/{signature}									=> get public portion of server key
		assert_eq!(parse_request(&HttpMethod::GET, "/server/0000000000000000000000000000000000000000000000000000000000000001/a199fb39e11eefb61c78a4074a53c0d4424600a3e74aad4fb9d93a26c30d067e1d4d29936de0c73f19827394a1dd049480a0d581aee7ae7546968da7d3d1c2fd01", Default::default()),
			Request::GetServerKey(H256::from_str("0000000000000000000000000000000000000000000000000000000000000001").unwrap(),
				"a199fb39e11eefb61c78a4074a53c0d4424600a3e74aad4fb9d93a26c30d067e1d4d29936de0c73f19827394a1dd049480a0d581aee7ae7546968da7d3d1c2fd01".parse().unwrap()));
		// GET		/{server_key_id}/{signature}										=> get document key
		assert_eq!(parse_request(&HttpMethod::GET, "/0000000000000000000000000000000000000000000000000000000000000001/a199fb39e11eefb61c78a4074a53c0d4424600a3e74aad4fb9d93a26c30d067e1d4d29936de0c73f19827394a1dd049480a0d581aee7ae7546968da7d3d1c2fd01", Default::default()),
			Request::GetDocumentKey(H256::from_str("0000000000000000000000000000000000000000000000000000000000000001").unwrap(),
				"a199fb39e11eefb61c78a4074a53c0d4424600a3e74aad4fb9d93a26c30d067e1d4d29936de0c73f19827394a1dd049480a0d581aee7ae7546968da7d3d1c2fd01".parse().unwrap()));
		assert_eq!(parse_request(&HttpMethod::GET, "/%30000000000000000000000000000000000000000000000000000000000000001/a199fb39e11eefb61c78a4074a53c0d4424600a3e74aad4fb9d93a26c30d067e1d4d29936de0c73f19827394a1dd049480a0d581aee7ae7546968da7d3d1c2fd01", Default::default()),
			Request::GetDocumentKey(H256::from_str("0000000000000000000000000000000000000000000000000000000000000001").unwrap(),
				"a199fb39e11eefb61c78a4074a53c0d4424600a3e74aad4fb9d93a26c30d067e1d4d29936de0c73f19827394a1dd049480a0d581aee7ae7546968da7d3d1c2fd01".parse().unwrap()));
		// GET		/shadow/{server_key_id}/{signature}									=> get document key shadow
		assert_eq!(parse_request(&HttpMethod::GET, "/shadow/0000000000000000000000000000000000000000000000000000000000000001/a199fb39e11eefb61c78a4074a53c0d4424600a3e74aad4fb9d93a26c30d067e1d4d29936de0c73f19827394a1dd049480a0d581aee7ae7546968da7d3d1c2fd01", Default::default()),
			Request::GetDocumentKeyShadow(H256::from_str("0000000000000000000000000000000000000000000000000000000000000001").unwrap(),
				"a199fb39e11eefb61c78a4074a53c0d4424600a3e74aad4fb9d93a26c30d067e1d4d29936de0c73f19827394a1dd049480a0d581aee7ae7546968da7d3d1c2fd01".parse().unwrap()));
		// GET		/schnorr/{server_key_id}/{signature}/{message_hash}					=> schnorr-sign message with server key
		assert_eq!(parse_request(&HttpMethod::GET, "/schnorr/0000000000000000000000000000000000000000000000000000000000000001/a199fb39e11eefb61c78a4074a53c0d4424600a3e74aad4fb9d93a26c30d067e1d4d29936de0c73f19827394a1dd049480a0d581aee7ae7546968da7d3d1c2fd01/281b6bf43cb86d0dc7b98e1b7def4a80f3ce16d28d2308f934f116767306f06c", Default::default()),
			Request::SchnorrSignMessage(H256::from_str("0000000000000000000000000000000000000000000000000000000000000001").unwrap(),
				"a199fb39e11eefb61c78a4074a53c0d4424600a3e74aad4fb9d93a26c30d067e1d4d29936de0c73f19827394a1dd049480a0d581aee7ae7546968da7d3d1c2fd01".parse().unwrap(),
				"281b6bf43cb86d0dc7b98e1b7def4a80f3ce16d28d2308f934f116767306f06c".parse().unwrap()));
		// GET		/ecdsa/{server_key_id}/{signature}/{message_hash}					=> ecdsa-sign message with server key
		assert_eq!(parse_request(&HttpMethod::GET, "/ecdsa/0000000000000000000000000000000000000000000000000000000000000001/a199fb39e11eefb61c78a4074a53c0d4424600a3e74aad4fb9d93a26c30d067e1d4d29936de0c73f19827394a1dd049480a0d581aee7ae7546968da7d3d1c2fd01/281b6bf43cb86d0dc7b98e1b7def4a80f3ce16d28d2308f934f116767306f06c", Default::default()),
			Request::EcdsaSignMessage(H256::from_str("0000000000000000000000000000000000000000000000000000000000000001").unwrap(),
				"a199fb39e11eefb61c78a4074a53c0d4424600a3e74aad4fb9d93a26c30d067e1d4d29936de0c73f19827394a1dd049480a0d581aee7ae7546968da7d3d1c2fd01".parse().unwrap(),
				"281b6bf43cb86d0dc7b98e1b7def4a80f3ce16d28d2308f934f116767306f06c".parse().unwrap()));
		// POST		/admin/servers_set_change/{old_set_signature}/{new_set_signature} + body
		let node1: Public = "843645726384530ffb0c52f175278143b5a93959af7864460f5a4fec9afd1450cfb8aef63dec90657f43f55b13e0a73c7524d4e9a13c051b4e5f1e53f39ecd91".parse().unwrap();
		let node2: Public = "07230e34ebfe41337d3ed53b186b3861751f2401ee74b988bba55694e2a6f60c757677e194be2e53c3523cc8548694e636e6acb35c4e8fdc5e29d28679b9b2f3".parse().unwrap();
		let nodes = vec![node1, node2].into_iter().collect();
		assert_eq!(parse_request(&HttpMethod::POST, "/admin/servers_set_change/a199fb39e11eefb61c78a4074a53c0d4424600a3e74aad4fb9d93a26c30d067e1d4d29936de0c73f19827394a1dd049480a0d581aee7ae7546968da7d3d1c2fd01/b199fb39e11eefb61c78a4074a53c0d4424600a3e74aad4fb9d93a26c30d067e1d4d29936de0c73f19827394a1dd049480a0d581aee7ae7546968da7d3d1c2fd01",
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
		assert_eq!(parse_request(&HttpMethod::GET, "", Default::default()), Request::Invalid);
		assert_eq!(parse_request(&HttpMethod::GET, "/shadow", Default::default()), Request::Invalid);
		assert_eq!(parse_request(&HttpMethod::GET, "///2", Default::default()), Request::Invalid);
		assert_eq!(parse_request(&HttpMethod::GET, "/shadow///2", Default::default()), Request::Invalid);
		assert_eq!(parse_request(&HttpMethod::GET, "/0000000000000000000000000000000000000000000000000000000000000001", Default::default()), Request::Invalid);
		assert_eq!(parse_request(&HttpMethod::GET, "/0000000000000000000000000000000000000000000000000000000000000001/", Default::default()), Request::Invalid);
		assert_eq!(parse_request(&HttpMethod::GET, "/a/b", Default::default()), Request::Invalid);
		assert_eq!(parse_request(&HttpMethod::GET, "/schnorr/0000000000000000000000000000000000000000000000000000000000000001/a199fb39e11eefb61c78a4074a53c0d4424600a3e74aad4fb9d93a26c30d067e1d4d29936de0c73f19827394a1dd049480a0d581aee7ae7546968da7d3d1c2fd01/0000000000000000000000000000000000000000000000000000000000000002/0000000000000000000000000000000000000000000000000000000000000002", Default::default()), Request::Invalid);
		assert_eq!(parse_request(&HttpMethod::GET, "/ecdsa/0000000000000000000000000000000000000000000000000000000000000001/a199fb39e11eefb61c78a4074a53c0d4424600a3e74aad4fb9d93a26c30d067e1d4d29936de0c73f19827394a1dd049480a0d581aee7ae7546968da7d3d1c2fd01/0000000000000000000000000000000000000000000000000000000000000002/0000000000000000000000000000000000000000000000000000000000000002", Default::default()), Request::Invalid);
		assert_eq!(parse_request(&HttpMethod::POST, "/admin/servers_set_change/xxx/yyy",
			&r#"["0x843645726384530ffb0c52f175278143b5a93959af7864460f5a4fec9afd1450cfb8aef63dec90657f43f55b13e0a73c7524d4e9a13c051b4e5f1e53f39ecd91",
				"0x07230e34ebfe41337d3ed53b186b3861751f2401ee74b988bba55694e2a6f60c757677e194be2e53c3523cc8548694e636e6acb35c4e8fdc5e29d28679b9b2f3"]"#.as_bytes()),
			Request::Invalid);
		assert_eq!(parse_request(&HttpMethod::POST, "/admin/servers_set_change/a199fb39e11eefb61c78a4074a53c0d4424600a3e74aad4fb9d93a26c30d067e1d4d29936de0c73f19827394a1dd049480a0d581aee7ae7546968da7d3d1c2fd01/a199fb39e11eefb61c78a4074a53c0d4424600a3e74aad4fb9d93a26c30d067e1d4d29936de0c73f19827394a1dd049480a0d581aee7ae7546968da7d3d1c2fd01", "".as_bytes()),
			Request::Invalid);
	}
}
