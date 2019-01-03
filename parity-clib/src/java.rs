use std::{mem, ptr};
use std::sync::Arc;
use std::time::Duration;
use std::thread;
use std::os::raw::c_void;

use {parity_config_from_cli, parity_destroy, parity_start, parity_unsubscribe_ws, ParityParams, error};

use futures::{Future, Stream};
use futures::sync::mpsc;
use jni::{JavaVM, JNIEnv};
use jni::objects::{JClass, JString, JObject, JValue, GlobalRef};
use jni::sys::{jlong, jobjectArray, va_list};
use tokio_current_thread::CurrentThread;
use parity_ethereum::{RunningClient, PubSubSession};

type CheckedQuery<'a> = (&'a RunningClient, String, JavaVM, GlobalRef);

// Creates a Java callback to a static method named `void callback(Object)`
struct Callback<'a> {
	jvm: JavaVM,
	callback: GlobalRef,
	method_name: &'a str,
	method_descriptor: &'a str,
}

unsafe impl<'a> Send for Callback<'a> {}
unsafe impl<'a> Sync for Callback<'a> {}
impl<'a> Callback<'a> {
	fn new(jvm: JavaVM, callback: GlobalRef) -> Self {
		Self {
			jvm,
			callback,
			method_name: "callback",
			method_descriptor: "(Ljava/lang/Object;)V",
		}
	}

	fn call(&self, msg: &str) {
		let env = self.jvm.attach_current_thread().expect("JavaVM should have an environment; qed");
		let java_str = env.new_string(msg.to_string()).expect("Rust String is valid JString; qed");
		let val = &[JValue::Object(JObject::from(java_str))];
		env.call_method(self.callback.as_obj(), self.method_name, self.method_descriptor, val).expect(
			"The callback must be an instance method and be named \"void callback(Object)\"; qed)");
	}
}

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
		0 => out as jlong,
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
		0 => out as jlong,
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

unsafe fn async_checker<'a>(client: va_list, rpc: JString, callback: JObject, env: &JNIEnv<'a>)
-> Result<CheckedQuery<'a>, String> {
	let query: String = env.get_string(rpc)
		.map(Into::into)
		.map_err(|e| e.to_string())?;

	let client: &RunningClient = &*(client as *const RunningClient);
	let jvm = env.get_java_vm().map_err(|e| e.to_string())?;
	let global_ref = env.new_global_ref(callback).map_err(|e| e.to_string())?;
	Ok((client, query, jvm, global_ref))
}

#[no_mangle]
pub unsafe extern "system" fn Java_io_parity_ethereum_Parity_rpcQueryNative(
	env: JNIEnv,
	_: JClass,
	parity: va_list,
	rpc: JString,
	timeout_ms: jlong,
	callback: JObject,
	)
{
	let _ = async_checker(parity, rpc, callback, &env)
		.map(|(client, query, jvm, global_ref)| {
			let callback = Arc::new(Callback::new(jvm, global_ref));
			let cb = callback.clone();
			let future = client.rpc_query(&query, None).map(move |response| {
				let response = response.unwrap_or_else(|| error::EMPTY.to_string());
				callback.call(&response);
			});

			let _handle = thread::Builder::new()
				.name("rpc_query".to_string())
				.spawn(move || {
					let mut current_thread = CurrentThread::new();
					current_thread.spawn(future);
					let _ = current_thread.run_timeout(Duration::from_millis(timeout_ms as u64))
						.map_err(|_e| {
							cb.call(error::TIMEOUT);
						});
				})
				.expect("rpc-query thread shouldn't fail; qed");
		})
		.map_err(|e| {
			let _ = env.throw_new("java/lang/Exception", e);
		});
}

#[no_mangle]
pub unsafe extern "system" fn Java_io_parity_ethereum_Parity_subscribeWebSocketNative(
	env: JNIEnv,
	_: JClass,
	parity: va_list,
	rpc: JString,
	callback: JObject,
	) -> va_list {

	async_checker(parity, rpc, callback, &env)
		.map(move |(client, query, jvm, global_ref)| {
			let callback = Arc::new(Callback::new(jvm, global_ref));
			let (tx, mut rx) = mpsc::channel(1);
			let session = Arc::new(PubSubSession::new(tx));
			let weak_session = Arc::downgrade(&session);
			let query_future = client.rpc_query(&query, Some(session.clone()));;

			let _handle = thread::Builder::new()
				.name("ws-subscriber".into())
				.spawn(move || {
					// Wait for subscription ID
					// Note this may block forever and can't be destroyed using the session object
					// However, this will likely timeout or be catched the RPC layer
					if let Ok(Some(response)) = query_future.wait() {
						callback.call(&response);
					} else {
						callback.call(error::SUBSCRIBE);
						return;
					};

					loop {
						for response in rx.by_ref().wait() {
							if let Ok(r) = response {
								callback.call(&r);
							}
						}

						let rc = weak_session.upgrade().map_or(0,|session| Arc::strong_count(&session));
						// No subscription left, then terminate
						if rc <= 1 {
							break;
						}
					}
				})
			.expect("rpc-subscriber thread shouldn't fail; qed");
			Arc::into_raw(session) as va_list
		})
		.unwrap_or_else(|e| {
			let _ = env.throw_new("java/lang/Exception", e);
			ptr::null_mut()
		})
}

#[no_mangle]
pub unsafe extern "system" fn Java_io_parity_ethereum_Parity_unsubscribeWebSocketNative(
	_: JNIEnv,
	_: JClass,
	session: va_list) {
	parity_unsubscribe_ws(session as *const c_void);
}
