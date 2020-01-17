// Copyright 2015-2020 Parity Technologies (UK) Ltd.
// This file is part of Parity Ethereum.

// Parity Ethereum is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity Ethereum is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity Ethereum.  If not, see <http://www.gnu.org/licenses/>.

//! Account Metadata

use std::{
	collections::HashMap,
	time::Instant,
};

use parity_crypto::publickey::Address;
use ethkey::Password;
use serde_derive::{Serialize, Deserialize};
use serde_json;

/// Type of unlock.
#[derive(Clone, PartialEq)]
pub enum Unlock {
	/// If account is unlocked temporarily, it should be locked after first usage.
	OneTime,
	/// Account unlocked permanently can always sign message.
	/// Use with caution.
	Perm,
	/// Account unlocked with a timeout
	Timed(Instant),
}

/// Data associated with account.
#[derive(Clone)]
pub struct AccountData {
	pub unlock: Unlock,
	pub password: Password,
}

/// Collected account metadata
#[derive(Default, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AccountMeta {
	/// The name of the account.
	pub name: String,
	/// The rest of the metadata of the account.
	pub meta: String,
	/// The 128-bit Uuid of the account, if it has one (brain-wallets don't).
	pub uuid: Option<String>,
}

impl AccountMeta {
	/// Read a hash map of Address -> AccountMeta
	pub fn read<R>(reader: R) -> Result<HashMap<Address, Self>, serde_json::Error> where
		R: ::std::io::Read,
	{
		serde_json::from_reader(reader)
	}

	/// Write a hash map of Address -> AccountMeta
	pub fn write<W>(m: &HashMap<Address, Self>, writer: &mut W) -> Result<(), serde_json::Error> where
		W: ::std::io::Write,
	{
		serde_json::to_writer(writer, m)
	}
}

