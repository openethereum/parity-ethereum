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

pub trait IpcInterface<T> {
	/// reads the message from io, dispatches the call and returns result
	fn dispatch<R>(&self, r: &mut R) -> Vec<u8> where R: ::std::io::Read;
}

pub fn invoke<W>(method_num: u16, params: &Option<Vec<u8>>, w: &mut W) where W: ::std::io::Write {
	let buf_len = match *params { None => 2, Some(ref val) => val.len() + 2 };
	let mut buf = vec![0u8; buf_len];
	buf[0] = (method_num & (255<<8)) as u8;
	buf[1] = (method_num >> 8) as u8;
	if params.is_some() {
		buf[2..buf_len].clone_from_slice(params.as_ref().unwrap());
	}
	if w.write(&buf).unwrap() != buf_len
	{
		panic!("failed to write to socket");
	}
}

pub trait IpcSocket: ::std::io::Read + ::std::io::Write {
	fn ready(&self) -> ::std::sync::atomic::AtomicBool;
}

impl IpcSocket for ::devtools::TestSocket {
	fn ready(&self) -> ::std::sync::atomic::AtomicBool {
		::std::sync::atomic::AtomicBool::new(true)
	}
}
