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

/// Errors that can occur during usage of snappy.
#[derive(Debug)]
pub enum Error {
	/// An invalid input was supplied. Usually means that you tried to decompress an uncompressed
	/// buffer.
	InvalidInput,
	/// The output buffer supplied was too small. Make sure to provide buffers large enough to hold
	/// all the output data.
	BufferTooSmall,
}

impl fmt::Display for Error {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match *self {
			Error::InvalidInput => write!(f, "Snappy error (invalid input)"),
			Error::BufferTooSmall => write!(f, "Snappy error (buffer too small)"),
		}
	}
}

/// The maximum compressed length given a size.
pub fn max_compressed_len(len: usize) -> usize {
	unsafe { snappy_max_compressed_length(len as size_t) as usize }
}

/// How large the given data will be when decompressed.
pub fn decompressed_len(compressed: &[u8]) -> Result<usize, Error> {
	let mut size: size_t = 0;
	let len = compressed.len() as size_t;

	let status = unsafe { snappy_uncompressed_length(compressed.as_ptr() as *const c_char, len, &mut size) };

	if status == SNAPPY_INVALID_INPUT {
		Err(Error::InvalidInput)
	} else {
		Ok(len)
	}
}

/// Compress a buffer using snappy.
pub fn compress(input: &[u8]) -> Vec<u8> {
	let mut buf_size = max_compressed_len(input.len());
	let mut output = vec![0; buf_size as usize];

	buf_size = compress_into(input, &mut output).expect("snappy compression failed with large enough buffer.");
	output.truncate(buf_size);
	output
}

/// Compress a buffer using snappy, writing the result into
/// the given output buffer. Will error iff the buffer is too small.
/// Otherwise, returns the length of the compressed data.
pub fn compress_into(input: &[u8], output: &mut [u8]) -> Result<usize, Error> {
	let mut len = output.len() as size_t;
	let status = unsafe {
		snappy_compress(
			input.as_ptr() as *const c_char,
			input.len() as size_t,
			output.as_mut_ptr() as *mut c_char,
			&mut len,
		)
	};

	match status {
		SNAPPY_OK => Ok(len as usize),
		SNAPPY_INVALID_INPUT => Err(Error::InvalidInput), // should never happen, but can't hurt!
		SNAPPY_BUFFER_TOO_SMALL => Err(Error::BufferTooSmall),
		_ => panic!("snappy returned unspecified status"),
	}
}

/// Decompress a buffer using snappy. Will return an error if the buffer is not snappy-compressed.
pub fn decompress(input: &[u8]) -> Result<Vec<u8>, Error> {
	decompressed_len(input).and_then(|mut buf_size| {
		let mut output = vec![0; buf_size];

		buf_size = try!(decompress_into(input, &mut output));
		output.truncate(buf_size);
		Ok(output)
	})
}

/// Decompress a buffer using snappy, writing the result into
/// the given output buffer. Will error if the input buffer is not snappy-compressed
/// or the output buffer is too small.
/// Otherwise, returns the length of the decompressed data.
pub fn decompress_into(input: &[u8], output: &mut [u8]) -> Result<usize, Error> {
	let mut len = output.len() as size_t;
	let status = unsafe {
		snappy_uncompress(
			input.as_ptr() as *const c_char,
			input.len() as size_t,
			output.as_mut_ptr() as *mut c_char,
			&mut len,
		)
	};

	match status {
		SNAPPY_OK => Ok(len as usize),
		SNAPPY_INVALID_INPUT => Err(Error::InvalidInput),
		SNAPPY_BUFFER_TOO_SMALL => Err(Error::BufferTooSmall),
		_ => panic!("snappy returned unspecified status"),
	}
}

/// Validate a compressed buffer. True if valid, false if not.
pub fn validate_compressed_buffer(input: &[u8]) -> bool {
	let status = unsafe { snappy_validate_compressed_buffer(input.as_ptr() as *const c_char, input.len() as size_t )};
	status == SNAPPY_OK
}