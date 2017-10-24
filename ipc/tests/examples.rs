// Copyright 2015-2017 Parity Technologies (UK) Ltd.
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

#[cfg(test)]
mod tests {

	use super::super::service::*;
	use super::super::binary::*;
	use super::super::nested::{DBClient, DBWriter};
	use ipc::*;
	use devtools::*;
	use semver::Version;
	use std::sync::Arc;

	#[test]
	fn call_service() {
		// method_num = 0, f = 10 (method Service::commit)
		let mut socket = TestSocket::new_ready(vec![
			0, 16,
			0, 0, 0, 0, 0, 0, 0, 0,
			4, 0, 0, 0, 0, 0, 0, 0,
			10, 0, 0, 0]);

		let service = Arc::new(Service::new());
		assert_eq!(0, *service.commits.read().unwrap());

		service.dispatch(&mut socket);

		assert_eq!(10, *service.commits.read().unwrap());
	}

	#[test]
	fn call_service_client() {
		let mut socket = TestSocket::new();
		socket.read_buffer = vec![10, 0, 0, 0];
		let service_client = ServiceClient::init(socket);

		let result = service_client.commit(5);

		assert_eq!(
			vec![0, 16,
				0, 0, 0, 0, 0, 0, 0, 0,
				4, 0, 0, 0, 0, 0, 0, 0,
				5, 0, 0, 0],
			service_client.socket().write().unwrap().write_buffer.clone());
		assert_eq!(10, result);
	}

	#[test]
	fn call_service_client_optional() {
		let mut socket = TestSocket::new();
		socket.read_buffer = vec![10, 0, 0, 0];
		let service_client = ServiceClient::init(socket);

		let result = service_client.rollback(Some(5), 10);

		assert_eq!(vec![
			0, 17,
			1, 0, 0, 0, 0, 0, 0, 0,
			4, 0, 0, 0, 0, 0, 0, 0,
			8, 0, 0, 0, 0, 0, 0, 0,
			5, 0, 0, 0, 10, 0, 0, 0], service_client.socket().write().unwrap().write_buffer.clone());
		assert_eq!(10, result);
	}

	#[test]
	fn query_default_version() {
		let ver = Service::protocol_version();
		assert_eq!(ver, Version::parse("1.0.0").unwrap());
		let ver = Service::api_version();
		assert_eq!(ver, Version::parse("1.0.0").unwrap());
	}

	#[test]
	fn call_service_client_handshake() {
		let mut socket = TestSocket::new();
		socket.read_buffer = vec![1];
		let service_client = ServiceClient::init(socket);

		let result = service_client.handshake();

		assert!(result.is_ok());
	}

	#[test]
	fn can_use_custom_params() {
		let mut socket = TestSocket::new();
		socket.read_buffer = vec![1];
		let service_client = ServiceClient::init(socket);

		let result = service_client.push_custom(CustomData { a: 3, b: 11});

		assert_eq!(vec![
			// message num..
			0, 18,
			// variable size length-s
			1, 0, 0, 0, 0, 0, 0, 0,
			16, 0, 0, 0, 0, 0, 0, 0,
			// total length
			16, 0, 0, 0, 0, 0, 0, 0,
			// items
			3, 0, 0, 0, 0, 0, 0, 0,
			11, 0, 0, 0, 0, 0, 0, 0],
			service_client.socket().write().unwrap().write_buffer.clone());
		assert_eq!(true, result);
	}

	#[test]
	fn can_invoke_generic_service() {
		let mut socket = TestSocket::new();
		socket.read_buffer = vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
		let db_client = DBClient::<u64, _>::init(socket);
		let result = db_client.write(vec![1u8; 1]);
		assert_eq!(vec![0, 16, 1, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 1],
			db_client.socket().write().unwrap().write_buffer.clone());
		assert!(result.is_ok());
	}

	#[test]
	fn can_handshake_generic_service() {
		let mut socket = TestSocket::new();
		socket.read_buffer = vec![1];
		let db_client = DBClient::<u64, _>::init(socket);

		let result = db_client.handshake();

		assert!(result.is_ok());
	}

	#[test]
	fn can_serialize_dummy_structs() {
		let mut socket = TestSocket::new();

		let struct_ = DoubleRoot { x1: 0, x2: 100, x3: 100000};
		let res = ::ipc::binary::serialize_into(&struct_, &mut socket);

		assert!(res.is_ok());

		let mut read_socket = TestSocket::new_ready(socket.write_buffer.clone());
		let new_struct: DoubleRoot = ::ipc::binary::deserialize_from(&mut read_socket).unwrap();

		assert_eq!(struct_, new_struct);
	}

	#[test]
	fn can_call_void_method() {
		let mut socket = TestSocket::new();
		socket.read_buffer = vec![1];
		let service_client = ServiceClient::init(socket);

		service_client.void(99);

		assert_eq!(vec![
			0, 19,
			0, 0, 0, 0, 0, 0, 0, 0,
			8, 0, 0, 0, 0, 0, 0, 0,
			99, 0, 0, 0, 0, 0, 0, 0],
			service_client.socket().write().unwrap().write_buffer.clone());
	}
}
