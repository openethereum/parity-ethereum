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

/// Uniquely identifies bloom group position.
#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct GroupPosition {
	/// Bloom level.
	pub level: usize,
	/// Index of the group.
	pub index: usize,
}

/// Uniquely identifies bloom position including the position in the group.
#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct Position {
 	/// Group position.
	pub group: GroupPosition,
	/// Number in group.
	pub number: usize,
}
