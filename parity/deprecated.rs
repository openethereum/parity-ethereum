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

#![allow(dead_code)]

use std::fmt;
use cli::Args;

#[derive(Debug, PartialEq)]
pub enum Deprecated {
	DoesNothing(&'static str),
	Replaced(&'static str, &'static str),
	Removed(&'static str),
}

impl fmt::Display for Deprecated {
	fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
		match *self {
			Deprecated::DoesNothing(s) => write!(f, "Option '{}' does nothing. It's on by default.", s),
			Deprecated::Replaced(old, new) => write!(f, "Option '{}' is deprecated. Please use '{}' instead.", old, new),
			Deprecated::Removed(s) => write!(f, "Option '{}' has been removed and is no longer supported.", s)
		}
	}
}

/// Removed flags will go here.
pub fn find_deprecated(_args: &Args) -> Vec<Deprecated> {
	Vec::new()
}
