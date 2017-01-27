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

//! Mode type

pub use std::time::Duration;
use client::Mode as ClientMode;

/// IPC-capable shadow-type for `client::config::Mode`
#[derive(Clone, Debug)]
#[cfg_attr(feature = "ipc", binary)]
pub enum Mode {
	/// Same as `ClientMode::Off`.
	Off,
	/// Same as `ClientMode::Dark`; values in seconds.
	Dark(u64),
	/// Same as `ClientMode::Passive`; values in seconds.
	Passive(u64, u64),
	/// Same as `ClientMode::Active`.
	Active,
}

impl From<ClientMode> for Mode {
	fn from(mode: ClientMode) -> Self {
		match mode {
			ClientMode::Off => Mode::Off,
			ClientMode::Dark(timeout) => Mode::Dark(timeout.as_secs()),
			ClientMode::Passive(timeout, alarm) => Mode::Passive(timeout.as_secs(), alarm.as_secs()),
			ClientMode::Active => Mode::Active,
		}
	}
}

impl From<Mode> for ClientMode {
	fn from(mode: Mode) -> Self {
		match mode {
			Mode::Off => ClientMode::Off,
			Mode::Dark(timeout) => ClientMode::Dark(Duration::from_secs(timeout)),
			Mode::Passive(timeout, alarm) => ClientMode::Passive(Duration::from_secs(timeout), Duration::from_secs(alarm)),
			Mode::Active => ClientMode::Active,
		}
	}
}
