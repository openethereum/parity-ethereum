// Copyright 2015-2020 Parity Technologies (UK) Ltd.
// This file is part of Open Ethereum.

// Open Ethereum is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Open Ethereum is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Open Ethereum.  If not, see <http://www.gnu.org/licenses/>.

//! EIP-191 specific types

use ethereum_types::H160;
use serde::{Deserialize, Deserializer};
use serde::de;
use v1::types::Bytes;

/// EIP-191 version specifier
#[derive(Debug)]
pub enum EIP191Version {
	/// byte specifier for structured data (0x01)
	StructuredData,
	/// byte specifier for personal message (0x45)
	PersonalMessage,
	/// byte specifier for presignedtransaction (0x00)
	PresignedTransaction
}

/// EIP-191 version 0x0 struct
#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PresignedTransaction {
	// address of intended validator
	pub validator: H160,
	// application specific data
	pub data: Bytes
}

impl<'de> Deserialize<'de> for EIP191Version {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
		where
			D: Deserializer<'de>,
	{
		let s = String::deserialize(deserializer)?;
		let byte_version = match s.as_str() {
			"0x00" => EIP191Version::PresignedTransaction,
			"0x01" => EIP191Version::StructuredData,
			"0x45" => EIP191Version::PersonalMessage,
			other => return Err(de::Error::custom(format!("Invalid byte version '{}'", other))),
		};
		Ok(byte_version)
	}
}
