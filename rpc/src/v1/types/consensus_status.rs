// Copyright 2015, 2016 Parity Technologies (UK) Ltd.
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

use updater::CapState;

/// Capability info
#[derive(Debug, Serialize, PartialEq)]
pub enum ConsensusCapability {
	/// Unknown.
	#[serde(rename="unknown")]
	Unknown,
	/// Capable of consensus indefinitely.
	#[serde(rename="capable")]
	Capable,
	/// Capable of consensus up until a definite block. 
	#[serde(rename="capableUntil")]
	CapableUntil(u64),
	/// Incapable of consensus since a particular block. 
	#[serde(rename="incapableSince")]
	IncapableSince(u64),
}

impl Into<ConsensusCapability> for CapState {
	fn into(self) -> ConsensusCapability {
		match self {
			CapState::Unknown => ConsensusCapability::Unknown, 
			CapState::Capable => ConsensusCapability::Capable, 
			CapState::CapableUntil(n) => ConsensusCapability::CapableUntil(n), 
			CapState::IncapableSince(n) => ConsensusCapability::IncapableSince(n), 
		}
	}
}

