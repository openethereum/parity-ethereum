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

type CCallback = Option<extern "C" fn(*mut c_void, *const c_char, usize)>;
type CheckedQuery<'a> = (&'a RunningClient, &'static str);

pub mod error {
	pub const EMPTY: &str = r#"{"jsonrpc":"2.0","result":"null","id":1}"#;
	pub const TIMEOUT: &str = r#"{"jsonrpc":"2.0","result":"timeout","id":1}"#;
	pub const SUBSCRIBE: &str = r#"{"jsonrpc":"2.0","result":"subcribe_fail","id":1}"#;
}

#[repr(C)]
pub struct ParityParams {
	pub configuration: *mut c_void,
	pub on_client_restart_cb: CCallback,
	pub on_client_restart_cb_custom: *mut c_void,
	pub logger: *mut c_void
}

/// Trait representing a callback that passes a string
pub(crate) trait Callback: Send + Sync {
	fn call(&self, msg: &str);
}

// Internal structure for handling callbacks that get passed a string.
struct CallbackStr {
	user_data: *mut c_void,
	function: CCallback,
}

unsafe impl Send for CallbackStr {}
unsafe impl Sync for CallbackStr {}
impl Callback for CallbackStr {
	fn call(&self, msg: &str) {
		if let Some(ref cb) = self.function {
			let cstr = CString::new(msg).expect("valid string with no nul bytes in the middle; qed").into_raw();
			cb(self.user_data, cstr, msg.len())
		}
	}
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
		let logger = Arc::from_raw(cfg.logger as *mut parity_ethereum::RotatingLogger);
		let config = Box::from_raw(cfg.configuration as *mut parity_ethereum::Configuration);

		let on_client_restart_cb = {
			let cb = CallbackStr {
				user_data: cfg.on_client_restart_cb_custom,
				function: cfg.on_client_restart_cb,
			};
			move |new_chain: String| { cb.call(&new_chain); }
		};

		let action = match parity_ethereum::start(*config, logger, on_client_restart_cb, || {}) {
			Ok(action) => action,
			Err(_) => return 1,
		};

		match action {
			parity_ethereum::ExecutionAction::Instant(Some(s)) => { println!("{}", s); 0 },
			parity_ethereum::ExecutionAction::Instant(None) => 0,
			parity_ethereum::ExecutionAction::Running(client) => {
				*output = Box::into_raw(Box::new(client)) as *mut c_void;
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

#[no_mangle]
pub unsafe extern fn parity_rpc(
	client: *const c_void,
	query: *const c_char,
	len: usize,
	timeout_ms: usize,
	callback: CCallback,
	user_data: *mut c_void,
) -> c_int {
	panic::catch_unwind(|| {
		if let Some((client, query)) = parity_rpc_query_checker(client, query, len) {
			let callback = Arc::new(CallbackStr {user_data, function: callback} );
			parity_rpc_worker(client, query, callback, timeout_ms as u64);
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
	callback: CCallback,
	user_data: *mut c_void,
) -> *const c_void {
	panic::catch_unwind(|| {
		if let Some((client, query)) = parity_rpc_query_checker(client, query, len) {
			let callback = Arc::new(CallbackStr { user_data, function: callback});
			parity_ws_worker(client, query, callback)
		} else {
			ptr::null()
		}
	})
	.unwrap_or(ptr::null())
}

#[no_mangle]
pub unsafe extern fn parity_unsubscribe_ws(session: *const c_void) {
	let _ = panic::catch_unwind(|| {
		let _session = Arc::from_raw(session as *const PubSubSession);
	});
}

#[no_mangle]
pub extern fn parity_set_panic_hook(callback: CCallback, param: *mut c_void) {
	let cb = CallbackStr {user_data: param, function: callback};
	panic_hook::set_with(move |panic_msg| {
		cb.call(panic_msg);
	});
}

#[no_mangle]
pub unsafe extern fn parity_set_logger(
	logger_mode: *const u8,
	logger_mode_len: usize,
	log_file: *const u8,
	log_file_len: usize,
	logger: *mut *mut c_void) {

	let mut logger_cfg = parity_ethereum::LoggerConfig::default();
	logger_cfg.mode = String::from_utf8(slice::from_raw_parts(logger_mode, logger_mode_len).to_owned()).ok();

	// Make sure an empty string is not constructed as file name (to prevent panic)
	if log_file_len != 0 && !log_file.is_null() {
		logger_cfg.file = String::from_utf8(slice::from_raw_parts(log_file, log_file_len).to_owned()).ok();
	}

	*logger = Arc::into_raw(parity_ethereum::setup_log(&logger_cfg).expect("Logger initialized only once; qed")) as *mut _;
}

// WebSocket event loop
fn parity_ws_worker(client: &RunningClient, query: &str, callback: Arc<dyn Callback>) -> *const c_void {
	let (tx, mut rx) = mpsc::channel(1);
	let session = Arc::new(PubSubSession::new(tx));
	let query_future = client.rpc_query(query, Some(session.clone()));
	let weak_session = Arc::downgrade(&session);
	let _handle = thread::Builder::new()
		.name("ws-subscriber".into())
		.spawn(move || {
			// Wait for subscription ID
			// Note this may block forever and be can't destroyed using the session object
			// However, this will likely timeout or be catched the RPC layer
			if let Ok(Some(response)) = query_future.wait() {
				callback.call(&response);
			} else {
				callback.call(error::SUBSCRIBE);
				return;
			}

			while weak_session.upgrade().map_or(0, |session| Arc::strong_count(&session)) > 1 {
				for response in rx.by_ref().wait() {
					if let Ok(r) = response {
						callback.call(&r);
					}
				}
			}
		})
		.expect("rpc-subscriber thread shouldn't fail; qed");
	Arc::into_raw(session) as *const c_void
}

// RPC event loop that runs for at most `timeout_ms`
fn parity_rpc_worker(client: &RunningClient, query: &str, callback: Arc<dyn Callback>, timeout_ms: u64) {
	let cb = callback.clone();
	let query = client.rpc_query(query, None).map(move |response| {
		let response = response.unwrap_or_else(|| error::EMPTY.to_string());
		callback.call(&response);
	});

	let _handle = thread::Builder::new()
		.name("rpc_query".to_string())
		.spawn(move || {
			let mut current_thread = CurrentThread::new();
			current_thread.spawn(query);
			let _ = current_thread
				.run_timeout(Duration::from_millis(timeout_ms))
				.map_err(|_e| {
					cb.call(error::TIMEOUT);
				});
		})
		.expect("rpc-query thread shouldn't fail; qed");
}

unsafe fn parity_rpc_query_checker<'a>(client: *const c_void, query: *const c_char, len: usize)
	-> Option<CheckedQuery<'a>>
{
	let query_str = str::from_utf8(slice::from_raw_parts(query as *const u8, len)).ok()?;
	let client: &RunningClient = &*(client as *const RunningClient);
	Some((client, query_str))
}
