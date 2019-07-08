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

//! Traces import request.
use ethereum_types::H256;

use crate::{FlatBlockTraces, BlockNumber};

/// Traces import request.
pub struct ImportRequest {
	/// Traces to import.
	pub traces: FlatBlockTraces,
	/// Hash of traces block.
	pub block_hash: H256,
	/// Number of traces block.
	pub block_number: BlockNumber,
	/// Blocks enacted by this import.
	///
	/// They should be ordered from oldest to newest.
	pub enacted: Vec<H256>,
	/// Number of blocks retracted by this import.
	pub retracted: usize,
}
