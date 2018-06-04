// Copyright 2015-2018 Parity Technologies (UK) Ltd.
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

use byteorder::{LittleEndian, ReadBytesExt};
use std::io::{self, Read};

#[derive(Debug, PartialEq, Eq)]
pub struct PanicPayload {
	pub msg: Option<String>,
	pub file: Option<String>,
	pub line: Option<u32>,
	pub col: Option<u32>,
}

fn read_string(rdr: &mut io::Cursor<&[u8]>) -> io::Result<Option<String>> {
	let string_len = rdr.read_u32::<LittleEndian>()?;
	let string = if string_len == 0 {
		None
	} else {
		let mut content = vec![0; string_len as usize];
		rdr.read_exact(&mut content)?;
		Some(String::from_utf8_lossy(&content).into_owned())
	};
	Ok(string)
}

pub fn decode(raw: &[u8]) -> PanicPayload {
	let mut rdr = io::Cursor::new(raw);
	let msg = read_string(&mut rdr).ok().and_then(|x| x);
	let file = read_string(&mut rdr).ok().and_then(|x| x);
	let line = rdr.read_u32::<LittleEndian>().ok();
	let col = rdr.read_u32::<LittleEndian>().ok();
	PanicPayload {
		msg: msg,
		file: file,
		line: line,
		col: col,
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use byteorder::WriteBytesExt;

	fn write_u32(payload: &mut Vec<u8>, val: u32) {
		payload.write_u32::<LittleEndian>(val).unwrap();
	}

	fn write_bytes(payload: &mut Vec<u8>, bytes: &[u8]) {
		write_u32(payload, bytes.len() as u32);
		payload.extend(bytes);
	}

	#[test]
	fn it_works() {
		let mut raw = Vec::new();
		write_bytes(&mut raw, b"msg");
		write_bytes(&mut raw, b"file");
		write_u32(&mut raw, 1);
		write_u32(&mut raw, 2);

		let payload = decode(&raw);

		assert_eq!(
			payload,
			PanicPayload {
				msg: Some("msg".to_string()),
				file: Some("file".to_string()),
				line: Some(1),
				col: Some(2),
			}
		);
	}

	#[test]
	fn only_msg() {
		let mut raw = Vec::new();
		write_bytes(&mut raw, b"msg");

		let payload = decode(&raw);

		assert_eq!(
			payload,
			PanicPayload {
				msg: Some("msg".to_string()),
				file: None,
				line: None,
				col: None,
			}
		);
	}

	#[test]
	fn invalid_utf8() {
		let mut raw = Vec::new();
		write_bytes(&mut raw, b"\xF0\x90\x80msg");
		write_bytes(&mut raw, b"file");
		write_u32(&mut raw, 1);
		write_u32(&mut raw, 2);

		let payload = decode(&raw);

		assert_eq!(
			payload,
			PanicPayload {
				msg: Some("�msg".to_string()),
				file: Some("file".to_string()),
				line: Some(1),
				col: Some(2),
			}
		);
	}

	#[test]
	fn trailing_data() {
		let mut raw = Vec::new();
		write_bytes(&mut raw, b"msg");
		write_bytes(&mut raw, b"file");
		write_u32(&mut raw, 1);
		write_u32(&mut raw, 2);
		write_u32(&mut raw, 0xdeadbeef);

		let payload = decode(&raw);

		assert_eq!(
			payload,
			PanicPayload {
				msg: Some("msg".to_string()),
				file: Some("file".to_string()),
				line: Some(1),
				col: Some(2),
			}
		);
	}

	#[test]
	fn empty_str_is_none() {
		let mut raw = Vec::new();
		write_bytes(&mut raw, b"msg");
		write_bytes(&mut raw, b"");

		let payload = decode(&raw);

		assert_eq!(
			payload,
			PanicPayload {
				msg: Some("msg".to_string()),
				file: None,
				line: None,
				col: None,
			}
		);
	}
}
