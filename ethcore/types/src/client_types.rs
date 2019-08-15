// Copyright 2015-2019 Parity Technologies (UK) Ltd.
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

//! Client related types.

use std::{
	fmt::{Display, Formatter, Error as FmtError},
	time::Duration,
};

/// Operating mode for the client.
#[derive(Debug, Eq, PartialEq, Clone)]
pub enum Mode {
	/// Always on.
	Active,
	/// Goes offline after client is inactive for some (given) time, but
	/// comes back online after a while of inactivity.
	Passive(Duration, Duration),
	/// Goes offline after client is inactive for some (given) time and
	/// stays inactive.
	Dark(Duration),
	/// Always off.
	Off,
}

impl Display for Mode {
	fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
		match *self {
			Mode::Active => write!(f, "active"),
			Mode::Passive(..) => write!(f, "passive"),
			Mode::Dark(..) => write!(f, "dark"),
			Mode::Off => write!(f, "offline"),
		}
	}
}

