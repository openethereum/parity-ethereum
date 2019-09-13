// Copyright 2015-2019 Parity Technologies (UK) Ltd.
// This file is part of Parity Ethereum.

// Parity Ethereum is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity Ethereum is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity Ethereum.  If not, see <http://www.gnu.org/licenses/>.

//! Secondary chunk creation and restoration, implementations for different consensus
//! engines.

mod authority;
mod work;

pub use self::authority::*;
pub use self::work::*;

use crate::SnapshotComponents;
use common_types::snapshot::Snapshotting::{self, *};

/// Create a factory for building snapshot chunks and restoring from them.
/// `None` indicates that the engine doesn't support snapshot creation.
pub fn chunker(snapshot_type: Snapshotting) -> Option<Box<dyn SnapshotComponents>> {
	match snapshot_type {
		PoA => Some(Box::new(PoaSnapshot)),
		PoW { blocks, max_restore_blocks } => Some(Box::new(PowSnapshot::new(blocks, max_restore_blocks))),
		Unsupported => None,
	}
}
