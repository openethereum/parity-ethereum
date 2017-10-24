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

//! Types used in the public API

use std::fmt;

/// A release's track.
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
#[binary]
pub enum ReleaseTrack {
	/// Stable track.
	Stable,
	/// Beta track.
	Beta,
	/// Nightly track.
	Nightly,
	/// Testing track.
	Testing,
	/// No known track, also "current executable's track" when it's not yet known.
	Unknown,
}

impl fmt::Display for ReleaseTrack {
	fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
		write!(f, "{}", match *self {
			ReleaseTrack::Stable => "stable",
			ReleaseTrack::Beta => "beta",
			ReleaseTrack::Nightly => "nightly",
			ReleaseTrack::Testing => "testing",
			ReleaseTrack::Unknown => "unknown",
		})
	}
}

impl<'a> From<&'a str> for ReleaseTrack {
	fn from(s: &'a str) -> Self {
		match s {
			"stable" => ReleaseTrack::Stable,
			"beta" => ReleaseTrack::Beta,
			"nightly" => ReleaseTrack::Nightly,
			"testing" => ReleaseTrack::Testing,
			_ => ReleaseTrack::Unknown,
		}
	}
}

impl From<u8> for ReleaseTrack {
	fn from(i: u8) -> Self {
		match i {
			1 => ReleaseTrack::Stable,
			2 => ReleaseTrack::Beta,
			3 => ReleaseTrack::Nightly,
			4 => ReleaseTrack::Testing,
			_ => ReleaseTrack::Unknown,
		}
	}
}

impl Into<u8> for ReleaseTrack {
	fn into(self) -> u8 {
		match self {
			ReleaseTrack::Stable => 1,
			ReleaseTrack::Beta => 2,
			ReleaseTrack::Nightly => 3,
			ReleaseTrack::Testing => 4,
			ReleaseTrack::Unknown => 0,
		}
	}
}
