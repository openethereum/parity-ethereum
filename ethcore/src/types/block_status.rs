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

//! Block status description module
use verification::queue::Status as QueueStatus;

/// General block status
#[derive(Debug, Eq, PartialEq)]
#[cfg_attr(feature = "ipc", binary)]
pub enum BlockStatus {
	/// Part of the blockchain.
	InChain,
	/// Queued for import.
	Queued,
	/// Known as bad.
	Bad,
	/// Unknown.
	Unknown,
}

impl From<QueueStatus> for BlockStatus {
	fn from(status: QueueStatus) -> Self {
		match status {
			QueueStatus::Queued => BlockStatus::Queued,
			QueueStatus::Bad => BlockStatus::Bad,
			QueueStatus::Unknown => BlockStatus::Unknown,
		}
	}
}
