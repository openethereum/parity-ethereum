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

//! `TraceDB` errors.

use std::fmt::{Display, Formatter, Error as FmtError};

const RESYNC_ERR: &'static str =
"Your current parity installation has synced without transaction tracing.
To use Parity with transaction tracing, you'll need to resync with tracing.
To do this, remove or move away your current database and restart parity. e.g.:

> mv ~/.parity/906a34e69aec8c0d /tmp
> parity";

/// `TraceDB` errors.
#[derive(Debug)]
pub enum Error {
	/// Returned when tracing is enabled,
	/// but database does not contain traces of old transactions.
	ResyncRequired,
}

impl Display for Error {
	fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
		write!(f, "{}", RESYNC_ERR)
	}
}
