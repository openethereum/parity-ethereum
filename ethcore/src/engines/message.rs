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

//! Consensus message trait.

use std::fmt::Debug;
use util::{H256, H520, Hash};
use rlp::Encodable;

pub trait Message: Clone + PartialEq + Eq + Hash + Encodable + Debug {
	type Round: Clone + PartialEq + Eq + Hash + Default + Debug + Ord;

	fn signature(&self) -> H520;

	fn block_hash(&self) -> Option<H256>;

	fn round(&self) -> &Self::Round;

	fn is_broadcastable(&self) -> bool;
}
