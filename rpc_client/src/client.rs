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

use serde::Serialize;
use serde::Deserialize;
use serde::ser::Serializer;
use serde_json::{self as json,
				 Value as JsonValue,
				 Error as JsonError};

//use jsonrpc_core::

use futures::{BoxFuture, Canceled, Complete, Future, oneshot, done};

/// The actual websocket connection handler, passed into the
/// event loop of ws-rs
struct RpcHandler {
	pending: Pending,
	// Option is used here as
	// temporary storage until
	// connection is setup
	// and the values are moved into
	// the new `Rpc`
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
		match parse_response(&msg.to_string()) {
			(Some(id), response) => {
				match self.pending.remove(id) {
					Some(c) => c.complete(response),
					None => println!("warning: unexpected id: {}", id),
				}
			}
			(None, response) => println!("warning: error: {:?}, {}", response, msg.to_string()),
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
		self.0.lock().unwrap().insert(k, v);
	}
	fn remove(&mut self, k: usize) -> Option<Complete<Result<JsonValue, RpcError>>> {
		self.0.lock().unwrap().remove(&k)
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
				// The ws::connect takes a FnMut closure,
				// which means c cannot be moved into it,
				// since it's consumed on complete.
				// Therefore we wrap it in an option
				// and pick it out once.
				let mut once = Some(c);
				thread::spawn(move || {
					let conn = ws::connect(url, |out| {
						// this will panic if the closure
						// is called twice, which it should never
						// be.
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

		let serialized = json::to_string(&RpcRequest::new(id, method, params)).unwrap();
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


struct RpcRequest {
	method: &'static str,
	params: Vec<JsonValue>,
	id: usize,
}

impl RpcRequest {
	fn new(id: usize, method: &'static str, params: Vec<JsonValue>) -> Self {
		RpcRequest {
			method: method,
			id: id,
			params: params,
		}
	}
}

impl Serialize for RpcRequest {
    fn serialize<S>(&self, s: &mut S)
					-> Result<(), S::Error>
		where S: Serializer {
        let mut state = try!(s.serialize_struct("RpcRequest" , 3));
        try!(s.serialize_struct_elt(&mut state ,"jsonrpc", "2.0"));
        try!(s.serialize_struct_elt(&mut state ,"id" , &self.id));
        try!(s.serialize_struct_elt(&mut state ,"method" , &self.method));
        try!(s.serialize_struct_elt(&mut state ,"params" , &self.params));
        s.serialize_struct_end(state)
    }
}

pub enum RpcError {
	WrongVersion(String),
	ParseError(JsonError),
	MalformedResponse(String),
	Remote(String),
	WsError(WsError),
	Canceled(Canceled),
	UnexpectedId,
	NoAuthCode,
}

impl Debug for RpcError {
	fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
		match self {
			&RpcError::WrongVersion(ref s)
				=> write!(f, "Expected version 2.0, got {}", s),
			&RpcError::ParseError(ref err)
				=> write!(f, "ParseError: {}", err),
			&RpcError::MalformedResponse(ref s)
				=> write!(f, "Malformed response: {}", s),
			&RpcError::Remote(ref s)
				=> write!(f, "Remote error: {}", s),
			&RpcError::WsError(ref s)
				=> write!(f, "Websocket error: {}", s),
			&RpcError::Canceled(ref s)
				=> write!(f, "Futures error: {:?}", s),
			&RpcError::UnexpectedId
				=> write!(f, "Unexpected response id"),
			&RpcError::NoAuthCode
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

fn parse_response(s: &str) -> (Option<usize>, Result<JsonValue, RpcError>) {
	let mut json: JsonValue = match json::from_str(s) {
		Err(e) => return (None, Err(RpcError::ParseError(e))),
		Ok(json) => json,
	};

	let obj = match json.as_object_mut() {
		Some(o) => o,
		None => return
			(None,
			 Err(RpcError::MalformedResponse("Not a JSON object".to_string()))),
	};

	let id;
	match obj.get("id") {
		Some(&JsonValue::U64(u)) => {
			id = u as usize;
		},
		_ => return (None,
					 Err(RpcError::MalformedResponse("Missing id".to_string()))),
	}

	match obj.get("jsonrpc") {
		Some(&JsonValue::String(ref s)) => {
			if *s != "2.0".to_string() {
				return (Some(id),
						Err(RpcError::WrongVersion(s.clone())))
			}
		},
		_ => return
			(Some(id),
			 Err(RpcError::MalformedResponse("Not a jsonrpc object".to_string()))),
	}

	match obj.get("error") {
		Some(err) => return
			(Some(id),
			 Err(RpcError::Remote(format!("{}", err)))),
		None => (),
	};

	match obj.remove("result") {
		None => (Some(id),
				 Err(RpcError::MalformedResponse("No result".to_string()))),
		Some(result) => (Some(id),
						 Ok(result)),
	}
}
