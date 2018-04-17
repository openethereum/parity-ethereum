// Copyright 2018 Parity Technologies (UK) Ltd.
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

extern crate parity;

use std::os::raw::{c_char, c_void, c_int};
use std::panic;
use std::ptr;
use std::slice;

#[repr(C)]
pub struct ParityParams {
	pub configuration: *mut c_void,
	pub on_client_restart_cb: Option<extern "C" fn(*mut c_void, *const c_char, usize)>,
	pub on_client_restart_cb_custom: *mut c_void,
}

#[no_mangle]
pub extern fn parity_config_from_cli(args: *const *const c_char, args_lens: *const usize, len: usize, output: *mut *mut c_void) -> c_int {
	unsafe {
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

			match parity::Configuration::parse_cli(&args) {
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
}

#[no_mangle]
pub extern fn parity_config_destroy(cfg: *mut c_void) {
	unsafe {
		let _ = panic::catch_unwind(|| {
			let _cfg = Box::from_raw(cfg as *mut parity::Configuration);
		});
	}
}

#[no_mangle]
pub extern fn parity_start(cfg: *const ParityParams, output: *mut *mut c_void) -> c_int {
	unsafe {
		panic::catch_unwind(|| {
			*output = ptr::null_mut();
			let cfg: &ParityParams = &*cfg;

			let config = Box::from_raw(cfg.configuration as *mut parity::Configuration);

			let on_client_restart_cb = {
				struct Cb(Option<extern "C" fn(*mut c_void, *const c_char, usize)>, *mut c_void);
				unsafe impl Send for Cb {}
				unsafe impl Sync for Cb {}
				impl Cb {
					fn call(&self, new_chain: String) {
						if let Some(ref cb) = self.0 {
							cb(self.1, new_chain.as_bytes().as_ptr() as *const _, new_chain.len())
						}
					}
				}
				let cb = Cb(cfg.on_client_restart_cb, cfg.on_client_restart_cb_custom);
				move |new_chain: String| { cb.call(new_chain); }
			};

			let action = match parity::start(*config, on_client_restart_cb, || {}) {
				Ok(action) => action,
				Err(_) => return 1,
			};

			match action {
				parity::ExecutionAction::Instant(Some(s)) => { println!("{}", s); 0 },
				parity::ExecutionAction::Instant(None) => 0,
				parity::ExecutionAction::Running(client) => {
					*output = Box::into_raw(Box::<parity::RunningClient>::new(client)) as *mut c_void;
					0
				}
			}
		}).unwrap_or(1)
	}
}

#[no_mangle]
pub extern fn parity_destroy(client: *mut c_void) {
	unsafe {
		let _ = panic::catch_unwind(|| {
			let client = Box::from_raw(client as *mut parity::RunningClient);
			client.shutdown();
		});
	}
}
