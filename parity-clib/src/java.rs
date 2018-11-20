use std::{mem, ptr};

use {Callback, parity_config_from_cli, parity_destroy, parity_start, parity_rpc, parity_subscribe_ws,
	parity_unsubscribe_ws, ParityParams};
use jni::{JNIEnv, objects::JClass, objects::JString, sys::jlong, sys::jobjectArray, sys::va_list};

#[no_mangle]
pub unsafe extern "system" fn Java_io_parity_ethereum_Parity_configFromCli(env: JNIEnv, _: JClass, cli: jobjectArray) -> jlong {
	let cli_len = env.get_array_length(cli).expect("invalid Java bindings");

	let mut jni_strings = Vec::with_capacity(cli_len as usize);
	let mut opts = Vec::with_capacity(cli_len as usize);
	let mut opts_lens = Vec::with_capacity(cli_len as usize);

	for n in 0..cli_len {
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
pub unsafe extern "system" fn Java_io_parity_ethereum_Parity_build(env: JNIEnv, _: JClass, config: va_list) -> jlong {
	let params = ParityParams {
		configuration: config,
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
pub unsafe extern "system" fn Java_io_parity_ethereum_Parity_destroy(_env: JNIEnv, _: JClass, parity: va_list) {
	parity_destroy(parity);
}

#[no_mangle]
pub unsafe extern "system" fn Java_io_parity_ethereum_Parity_rpcQueryNative<'a>(
	env: JNIEnv<'a>,
	_: JClass,
	parity: va_list,
	rpc: JString,
	timeout_ms: jlong,
	callback: Callback,
	user_data: va_list,
)
{
	let rpc = match env.get_string(rpc) {
		Ok(s) => s,
		Err(err) => {
			let _ = env.throw_new("java/lang/Exception", err.to_string());
			return;
		},
	};

	match parity_rpc(parity, rpc.as_ptr(), rpc.to_bytes().len(), timeout_ms as usize, callback, user_data) {
		0 => (),
		_ => {
			let _ = env.throw_new("java/lang/Exception", "failed to perform RPC query");
			return;
		},
	}
}

#[no_mangle]
pub unsafe extern "system" fn Java_io_parity_ethereum_Parity_subscribeWebSocket<'a>(
	env: JNIEnv<'a>,
	_: JClass,
	parity: va_list,
	rpc: JString,
	callback: Callback,
	user_data: va_list,
) -> va_list {
	let rpc = match env.get_string(rpc) {
		Ok(s) => s,
		Err(err) => {
			let _ = env.throw_new("java/lang/Exception", err.to_string());
			return ptr::null_mut();
		},
	};

	let result = parity_subscribe_ws(parity, rpc.as_ptr(), rpc.to_bytes().len(), callback, user_data) as va_list;
	if result.is_null() {
		let _ = env.throw_new("java/lang/Exception", "failed to subscribe to WebSocket");
	}
	result
}

#[no_mangle]
pub unsafe extern "system" fn Java_io_parity_ethereum_Parity_unsubscribeWebSocket<'a>(session: va_list) {
	parity_unsubscribe_ws(session);
}
