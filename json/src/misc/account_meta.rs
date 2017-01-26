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

//! Misc deserialization.

use std::io::{Read, Write};
use std::collections::HashMap;
use serde_json;
use util;
use hash;

/// Collected account metadata
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AccountMeta {
	/// The name of the account.
	pub name: String,
	/// The rest of the metadata of the account.
	pub meta: String,
	/// The 128-bit Uuid of the account, if it has one (brain-wallets don't).
	pub uuid: Option<String>,
}

impl Default for AccountMeta {
	fn default() -> Self {
		AccountMeta {
			name: String::new(),
			meta: "{}".to_owned(),
			uuid: None,
		}
	}
}

impl AccountMeta {
	/// Read a hash map of Address -> AccountMeta.
	pub fn read_address_map<R>(reader: R) -> Result<HashMap<util::Address, AccountMeta>, serde_json::Error> where R: Read {
		serde_json::from_reader(reader).map(|ok: HashMap<hash::Address, AccountMeta>|
			ok.into_iter().map(|(a, m)| (a.into(), m)).collect()
		)
	}

	/// Write a hash map of Address -> AccountMeta.
	pub fn write_address_map<W>(m: &HashMap<util::Address, AccountMeta>, writer: &mut W) -> Result<(), serde_json::Error> where W: Write {
		serde_json::to_writer(writer, &m.iter().map(|(a, m)| (a.clone().into(), m)).collect::<HashMap<hash::Address, _>>())
	}
}
