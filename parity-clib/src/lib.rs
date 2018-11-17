// Copyright 2015-2018 Parity Technologies (UK) Ltd.
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

//! Note that all the structs and functions here are documented in `parity.h`, to avoid
//! duplicating documentation.

extern crate futures;
extern crate panic_hook;
extern crate parity_ethereum;
extern crate tokio;
extern crate tokio_current_thread;

#[cfg(feature = "jni")]
extern crate jni;

#[cfg(feature = "jni")]
mod java;

use std::ffi::CString;
use std::os::raw::{c_char, c_void, c_int};
use std::{panic, ptr, slice, str, thread};
use std::sync::Arc;
use std::time::Duration;

use futures::{Future, Stream};
use futures::sync::mpsc;
use parity_ethereum::{PubSubSession, RunningClient};
use tokio_current_thread::CurrentThread;

type Callback = Option<extern "C" fn(*mut c_void, *const c_char, usize)>;
type CheckedCallback = Arc<extern "C" fn(*mut c_void, *const c_char, usize)>;
type CheckedQuery<'a> = (&'a RunningClient, &'static str, CheckedCallback);

pub mod error {
	pub const EMPTY: &str = r#"{"jsonrpc":"2.0","result":"null","id":1}"#;
	pub const TIMEOUT: &str = r#"{"jsonrpc":"2.0","result":"timeout","id":1}"#;
	pub const SUBSCRIBE: &str = r#"{"jsonrpc":"2.0","result":"subcribe_fail","id":1}"#;
}


#[repr(C)]
pub struct ParityParams {
	pub configuration: *mut c_void,
	pub on_client_restart_cb: Callback,
	pub on_client_restart_cb_custom: *mut c_void,
}

#[no_mangle]
pub unsafe extern fn parity_config_from_cli(
	args: *const *const c_char,
	args_lens: *const usize,
	len: usize,
	output: *mut *mut c_void
) -> c_int {
	panic::catch_unwind(|| {
		*output = ptr::null_mut();

		let args = {
			let arg_ptrs = slice::from_raw_parts(args, len);
			let arg_lens = slice::from_raw_parts(args_lens, len);

			let mut args = Vec::with_capacity(len + 1);
			args.push("parity".to_owned());

			for (&arg, &len) in arg_ptrs.iter().zip(arg_lens.iter()) {
				let string = slice::from_raw_parts(arg as *const u8, len);
				match String::from_utf8(string.to_owned()) {
					Ok(a) => args.push(a),
					Err(_) => return 1,
				};
			}
			args
		};

		match parity_ethereum::Configuration::parse_cli(&args) {
			Ok(mut cfg) => {
				// Always disable the auto-updater when used as a library.
				cfg.args.arg_auto_update = "none".to_owned();

				let cfg = Box::into_raw(Box::new(cfg));
				*output = cfg as *mut _;
				0
			},
			Err(_) => {
				1
			},
		}
	}).unwrap_or(1)
}

#[no_mangle]
pub unsafe extern fn parity_config_destroy(cfg: *mut c_void) {
	let _ = panic::catch_unwind(|| {
		let _cfg = Box::from_raw(cfg as *mut parity_ethereum::Configuration);
	});
}

#[no_mangle]
pub unsafe extern fn parity_start(cfg: *const ParityParams, output: *mut *mut c_void) -> c_int {
	panic::catch_unwind(|| {
		*output = ptr::null_mut();
		let cfg: &ParityParams = &*cfg;

		let config = Box::from_raw(cfg.configuration as *mut parity_ethereum::Configuration);

		let on_client_restart_cb = {
			let cb = CallbackStr(cfg.on_client_restart_cb, cfg.on_client_restart_cb_custom);
			move |new_chain: String| { cb.call(&new_chain); }
		};

		let action = match parity_ethereum::start(*config, on_client_restart_cb, || {}) {
			Ok(action) => action,
			Err(_) => return 1,
		};

		match action {
			parity_ethereum::ExecutionAction::Instant(Some(s)) => { println!("{}", s); 0 },
			parity_ethereum::ExecutionAction::Instant(None) => 0,
			parity_ethereum::ExecutionAction::Running(client) => {
				*output = Box::into_raw(Box::<parity_ethereum::RunningClient>::new(client)) as *mut c_void;
				0
			}
		}
	}).unwrap_or(1)
}

#[no_mangle]
pub unsafe extern fn parity_destroy(client: *mut c_void) {
	let _ = panic::catch_unwind(|| {
		let client = Box::from_raw(client as *mut RunningClient);
		client.shutdown();
	});
}


unsafe fn parity_rpc_query_checker<'a>(client: *const c_void, query: *const c_char, len: usize, callback: Callback)
	-> Option<CheckedQuery<'a>>
{
	let callback = callback?;
	let query_str = {
			let string = slice::from_raw_parts(query as *const u8, len);
			str::from_utf8(string).ok()?
	};
	let client: &RunningClient = &*(client as *const RunningClient);

	Some((client, query_str, Arc::new(callback)))
}

fn parity_rpc_dispatcher(
	client: &RunningClient,
	query: &str,
	session: Option<Arc<PubSubSession>>,
	callback: CheckedCallback,
	timeout_ms: usize,
	name: &str) {

	let cb = callback.clone();
	let client = client as &RunningClient;
	let query = client.rpc_query(query, session).map(move |response| {
		let (cstr, len) = match response {
			Some(response) => to_cstring(response.as_bytes()),
			None => to_cstring(error::EMPTY.as_bytes()),
		};
		cb(ptr::null_mut(), cstr, len);
		()
	});

	let _handle = thread::Builder::new()
		.name(name.into())
		.spawn(move || {
			let mut current_thread = CurrentThread::new();
			current_thread.spawn(query);
			let _ = current_thread.run_timeout(Duration::from_millis(timeout_ms as u64))
				.map_err(|_e| {
					let (cstr, len) = to_cstring(error::TIMEOUT.as_bytes());
					callback(ptr::null_mut(), cstr, len);
				});
			})
		.expect("rpc-query thread shouldn't fail; qed");
}

#[no_mangle]
pub unsafe extern fn parity_rpc(
	client: *const c_void,
	query: *const c_char,
	len: usize,
	timeout_ms: usize,
	callback: Callback,
) -> c_int {
	panic::catch_unwind(|| {
		if let Some((client, query, callback)) = parity_rpc_query_checker(client, query, len, callback) {
			parity_rpc_dispatcher(client, query, None, callback, timeout_ms, "rpc-query");
			0
		} else {
			1
		}
	}).unwrap_or(1)
}


#[no_mangle]
pub unsafe extern fn parity_subscribe_ws(
	client: *const c_void,
	query: *const c_char,
	len: usize,
	callback: Callback,
) -> *const c_void {

	panic::catch_unwind(|| {
		if let Some((client, query, callback)) = parity_rpc_query_checker(client, query, len, callback) {
			let (tx, mut rx) = mpsc::channel(1);
			let session = Arc::new(PubSubSession::new(tx));
			let ffi_session = session.clone();
			let query = client.rpc_query(query, Some(session.clone()));

			let _handle = thread::Builder::new()
				.name("ws-subscriber".into())
				.spawn(move || {

					// wait for subscription ID
					match query.wait() {
						Ok(Some(response)) => {
							let (cstr, len) = to_cstring(response.as_bytes());
							callback(ptr::null_mut(), cstr, len);
						}
						// subscription failed return error and exit
						_ => {
							let (cstr, len) = to_cstring(error::SUBSCRIBE.as_bytes());
							callback(ptr::null_mut(), cstr, len);
							return;
						}
					};

					while Arc::strong_count(&session) > 1 {
						for response in rx.by_ref().wait() {
							if let Ok(r) = response {
								let (cstring, len) = to_cstring(r.as_bytes());
								callback(ptr::null_mut(), cstring, len);
						}
					}
				}
			})
			.expect("rpc-subscriber thread shouldn't fail; qed");

			Arc::into_raw(ffi_session) as *const c_void
		} else {
			ptr::null()
		}
	})
	.unwrap_or(ptr::null())
}

#[no_mangle]
pub unsafe extern fn parity_unsubscribe_ws(
	client: *const c_void,
	session: *const c_void,
	query: *const c_char,
	len: usize,
	timeout_ms: usize,
	callback: Callback,
) -> c_int {
	panic::catch_unwind(|| {
		if let Some((client, query, callback)) = parity_rpc_query_checker(client, query, len, callback) {
			let session = Some(Arc::from_raw(session as *const PubSubSession));
			parity_rpc_dispatcher(client, query, session, callback, timeout_ms, "ws-unsubscribe");
			0
		} else {
			1
		}
	}).unwrap_or(1)
}

#[no_mangle]
pub unsafe extern fn parity_set_panic_hook(callback: Callback, param: *mut c_void) {
	let cb = CallbackStr(callback, param);
	panic_hook::set_with(move |panic_msg| {
		cb.call(panic_msg);
	});
}

// Internal structure for handling callbacks that get passed a string.
struct CallbackStr(Callback, *mut c_void);
unsafe impl Send for CallbackStr {}
unsafe impl Sync for CallbackStr {}
impl CallbackStr {
	fn call(&self, new_chain: &str) {
		if let Some(ref cb) = self.0 {
			cb(self.1, new_chain.as_bytes().as_ptr() as *const _, new_chain.len())
		}
	}
}

fn to_cstring(response: &[u8]) -> (*const c_char, usize) {
	let len = response.len();
	let cstr = CString::new(response).expect("valid string with no null bytes in the middle; qed").into_raw();
	(cstr, len)
}
