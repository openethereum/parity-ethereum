use std::{mem, ptr};
use std::os::raw::c_void;

use {Callback, parity_config_from_cli, parity_destroy, parity_start, parity_rpc, ParityParams};
use jni::{JNIEnv, objects::JClass, objects::JString, sys::jlong, sys::jobjectArray};

#[no_mangle]
pub unsafe extern "system" fn Java_io_parity_ethereum_Parity_configFromCli(env: JNIEnv, _: JClass, cli: jobjectArray) -> jlong {
	let cli_len = env.get_array_length(cli).expect("invalid Java bindings");

	let mut jni_strings = Vec::with_capacity(cli_len as usize);
	let mut opts = Vec::with_capacity(cli_len as usize);
	let mut opts_lens = Vec::with_capacity(cli_len as usize);

	for n in 0 .. cli_len {
		let elem = env.get_object_array_element(cli, n).expect("invalid Java bindings");
		let elem_str: JString = elem.into();
		match env.get_string(elem_str) {
			Ok(s) => {
				opts.push(s.as_ptr());
				opts_lens.push(s.to_bytes().len());
				jni_strings.push(s);
			},
			Err(err) => {
				let _ = env.throw_new("java/lang/Exception", err.to_string());
				return 0
			}
		};
	}

	let mut out = ptr::null_mut();
	match parity_config_from_cli(opts.as_ptr(), opts_lens.as_ptr(), cli_len as usize, &mut out) {
		0 => out as usize as jlong,
		_ => {
			let _ = env.throw_new("java/lang/Exception", "failed to create config object");
			0
		},
	}
}

#[no_mangle]
pub unsafe extern "system" fn Java_io_parity_ethereum_Parity_build(env: JNIEnv, _: JClass, config: jlong) -> jlong {
	let params = ParityParams {
		configuration: config as usize as *mut c_void,
		.. mem::zeroed()
	};

	let mut out = ptr::null_mut();
	match parity_start(&params, &mut out) {
		0 => out as usize as jlong,
		_ => {
			let _ = env.throw_new("java/lang/Exception", "failed to start Parity");
			0
		},
	}
}

#[no_mangle]
pub unsafe extern "system" fn Java_io_parity_ethereum_Parity_destroy(_env: JNIEnv, _: JClass, parity: jlong) {
	let parity = parity as usize as *mut c_void;
	parity_destroy(parity);
}

#[no_mangle]
pub unsafe extern "system" fn Java_io_parity_ethereum_Parity_rpcQueryNative<'a>(
	env: JNIEnv<'a>,
	_: JClass,
	parity:jlong,
	rpc: JString,
	callback: Callback,
)
{
	let parity = parity as usize as *mut c_void;

	let rpc = match env.get_string(rpc) {
		Ok(s) => s,
		Err(err) => {
			let _ = env.throw_new("java/lang/Exception", err.to_string());
			return;
		},
	};

	//FIXME: provide "callback" in java fashion
	match parity_rpc(parity, rpc.as_ptr(), rpc.to_bytes().len(), callback) {
		0 => (),
		_ => {
			let _ = env.throw_new("java/lang/Exception", "failed to perform RPC query");
			return;
		},
	}
}
