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

//! Spec-related errors.

use serde_json::Error as SerdeError;
use std::fmt::{Display, Formatter, Error as FmtError};

/// Spec-related errors.
#[derive(Debug)]
pub enum Error {
	/// An error with the deserialization of the Spec.
	Parse(SerdeError),
	/// A divisor has value 0.
	ZeroValueDivisor,
}

impl Display for Error {
	fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
		match *self {
			Error::Parse(ref err) => write!(f, "{}", err),
			Error::ZeroValueDivisor => write!(f, "Divisor cannot be 0"),
		}
	}
}

impl From<SerdeError> for Error {
	fn from(err: SerdeError) -> Error {
		Error::Parse(err)
	}
}
