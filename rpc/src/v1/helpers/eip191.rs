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

//! EIP-191 compliant decoding + hashing
use v1::types::{EIP191Version, Bytes, PresignedTransaction};
use eip712::{hash_structured_data, EIP712};
use serde_json::{Value, from_value};
use v1::helpers::errors;
use jsonrpc_core::Error;
use v1::helpers::dispatch::eth_data_hash;
use hash::keccak;
use std::fmt::Display;
use ethereum_types::H256;

/// deserializes and hashes the message depending on the version specifier
pub fn hash_message(version: EIP191Version, message: Value) -> Result<H256, Error> {
	let data = match version {
		EIP191Version::StructuredData => {
			let typed_data = from_value::<EIP712>(message)
				.map_err(map_serde_err("StructuredData"))?;

			hash_structured_data(typed_data)
				.map_err(|err| errors::invalid_call_data(err.kind()))?
		}

		EIP191Version::PresignedTransaction => {
			let data = from_value::<PresignedTransaction>(message)
				.map_err(map_serde_err("WithValidator"))?;
			let prefix = b"\x19\x00";
			let data = [&prefix[..], &data.validator.0[..], &data.data.0[..]].concat();
			keccak(data)
		}

		EIP191Version::PersonalMessage => {
			let bytes = from_value::<Bytes>(message)
				.map_err(map_serde_err("Bytes"))?;
			eth_data_hash(bytes.0)
		}
	};

	Ok(data)
}

fn map_serde_err<T: Display>(struct_name: &'static str) -> impl Fn(T) -> Error {
	move |error: T| {
		errors::invalid_call_data(format!("Error deserializing '{}': {}", struct_name, error))
	}
}
