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

use util::H256;
use super::trace::{Trace, Action, Res};
use header::BlockNumber;

/// Localized trace.
pub struct LocalizedTrace {
	/// Index of the parent trace within the same transaction.
	pub parent: Option<usize>,
	/// Indexes of child traces within the same transaction.
	pub children: Vec<usize>,
	/// VM depth.
	pub depth: usize,
	/// Type of action performed by a transaction.
	pub action: Action,
	/// Result of this action.
	pub result: Res,
	/// Trace number within the transaction.
	pub trace_number: usize,
	/// Transaction number within the block.
	pub transaction_number: usize,
	/// Block number.
	pub block_number: BlockNumber,
	/// Block hash.
	pub block_hash: H256,
}
