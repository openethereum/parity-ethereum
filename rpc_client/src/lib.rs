pub mod client;
pub mod signer;
mod mock;

extern crate ws;
extern crate ethcore_signer;
extern crate url;
extern crate futures;
extern crate ethcore_util as util;
extern crate ethcore_rpc as rpc;
extern crate serde;
extern crate serde_json;
extern crate rand;
extern crate tempdir;
extern crate jsonrpc_core;

#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate matches;

#[cfg(test)]
mod tests {
	use futures::Future;
	use std::path::PathBuf;

	use client::{Rpc, RpcError};

	use mock::serve;

	#[test]
	fn test_connection_refused() {
		let (_srv, port, tmpdir, _) = serve();

		let mut path = PathBuf::from(tmpdir.path());
		path.push("authcodes");
		let connect = Rpc::connect(&format!("ws://127.0.0.1:{}", port - 1), &path);

		let _ = connect.map(|conn| {
			assert!(matches!(&conn, &Err(RpcError::WsError(_))));
		}).wait();
	}

	#[test]
	fn test_authcode_fail() {
		let (_srv, port, _, _) = serve();
		let path = PathBuf::from("nonexist");

		let connect = Rpc::connect(&format!("ws://127.0.0.1:{}", port), &path);

		let _ = connect.map(|conn| {
			assert!(matches!(&conn, &Err(RpcError::NoAuthCode)));
		}).wait();
	}

	#[test]
	fn test_authcode_correct() {
		let (_srv, port, tmpdir, _) = serve();

		let mut path = PathBuf::from(tmpdir.path());
		path.push("authcodes");
		let connect = Rpc::connect(&format!("ws://127.0.0.1:{}", port), &path);

		let _ = connect.map(|conn| {
			assert!(conn.is_ok())
		}).wait();
	}

}
