extern crate jsonrpc_core;

use std::fmt::{Debug, Formatter, Error as FmtError};
use std::io::{BufReader, BufRead};
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::collections::BTreeMap;
use std::thread;
use std::time;

use std::path::PathBuf;
use util::Hashable;
use url::Url;
use std::fs::File;

use ws::{self,
		 Request,
		 Handler,
		 Sender,
		 Handshake,
		 Error as WsError,
		 ErrorKind as WsErrorKind,
		 Message,
		 Result as WsResult};

use serde::Deserialize;
use serde_json::{self as json,
				 Value as JsonValue,
				 Error as JsonError};

use futures::{BoxFuture, Canceled, Complete, Future, oneshot, done};

use jsonrpc_core::{Id, Version, Params, Error as JsonRpcError};
use jsonrpc_core::request::MethodCall;
use jsonrpc_core::response::{SyncOutput, Success, Failure};

/// The actual websocket connection handler, passed into the
/// event loop of ws-rs
struct RpcHandler {
	pending: Pending,
	// Option is used here as temporary storage until connection
	// is setup and the values are moved into the new `Rpc`
	complete: Option<Complete<Result<Rpc, RpcError>>>,
	auth_code: String,
	out: Option<Sender>,
}

impl RpcHandler {
	fn new(out: Sender,
		   auth_code: String,
		   complete: Complete<Result<Rpc, RpcError>>)
		   -> Self {
		RpcHandler {
			out: Some(out),
			auth_code: auth_code,
			pending: Pending::new(),
			complete: Some(complete),
		}
	}
}

impl Handler for RpcHandler {
    fn build_request(&mut self, url: &Url) -> WsResult<Request> {
		match Request::from_url(url) {
			Ok(mut r) => {
				let timestamp = try!(time::UNIX_EPOCH.elapsed().map_err(|err| {
					WsError::new(WsErrorKind::Internal, format!("{}", err))
				}));
				let secs = timestamp.as_secs();
				let hashed = format!("{}:{}", self.auth_code, secs).sha3();
				let proto = format!("{:?}_{}", hashed, secs);
				r.add_protocol(&proto);
				Ok(r)
			},
			Err(e) => Err(WsError::new(WsErrorKind::Internal, format!("{}", e))),
		}
    }
    fn on_error(&mut self, err: WsError) {
		match self.complete.take() {
			Some(c) => c.complete(Err(RpcError::WsError(err))),
			None => println!("unexpected error: {}", err),
		}
    }
    fn on_open(&mut self, _: Handshake) -> WsResult<()> {
		match (self.complete.take(), self.out.take()) {
			(Some(c), Some(out)) => {
				c.complete(Ok(Rpc {
					out: out,
					counter: AtomicUsize::new(0),
					pending: self.pending.clone(),
				}));
				Ok(())
			},
			_ => {
				Err(WsError::new(WsErrorKind::Internal, format!("on_open called twice")))
			}
		}
	}
    fn on_message(&mut self, msg: Message) -> WsResult<()> {
		let ret: Result<JsonValue, JsonRpcError>;
		let response_id;
		let string = &msg.to_string();
		match json::from_str::<SyncOutput>(&string) {
			Ok(SyncOutput::Success(Success { result, id: Id::Num(id), .. })) => {
				ret = Ok(result);
				response_id = id as usize;
			}
			Ok(SyncOutput::Failure(Failure { error, id: Id::Num(id), .. })) => {
				ret = Err(error);
				response_id = id as usize;
			}
			Err(e) => {
				warn!(target: "rpc-client", "recieved invalid message: {}\n {:?}", string, e);
				return Ok(())
			},
			_ => {
				warn!(target: "rpc-client", "recieved invalid message: {}", string);
				return Ok(())
			}
		}

		match self.pending.remove(response_id) {
			Some(c) => c.complete(ret.map_err(|err| {
				RpcError::JsonRpc(err)
			})),
			None => warn!(target: "rpc-client", "warning: unexpected id: {}", response_id),
		}
		Ok(())
    }
}

/// Keeping track of issued requests to be matched up with responses
#[derive(Clone)]
struct Pending(Arc<Mutex<BTreeMap<usize, Complete<Result<JsonValue, RpcError>>>>>);

impl Pending {
	fn new() -> Self {
		Pending(Arc::new(Mutex::new(BTreeMap::new())))
	}
	fn insert(&mut self, k: usize, v: Complete<Result<JsonValue, RpcError>>) {
		self.0.lock().expect("no panics in mutex guard").insert(k, v);
	}
	fn remove(&mut self, k: usize) -> Option<Complete<Result<JsonValue, RpcError>>> {
		self.0.lock().expect("no panics in mutex guard").remove(&k)
	}
}

fn get_authcode(path: &PathBuf) -> Result<String, RpcError> {
	match File::open(path) {
		Ok(fd) => match BufReader::new(fd).lines().next() {
			Some(Ok(code)) => Ok(code),
			_ => Err(RpcError::NoAuthCode),
		},
		Err(_) => Err(RpcError::NoAuthCode)
	}
}

/// The handle to the connection
pub struct Rpc {
	out: Sender,
	counter: AtomicUsize,
	pending: Pending,
}

impl Rpc {
	/// Blocking, returns a new initialized connection or RpcError
	pub fn new(url: &str, authpath: &PathBuf) -> Result<Self, RpcError> {
		let rpc = try!(Self::connect(url, authpath).map(|rpc| rpc).wait());
		rpc
	}
	/// Non-blocking, returns a future
	pub fn connect(url: &str, authpath: &PathBuf)
			   -> BoxFuture<Result<Self, RpcError>, Canceled> {
		let (c, p) = oneshot::<Result<Self, RpcError>>();
		match get_authcode(authpath) {
			Err(e) => return done(Ok(Err(e))).boxed(),
			Ok(code) => {
				let url = String::from(url);
				// The ws::connect takes a FnMut closure, which means c cannot be
				// moved into it, since it's consumed on complete.
				// Therefore we wrap it in an option and pick it out once.
				let mut once = Some(c);
				thread::spawn(move || {
					let conn = ws::connect(url, |out| {
						// this will panic if the closure is called twice,
						// which it should never be.
						let c = once.take().expect("connection closure called only once");
						RpcHandler::new(out, code.clone(), c)
					});
					match conn {
						Err(err) => {
							// since ws::connect is only called once, it cannot
							// both fail and succeed.
							let c = once.take().expect("connection closure called only once");
							c.complete(Err(RpcError::WsError(err)));
						},
						// c will complete on the `on_open` event in the Handler
						_ => ()
					}
				});
				p.boxed()
			}
		}
	}
	/// Non-blocking, returns a future of the request response
	pub fn request<T>(&mut self, method: &'static str, params: Vec<JsonValue>)
			   -> BoxFuture<Result<T, RpcError>, Canceled>
		where T: Deserialize + Send + Sized {

		let (c, p) = oneshot::<Result<JsonValue, RpcError>>();

		let id = self.counter.fetch_add(1, Ordering::Relaxed);
		self.pending.insert(id, c);

		let request = MethodCall {
			jsonrpc: Version::V2,
			method: method.to_owned(),
			params: Some(Params::Array(params)),
			id: Id::Num(id as u64),
		};

		let serialized = json::to_string(&request).expect("request is serializable");
		let _ = self.out.send(serialized);

		p.map(|result| {
			match result {
				Ok(json) => {
					let t: T = try!(json::from_value(json));
					Ok(t)
				},
				Err(err) => Err(err)
			}
		}).boxed()
	}
}

pub enum RpcError {
	WrongVersion(String),
	ParseError(JsonError),
	MalformedResponse(String),
	JsonRpc(JsonRpcError),
	WsError(WsError),
	Canceled(Canceled),
	UnexpectedId,
	NoAuthCode,
}

impl Debug for RpcError {
	fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
		match *self {
			RpcError::WrongVersion(ref s)
				=> write!(f, "Expected version 2.0, got {}", s),
			RpcError::ParseError(ref err)
				=> write!(f, "ParseError: {}", err),
			RpcError::MalformedResponse(ref s)
				=> write!(f, "Malformed response: {}", s),
			RpcError::JsonRpc(ref json)
				=> write!(f, "JsonRpc error: {:?}", json),
			RpcError::WsError(ref s)
				=> write!(f, "Websocket error: {}", s),
			RpcError::Canceled(ref s)
				=> write!(f, "Futures error: {:?}", s),
			RpcError::UnexpectedId
				=> write!(f, "Unexpected response id"),
			RpcError::NoAuthCode
				=> write!(f, "No authcodes available"),
		}
	}
}

impl From<JsonError> for RpcError {
	fn from(err: JsonError) -> RpcError {
		RpcError::ParseError(err)
	}
}

impl From<WsError> for RpcError {
	fn from(err: WsError) -> RpcError {
		RpcError::WsError(err)
	}
}

impl From<Canceled> for RpcError {
	fn from(err: Canceled) -> RpcError {
		RpcError::Canceled(err)
	}
}
