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

//! Unique identifiers.

use util::hash::H256;
use header::BlockNumber;
use ipc::binary::BinaryConvertError;
use std::mem;
use std::collections::VecDeque;

/// Uniquely identifies block.
#[derive(Debug, PartialEq, Clone, Hash, Eq, Binary)]
pub enum BlockID {
	/// Block's sha3.
	/// Querying by hash is always faster.
	Hash(H256),
	/// Block number within canon blockchain.
	Number(BlockNumber),
	/// Earliest block (genesis).
	Earliest,
	/// Latest mined block.
	Latest
}

/// Uniquely identifies transaction.
#[derive(Debug, PartialEq, Clone, Hash, Eq, Binary)]
pub enum TransactionID {
	/// Transaction's sha3.
	Hash(H256),
	/// Block id and transaction index within this block.
	/// Querying by block position is always faster.
	Location(BlockID, usize)
}

/// Uniquely identifies Trace.
pub struct TraceId {
	/// Transaction
	pub transaction: TransactionID,
	/// Trace address within transaction.
	pub address: Vec<usize>,
}

/// Uniquely identifies Uncle.
#[derive(Debug)]
pub struct UncleID (
	/// Block id.
	pub BlockID,
	/// Position in block.
	pub usize
);
