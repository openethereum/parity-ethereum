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

//! Interface for Evm externalities.

use std::sync::Arc;
use ethereum_types::{U256, H256, Address};
use bytes::Bytes;
use call_type::CallType;
use env_info::EnvInfo;
use schedule::Schedule;
use return_data::ReturnData;
use error::Result;

/// Result of externalities create function.
pub enum ContractCreateResult {
	/// Returned when creation was successfull.
	/// Contains an address of newly created contract and gas left.
	Created(Address, U256),
	/// Returned when contract creation failed.
	/// VM doesn't have to know the reason.
	Failed,
	/// Reverted with REVERT.
	Reverted(U256, ReturnData),
}

/// Result of externalities call function.
pub enum MessageCallResult {
	/// Returned when message call was successfull.
	/// Contains gas left and output data.
	Success(U256, ReturnData),
	/// Returned when message call failed.
	/// VM doesn't have to know the reason.
	Failed,
	/// Returned when message call was reverted.
	/// Contains gas left and output data.
	Reverted(U256, ReturnData),
}

/// Specifies how an address is calculated for a new contract.
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum CreateContractAddress {
	/// Address is calculated from nonce and sender. Pre EIP-86 (Metropolis)
	FromSenderAndNonce,
	/// Address is calculated from code hash. Default since EIP-86
	FromCodeHash,
	/// Address is calculated from code hash and sender. Used by CREATE_P2SH instruction.
	FromSenderAndCodeHash,
}

/// Externalities interface for EVMs
pub trait Ext {
	/// Returns a value for given key.
	fn storage_at(&self, key: &H256) -> Result<H256>;

	/// Stores a value for given key.
	fn set_storage(&mut self, key: H256, value: H256) -> Result<()>;

	/// Determine whether an account exists.
	fn exists(&self, address: &Address) -> Result<bool>;

	/// Determine whether an account exists and is not null (zero balance/nonce, no code).
	fn exists_and_not_null(&self, address: &Address) -> Result<bool>;

	/// Balance of the origin account.
	fn origin_balance(&self) -> Result<U256>;

	/// Returns address balance.
	fn balance(&self, address: &Address) -> Result<U256>;

	/// Returns the hash of one of the 256 most recent complete blocks.
	fn blockhash(&mut self, number: &U256) -> H256;

	/// Creates new contract.
	///
	/// Returns gas_left and contract address if contract creation was succesfull.
	fn create(&mut self, gas: &U256, value: &U256, code: &[u8], address: CreateContractAddress) -> ContractCreateResult;

	/// Message call.
	///
	/// Returns Err, if we run out of gas.
	/// Otherwise returns call_result which contains gas left
	/// and true if subcall was successfull.
	fn call(&mut self,
		gas: &U256,
		sender_address: &Address,
		receive_address: &Address,
		value: Option<U256>,
		data: &[u8],
		code_address: &Address,
		output: &mut [u8],
		call_type: CallType
	) -> MessageCallResult;

	/// Returns code at given address
	fn extcode(&self, address: &Address) -> Result<Arc<Bytes>>;

	/// Returns code size at given address
	fn extcodesize(&self, address: &Address) -> Result<usize>;

	/// Creates log entry with given topics and data
	fn log(&mut self, topics: Vec<H256>, data: &[u8]) -> Result<()>;

	/// Should be called when transaction calls `RETURN` opcode.
	/// Returns gas_left if cost of returning the data is not too high.
	fn ret(self, gas: &U256, data: &ReturnData, apply_state: bool) -> Result<U256>;

	/// Should be called when contract commits suicide.
	/// Address to which funds should be refunded.
	fn suicide(&mut self, refund_address: &Address) -> Result<()> ;

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

	/// Decide if any more operations should be traced. Passthrough for the VM trace.
	fn trace_next_instruction(&mut self, _pc: usize, _instruction: u8) -> bool { false }

	/// Prepare to trace an operation. Passthrough for the VM trace.
	fn trace_prepare_execute(&mut self, _pc: usize, _instruction: u8, _gas_cost: U256) {}

	/// Trace the finalised execution of a single instruction.
	fn trace_executed(&mut self, _gas_used: U256, _stack_push: &[U256], _mem_diff: Option<(usize, &[u8])>, _store_diff: Option<(U256, U256)>) {}

	/// Check if running in static context.
	fn is_static(&self) -> bool;
}
