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

#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate matches;

mod test {
	use futures::Future;
	use url::Url;
	use std::path::PathBuf;

	use client::{Rpc, RpcError};

	use mock;

	#[test]
	fn test_connection_refused() {
		let (srv, port, tmpdir, _) = mock::serve();

		let mut path = PathBuf::from(tmpdir.path());
		path.push("authcodes");
		let connect = Rpc::connect(&format!("ws://127.0.0.1:{}", port - 1), &path);

		connect.map(|conn| {
			assert!(matches!(&conn, &Err(RpcError::WsError(_))));
		}).wait();

		drop(srv);
	}

	#[test]
	fn test_authcode_fail() {
		let (srv, port, _, _) = mock::serve();
		let path = PathBuf::from("nonexist");

		let connect = Rpc::connect(&format!("ws://127.0.0.1:{}", port), &path);

		connect.map(|conn| {
			assert!(matches!(&conn, &Err(RpcError::NoAuthCode)));
		}).wait();

		drop(srv);
	}

	#[test]
	fn test_authcode_correct() {
		let (srv, port, tmpdir, _) = mock::serve();

		let mut path = PathBuf::from(tmpdir.path());
		path.push("authcodes");
		let connect = Rpc::connect(&format!("ws://127.0.0.1:{}", port), &path);

		connect.map(|conn| {
			assert!(conn.is_ok())
		}).wait();

		drop(srv);
	}

}
