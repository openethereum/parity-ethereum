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

use std::fmt;

use ethkey;
use util;

/// Document address type.
pub type DocumentAddress = util::H256;
/// Document key type.
pub type DocumentKey = util::Bytes;
/// Encrypted key type.
pub type DocumentEncryptedKey = util::Bytes;
/// Request signature type.
pub type RequestSignature = ethkey::Signature;
/// Public key type.
pub use ethkey::Public;

#[derive(Debug, Clone, PartialEq)]
#[binary]
/// Secret store error
pub enum Error {
	/// Bad signature is passed
	BadSignature,
	/// Access to resource is denied
	AccessDenied,
	/// Requested document not found
	DocumentNotFound,
	/// Database-related error
	Database(String),
	/// Internal error
	Internal(String),
}

#[derive(Debug)]
#[binary]
/// Secret store configuration
pub struct ServiceConfiguration {
	/// Interface to listen to
	pub listener_addr: String,
	/// Port to listen to
	pub listener_port: u16,
	/// Data directory path for secret store
	pub data_path: String,
}

impl fmt::Display for Error {
	fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
		match *self {
			Error::BadSignature => write!(f, "Bad signature"),
			Error::AccessDenied => write!(f, "Access dened"),
			Error::DocumentNotFound => write!(f, "Document not found"),
			Error::Database(ref msg) => write!(f, "Database error: {}", msg),
			Error::Internal(ref msg) => write!(f, "Internal error: {}", msg),
		}
	}
}

impl Into<String> for Error {
	fn into(self) -> String {
		format!("{}", self)
	}
}
