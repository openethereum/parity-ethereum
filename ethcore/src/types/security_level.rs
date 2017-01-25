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

//! Indication of how secure the chain is.

use header::BlockNumber;

/// Indication of how secure the chain is.
#[derive(Debug, PartialEq, Copy, Clone, Hash, Eq)]
#[cfg_attr(feature = "ipc", binary)]
pub enum SecurityLevel {
	/// All blocks from genesis to chain head are known to have valid state transitions and PoW.
	FullState,
	/// All blocks from genesis to chain head are known to have a valid PoW.
	FullProofOfWork,
	/// Some recent headers (the argument) are known to have a valid PoW.
	PartialProofOfWork(BlockNumber),
}

impl SecurityLevel {
	/// `true` for `FullPoW`/`FullState`.
	pub fn is_full(&self) -> bool {
		match *self {
			SecurityLevel::FullState | SecurityLevel::FullProofOfWork => true,
			_ => false,
		}
	}
}
