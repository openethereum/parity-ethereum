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

//! Snappy compression bindings.

use std::fmt;
use libc::{c_char, c_int, size_t};

const SNAPPY_OK: c_int = 0;
const SNAPPY_INVALID_INPUT: c_int = 1;
const SNAPPY_BUFFER_TOO_SMALL: c_int = 2;

#[link(name = "snappy")]
extern {
	fn snappy_compress(
		input: *const c_char,
		input_len: size_t,
		compressed: *mut c_char,
		compressed_len: *mut size_t
	) -> c_int;

	fn snappy_max_compressed_length(source_len: size_t) -> size_t;

	fn snappy_uncompress(
		compressed: *const c_char,
		compressed_len: size_t,
		uncompressed: *mut c_char,
		uncompressed_len: *mut size_t,
	) -> c_int;

	fn snappy_uncompressed_length(
		compressed: *const c_char,
		compressed_len: size_t,
		result: *mut size_t,
	) -> c_int;

	fn snappy_validate_compressed_buffer(
		compressed: *const c_char,
		compressed_len: size_t,
	) -> c_int;
}

/// Attempted to decompress an uncompressed buffer.
#[derive(Debug)]
pub struct InvalidInput;

impl fmt::Display for InvalidInput {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "Attempted snappy decompression with invalid input")
	}
}

/// The maximum compressed length given a size.
pub fn max_compressed_len(len: usize) -> usize {
	unsafe { snappy_max_compressed_length(len as size_t) as usize }
}

/// How large the given data will be when decompressed.
pub fn decompressed_len(compressed: &[u8]) -> Result<usize, InvalidInput> {
	let mut size: size_t = 0;
	let len = compressed.len() as size_t;

	let status = unsafe { snappy_uncompressed_length(compressed.as_ptr() as *const c_char, len, &mut size) };

	if status == SNAPPY_INVALID_INPUT {
		Err(InvalidInput)
	} else {
		Ok(size)
	}
}

/// Compress a buffer using snappy.
pub fn compress(input: &[u8]) -> Vec<u8> {
	let mut buf = Vec::new();
	compress_into(input, &mut buf);
	buf
}

/// Compress a buffer using snappy, writing the result into
/// the given output buffer, growing it if necessary.
/// Otherwise, returns the length of the compressed data.
pub fn compress_into(input: &[u8], output: &mut Vec<u8>) -> usize {
	let mut len = max_compressed_len(input.len());

	if output.len() < len {
		output.resize(len, 0);
	}

	let status = unsafe {
		snappy_compress(
			input.as_ptr() as *const c_char,
			input.len() as size_t,
			output.as_mut_ptr() as *mut c_char,
			&mut len as &mut size_t,
		)
	};

	match status {
		SNAPPY_OK => len,
		SNAPPY_INVALID_INPUT => panic!("snappy compression has no concept of invalid input"),
		SNAPPY_BUFFER_TOO_SMALL => panic!("buffer cannot be too small, the capacity was just ensured."),
		_ => panic!("snappy returned unspecified status"),
	}
}

/// Decompress a buffer using snappy. Will return an error if the buffer is not snappy-compressed.
pub fn decompress(input: &[u8]) -> Result<Vec<u8>, InvalidInput> {
	let mut v = Vec::new();
	decompress_into(input, &mut v).map(|_| v)
}

/// Decompress a buffer using snappy, writing the result into
/// the given output buffer, growing it if necessary.
/// Will error if the input buffer is not snappy-compressed.
/// Otherwise, returns the length of the decompressed data.
pub fn decompress_into(input: &[u8], output: &mut Vec<u8>) -> Result<usize, InvalidInput> {
	let mut len = try!(decompressed_len(input));

	if output.len() < len {
		output.resize(len, 0);
	}

	let status = unsafe {
		snappy_uncompress(
			input.as_ptr() as *const c_char,
			input.len() as size_t,
			output.as_mut_ptr() as *mut c_char,
			&mut len as &mut size_t,
		)
	};

	match status {
		SNAPPY_OK => Ok(len as usize),
		SNAPPY_INVALID_INPUT => Err(InvalidInput),
		SNAPPY_BUFFER_TOO_SMALL => panic!("buffer cannot be too small, size was just set to large enough."),
		_ => panic!("snappy returned unspecified status"),
	}
}

/// Validate a compressed buffer. True if valid, false if not.
pub fn validate_compressed_buffer(input: &[u8]) -> bool {
	let status = unsafe { snappy_validate_compressed_buffer(input.as_ptr() as *const c_char, input.len() as size_t )};
	status == SNAPPY_OK
}