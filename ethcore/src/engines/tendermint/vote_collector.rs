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

//! Collects votes on hashes at each height and round.

use super::vote::Vote;
use super::{Height, Round, BlockHash};
use common::{HashSet, HashMap, RwLock, H256, Address, Error, Hashable};
use ethkey::{Signature, recover};

/// Signed voting on hashes.
#[derive(Debug)]
pub struct VoteCollector {
	/// Structure for storing all votes.
	votes: RwLock<HashMap<(Height, Round), HashSet<Vote>>>,
}

impl VoteCollector {
	pub fn new() -> VoteCollector {
		VoteCollector { votes: RwLock::new(HashMap::new()) }
	}

	pub fn vote() {}
}
