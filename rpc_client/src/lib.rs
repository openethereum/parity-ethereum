pub mod client;
pub mod signer_client;

extern crate ethcore_util as util;
extern crate futures;
extern crate jsonrpc_core;
extern crate jsonrpc_ws_server as ws;
extern crate parity_rpc as rpc;
extern crate rand;
extern crate serde;
extern crate serde_json;
extern crate tempdir;
extern crate url;
extern crate hash;

#[macro_use]
extern crate log;

#[cfg(test)]
#[macro_use]
extern crate matches;


#[cfg(test)]
mod tests {

	use futures::Future;
	use std::path::PathBuf;
	use client::{Rpc, RpcError};
	use rpc;

	#[test]
	fn test_connection_refused() {
		let (_srv, port, mut authcodes) = rpc::tests::ws::serve();

		let _ = authcodes.generate_new();
		authcodes.to_file(&authcodes.path).unwrap();

		let connect = Rpc::connect(&format!("ws://127.0.0.1:{}", port - 1),
								   authcodes.path.as_path());

		let _ = connect.map(|conn| {
			assert!(matches!(&conn, &Err(RpcError::WsError(_))));
		}).wait();
	}

	#[test]
	fn test_authcode_fail() {
		let (_srv, port, _) = rpc::tests::ws::serve();
		let path = PathBuf::from("nonexist");

		let connect = Rpc::connect(&format!("ws://127.0.0.1:{}", port), &path);

		let _ = connect.map(|conn| {
			assert!(matches!(&conn, &Err(RpcError::NoAuthCode)));
		}).wait();
	}

	#[test]
	fn test_authcode_correct() {
		let (_srv, port, mut authcodes) = rpc::tests::ws::serve();

		let _ = authcodes.generate_new();
		authcodes.to_file(&authcodes.path).unwrap();

		let connect = Rpc::connect(&format!("ws://127.0.0.1:{}", port),
								   authcodes.path.as_path());

		let _ = connect.map(|conn| {
			assert!(conn.is_ok())
		}).wait();
	}

}
