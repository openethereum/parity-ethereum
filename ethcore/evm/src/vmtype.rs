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

/// Type of EVM to use.
#[derive(Debug, PartialEq, Clone)]
pub enum VMType {
	/// RUST EVM
	Interpreter
}

impl fmt::Display for VMType {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "{}", match *self {
			VMType::Interpreter => "INT"
		})
	}
}

impl Default for VMType {
	fn default() -> Self {
		VMType::Interpreter
	}
}

impl VMType {
	/// Return all possible VMs (Interpreter)
	pub fn all() -> Vec<VMType> {
		vec![VMType::Interpreter]
	}
}
