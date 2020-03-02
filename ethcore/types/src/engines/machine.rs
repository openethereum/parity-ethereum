// Copyright 2015-2020 Parity Technologies (UK) Ltd.
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

//! State machine types

use ethereum_types::{Address, U256};
use bytes::Bytes;

use crate::{
	log_entry::LogEntry,
	receipt,
	state_diff::StateDiff,
};

/// Type alias for a function we can make calls through synchronously.
/// Returns the call result and state proof for each call.
pub type Call<'a> = dyn Fn(Address, Vec<u8>) -> Result<(Vec<u8>, Vec<Vec<u8>>), String> + 'a;

/// Auxiliary data fetcher for an Ethereum machine. In Ethereum-like machines
/// there are two kinds of auxiliary data: bodies and receipts.
#[derive(Default, Clone)]
pub struct AuxiliaryData<'a> {
	/// The block receipts.
	pub receipts: Option<&'a [receipt::Receipt]>,
}


/// Transaction execution receipt.
#[derive(Debug, PartialEq, Clone)]
pub struct Executed<T, V> {
	/// True if the outer call/create resulted in an exceptional exit.
	pub exception: Option<vm::Error>,

	/// Gas paid up front for execution of transaction.
	pub gas: U256,

	/// Gas used during execution of transaction.
	pub gas_used: U256,

	/// Gas refunded after the execution of transaction.
	/// To get gas that was required up front, add `refunded` and `gas_used`.
	pub refunded: U256,

	/// Cumulative gas used in current block so far.
	///
	/// `cumulative_gas_used = gas_used(t0) + gas_used(t1) + ... gas_used(tn)`
	///
	/// where `tn` is current transaction.
	pub cumulative_gas_used: U256,

	/// Vector of logs generated by transaction.
	pub logs: Vec<LogEntry>,

	/// Addresses of contracts created during execution of transaction.
	/// Ordered from earliest creation.
	///
	/// eg. sender creates contract A and A in constructor creates contract B
	///
	/// B creation ends first, and it will be the first element of the vector.
	pub contracts_created: Vec<Address>,
	/// Transaction output.
	pub output: Bytes,
	/// The trace of this transaction.
	pub trace: Vec<T>,
	/// The VM trace of this transaction.
	pub vm_trace: Option<V>,
	/// The state diff, if we traced it.
	pub state_diff: Option<StateDiff>,
}
