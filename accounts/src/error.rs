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

use std::fmt;

use ethstore::{Error as SSError};
use hardware_wallet::{Error as HardwareError};

/// Signing error
#[derive(Debug)]
pub enum SignError {
	/// Account is not unlocked
	NotUnlocked,
	/// Account does not exist.
	NotFound,
	/// Low-level hardware device error.
	Hardware(HardwareError),
	/// Low-level error from store
	SStore(SSError),
}

impl fmt::Display for SignError {
	fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
		match *self {
			SignError::NotUnlocked => write!(f, "Account is locked"),
			SignError::NotFound => write!(f, "Account does not exist"),
			SignError::Hardware(ref e) => write!(f, "{}", e),
			SignError::SStore(ref e) => write!(f, "{}", e),
		}
	}
}

impl From<HardwareError> for SignError {
	fn from(e: HardwareError) -> Self {
		SignError::Hardware(e)
	}
}

impl From<SSError> for SignError {
	fn from(e: SSError) -> Self {
		SignError::SStore(e)
	}
}
