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

//! Common payload format definition, construction, and decoding.
//!
//! Format:
//! flags: 1 byte
//!
//! payload size: 0..4 bytes, BE, determined by flags.
//! optional padding: byte array up to 2^24 bytes in length. encoded in payload size.
//! optional signature: 65 bytes (r, s, v)
//!
//! payload: byte array of length of arbitrary size.
//!
//! flag bits used:
//!   0, 1 => how many bytes indicate padding length (up to 3)
//!   2 => whether signature is present
//!
//! padding is used to mask information about size of message.
//!
//! AES-256-GCM will append 12 bytes of metadata to the front of the message.

use ethereum_types::H256;
use byteorder::{BigEndian, ByteOrder, WriteBytesExt};
use ethkey::{Public, Secret};
use tiny_keccak::keccak256;

const SIGNATURE_LEN: usize = 65;

const STANDARD_PAYLOAD_VERSION: u8 = 1;

bitflags! {
	struct Flags: u8 {
		const FLAG_PAD_LEN_HIGH = 0b10000000;
		const FLAG_PAD_LEN_LOW  = 0b01000000;
		const FLAG_SIGNED       = 0b00100000;
	}
}

// number of bytes of padding length (in the range 0..4)
fn padding_length_bytes(flags: Flags) -> usize {
	match (flags & FLAG_PAD_LEN_HIGH, flags & FLAG_PAD_LEN_LOW) {
		(FLAG_PAD_LEN_HIGH, FLAG_PAD_LEN_LOW) => 3,
		(FLAG_PAD_LEN_HIGH, _) => 2,
		(_, FLAG_PAD_LEN_LOW) => 1,
		(_, _) => 0,
	}
}

// how many bytes are necessary to encode the given length. Range 0..4.
// `None` if too large.
fn num_padding_length_bytes(padding_len: usize) -> Option<usize> {
	let bits = 64 - (padding_len as u64).leading_zeros();
	match bits {
		0 => Some(0),
		0 ... 8 => Some(1),
		0 ... 16 => Some(2),
		0 ... 24 => Some(3),
		_ => None,
	}
}

/// Parameters for encoding a standard payload.
pub struct EncodeParams<'a> {
	/// Message to encode.
	pub message: &'a [u8],
	/// Padding bytes. Maximum padding allowed is 65536 bytes.
	pub padding: Option<&'a [u8]>,
	/// Private key to sign with.
	pub sign_with: Option<&'a Secret>,
}

impl<'a> Default for EncodeParams<'a> {
	fn default() -> Self {
		EncodeParams {
			message: &[],
			padding: None,
			sign_with: None,
		}
	}
}

/// Parameters for decoding a standard payload.
pub struct Decoded<'a> {
	/// Decoded message.
	pub message: &'a [u8],
	/// optional padding.
	pub padding: Option<&'a [u8]>,
	/// Recovered signature.
	pub from: Option<Public>,
}

/// Encode using provided parameters.
pub fn encode(params: EncodeParams) -> Result<Vec<u8>, &'static str> {
	const VEC_WRITE_INFALLIBLE: &'static str = "writing to a Vec<u8> can never fail; qed";

	let padding_len = params.padding.map_or(0, |x| x.len());
	let padding_len_bytes = num_padding_length_bytes(padding_len)
		.ok_or_else(|| "padding size too long")?;

	let signature = params.sign_with.map(|secret| {
		let hash = H256(keccak256(params.message));
		::ethkey::sign(secret, &hash)
	});

	let signature = match signature {
		Some(Ok(sig)) => Some(sig),
		Some(Err(_)) => return Err("invalid signing key provided"),
		None => None,
	};

	let (flags, plaintext_size) = {
		let mut flags = Flags::empty();

		// 1 byte each for flags and version.
		let mut plaintext_size = 2
			+ padding_len_bytes
			+ padding_len
			+ params.message.len();

		flags.bits = (padding_len_bytes << 6) as u8;
		debug_assert_eq!(padding_length_bytes(flags), padding_len_bytes);

		if let Some(ref sig) = signature {
			plaintext_size += sig.len();
			flags |= FLAG_SIGNED;
		}

		(flags, plaintext_size)
	};

	let mut plaintext = Vec::with_capacity(plaintext_size);

	plaintext.push(STANDARD_PAYLOAD_VERSION);
	plaintext.push(flags.bits);

	if let Some(padding) = params.padding {
		plaintext.write_uint::<BigEndian>(padding_len as u64, padding_len_bytes)
			.expect(VEC_WRITE_INFALLIBLE);

		plaintext.extend(padding)
	}

	if let Some(signature) = signature {
		plaintext.extend(signature.r());
		plaintext.extend(signature.s());
		plaintext.push(signature.v());
	}

	plaintext.extend(params.message);

	Ok(plaintext)
}

/// Decode using provided parameters
pub fn decode(payload: &[u8]) -> Result<Decoded, &'static str> {
	let mut offset = 0;

	let (padding, signature) = {
		// use a closure for reading slices since std::io::Read would require
		// us to copy.
		let mut next_slice = |len| {
			let end = offset + len;
			if payload.len() >= end {
				let slice = &payload[offset .. end];
				offset = end;

				Ok(slice)
			} else {
				return Err("unexpected end of payload")
			}
		};

		if next_slice(1)?[0] != STANDARD_PAYLOAD_VERSION {
			return Err("unknown payload version.");
		}

		let flags = Flags::from_bits_truncate(next_slice(1)?[0]);

		let padding_len_bytes = padding_length_bytes(flags);
		let padding = if padding_len_bytes != 0 {
			let padding_len = BigEndian::read_uint(
				next_slice(padding_len_bytes)?,
				padding_len_bytes,
			);

			Some(next_slice(padding_len as usize)?)
		} else {
			None
		};

		let signature = if flags & FLAG_SIGNED == FLAG_SIGNED {
			let slice = next_slice(SIGNATURE_LEN)?;
			let mut arr = [0; SIGNATURE_LEN];

			arr.copy_from_slice(slice);
			let signature = ::ethkey::Signature::from(arr);

			let not_rsv = signature.r() != &slice[..32]
				|| signature.s() != &slice[32..64]
				|| signature.v() != slice[64];

			if not_rsv {
				return Err("signature not in RSV format");
			} else {
				Some(signature)
			}
		} else {
			None
		};

		(padding, signature)
	};

	// remaining data is the message.
	let message = &payload[offset..];

	let from = match signature {
		None => None,
		Some(sig) => {
			let hash = H256(keccak256(message));
			Some(::ethkey::recover(&sig, &hash).map_err(|_| "invalid signature")?)
		}
	};

	Ok(Decoded {
		message: message,
		padding: padding,
		from: from,
	})
}

#[cfg(test)]
mod tests {
	use ethkey::{Generator, Random};
	use super::*;

	#[test]
	fn padding_len_bytes_sanity() {
		const U24_MAX: usize = (1 << 24) - 1;

		assert_eq!(padding_length_bytes(FLAG_PAD_LEN_HIGH | FLAG_PAD_LEN_LOW), 3);
		assert_eq!(padding_length_bytes(FLAG_PAD_LEN_HIGH), 2);
		assert_eq!(padding_length_bytes(FLAG_PAD_LEN_LOW), 1);
		assert_eq!(padding_length_bytes(Flags::empty()), 0);

		assert!(num_padding_length_bytes(u32::max_value() as _).is_none());
		assert!(num_padding_length_bytes(U24_MAX + 1).is_none());

		assert_eq!(num_padding_length_bytes(U24_MAX), Some(3));

		assert_eq!(num_padding_length_bytes(u16::max_value() as usize + 1), Some(3));
		assert_eq!(num_padding_length_bytes(u16::max_value() as usize), Some(2));

		assert_eq!(num_padding_length_bytes(u8::max_value() as usize + 1), Some(2));
		assert_eq!(num_padding_length_bytes(u8::max_value() as usize), Some(1));

		assert_eq!(num_padding_length_bytes(1), Some(1));
		assert_eq!(num_padding_length_bytes(0), Some(0));
	}

	#[test]
	fn encode_decode_roundtrip() {
		let message = [1, 2, 3, 4, 5];
		let encoded = encode(EncodeParams {
			message: &message,
			padding: None,
			sign_with: None,
		}).unwrap();

		let decoded = decode(&encoded).unwrap();

		assert_eq!(message, decoded.message);
	}

	#[test]
	fn encode_empty() {
		let encoded = encode(EncodeParams {
			message: &[],
			padding: None,
			sign_with: None,
		}).unwrap();

		let decoded = decode(&encoded).unwrap();

		assert!(decoded.message.is_empty());
	}

	#[test]
	fn encode_with_signature() {
		let key_pair = Random.generate().unwrap();
		let message = [1, 3, 5, 7, 9];

		let encoded = encode(EncodeParams {
			message: &message,
			padding: None,
			sign_with: Some(key_pair.secret()),
		}).unwrap();

		let decoded = decode(&encoded).unwrap();

		assert_eq!(decoded.message, message);
		assert_eq!(decoded.from, Some(key_pair.public().clone()));
		assert!(decoded.padding.is_none());
	}

	#[test]
	fn encode_with_padding() {
		let message = [1, 3, 5, 7, 9];
		let padding = [0xff; 1024 - 5];

		let encoded = encode(EncodeParams {
			message: &message,
			padding: Some(&padding),
			sign_with: None,
		}).unwrap();

		let decoded = decode(&encoded).unwrap();

		assert_eq!(decoded.message, message);
		assert_eq!(decoded.padding, Some(&padding[..]));
		assert!(decoded.from.is_none());
	}

	#[test]
	fn encode_with_padding_and_signature() {
		let key_pair = Random.generate().unwrap();
		let message = [1, 3, 5, 7, 9];
		let padding = [0xff; 1024 - 5];

		let encoded = encode(EncodeParams {
			message: &message,
			padding: Some(&padding),
			sign_with: Some(key_pair.secret()),
		}).unwrap();

		let decoded = decode(&encoded).unwrap();

		assert_eq!(decoded.message, message);
		assert_eq!(decoded.padding, Some(&padding[..]));
		assert_eq!(decoded.from, Some(key_pair.public().clone()));
	}
}
