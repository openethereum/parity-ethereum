// Copyright 2015, 2016 Ethcore (UK) Ltd.
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

//! IPC RPC interface

use std::io::{Read, Write};
use std::marker::Sync;
use semver::Version;

/// Handshake for client and server to negotiate api/protocol version
pub struct Handshake {
	pub protocol_version: Version,
	pub api_version: Version,
}

/// Allows to configure custom version and custom handshake response for
/// ipc host
pub trait IpcConfig {
	/// Current service api version
	/// Should be increased if any of the methods changes signature
	fn api_version() -> Version {
		Version::parse("1.0.0").unwrap()
	}

	/// Current ipc protocol version
	/// Should be increased only if signature of system methods changes
	fn protocol_version() -> Version {
		Version::parse("1.0.0").unwrap()
	}

	/// Default handshake requires exact versions match
	fn handshake(handshake: &Handshake) -> bool {
		handshake.protocol_version == Self::protocol_version() &&
			handshake.api_version == Self::api_version()
	}
}

/// Error in dispatching or invoking methods via IPC
#[derive(Debug)]
pub enum Error {
	UnkownSystemCall,
	ClientUnsupported,
	RemoteServiceUnsupported,
	HandshakeFailed,
}

/// Allows implementor to be attached to generic worker and dispatch rpc requests
/// over IPC
pub trait IpcInterface<T>: IpcConfig {
	/// reads the message from io, dispatches the call and returns serialized result
	fn dispatch<R>(&self, r: &mut R) -> Vec<u8> where R: Read;

	/// deserializes the payload from buffer, dispatches invoke and returns serialized result
	/// (for non-blocking io)
	fn dispatch_buf(&self, method_num: u16, buf: &[u8]) -> Vec<u8>;
}

/// serializes method invocation (method_num and parameters) to the stream specified by `w`
pub fn invoke<W>(method_num: u16, params: &Option<Vec<u8>>, w: &mut W) where W: Write {
	// creating buffer to contain all message
	let buf_len = match *params { None => 2, Some(ref val) => val.len() + 2 };
	let mut buf = vec![0u8; buf_len];

	// writing method_num as u16
	buf[1] = (method_num & 255) as u8;
	buf[0] = (method_num >> 8) as u8;

	// serializing parameters only if provided with any
	if params.is_some() {
		buf[2..buf_len].clone_from_slice(params.as_ref().unwrap());
	}

	if w.write(&buf).unwrap() != buf_len
	{
		// if write was inconsistent
		panic!("failed to write to socket");
	}
}

/// IpcSocket, read/write generalization
pub trait IpcSocket: Read + Write + Sync {
}

/// Basically something that needs only socket to be spawned
pub trait WithSocket<S: IpcSocket> {
	fn init(socket: S) -> Self;
}


impl IpcSocket for ::devtools::TestSocket {}

impl IpcSocket for ::nanomsg::Socket {}
