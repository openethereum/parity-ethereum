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

//! Interface for Evm externalities.

use util::common::*;
use evm::{Schedule, Error};
use env_info::*;

/// Result of externalities create function.
pub enum ContractCreateResult {
	/// Returned when creation was successfull.
	/// Contains an address of newly created contract and gas left.
	Created(Address, U256),
	/// Returned when contract creation failed.
	/// VM doesn't have to know the reason.
	Failed
}

/// Result of externalities call function.
pub enum MessageCallResult {
	/// Returned when message call was successfull.
	/// Contains gas left.
	Success(U256),
	/// Returned when message call failed.
	/// VM doesn't have to know the reason.
	Failed
}

/// Externalities interface for EVMs
pub trait Ext {
	/// Returns a value for given key.
	fn storage_at(&self, key: &H256) -> H256;

	/// Stores a value for given key.
	fn set_storage(&mut self, key: H256, value: H256);

	/// Determine whether an account exists.
	fn exists(&self, address: &Address) -> bool;

	/// Returns address balance.
	fn balance(&self, address: &Address) -> U256;

	/// Returns the hash of one of the 256 most recent complete blocks.
	fn blockhash(&self, number: &U256) -> H256;

	/// Creates new contract.
	///
	/// Returns gas_left and contract address if contract creation was succesfull.
	fn create(&mut self, gas: &U256, value: &U256, code: &[u8]) -> ContractCreateResult;

	/// Message call.
	///
	/// Returns Err, if we run out of gas.
	/// Otherwise returns call_result which contains gas left
	/// and true if subcall was successfull.
	#[cfg_attr(feature="dev", allow(too_many_arguments))]
	fn call(&mut self,
			gas: &U256,
			sender_address: &Address,
			receive_address: &Address,
			value: Option<U256>,
			data: &[u8],
			code_address: &Address,
			output: &mut [u8]) -> MessageCallResult;

	/// Returns code at given address
	fn extcode(&self, address: &Address) -> Bytes;

	/// Creates log entry with given topics and data
	fn log(&mut self, topics: Vec<H256>, data: &[u8]);

	/// Should be called when transaction calls `RETURN` opcode.
	/// Returns gas_left if cost of returning the data is not too high.
	fn ret(&mut self, gas: &U256, data: &[u8]) -> Result<U256, Error>;

	/// Should be called when contract commits suicide.
	/// Address to which funds should be refunded.
	fn suicide(&mut self, refund_address: &Address);

	/// Returns schedule.
	fn schedule(&self) -> &Schedule;

	/// Returns environment info.
	fn env_info(&self) -> &EnvInfo;

	/// Returns current depth of execution.
	///
	/// If contract A calls contract B, and contract B calls C,
	/// then A depth is 0, B is 1, C is 2 and so on.
	fn depth(&self) -> usize;

	/// Increments sstore refunds count by 1.
	fn inc_sstore_clears(&mut self);
}
