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

//! Tracing datatypes.
use common::*;

/// `TraceCall` result.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct TraceCallResult {
	/// Gas used by call.
	pub gas_used: U256,
	/// Call Output.
	pub output: Bytes,
}

/// `TraceCreate` result.
#[derive(Debug, Clone, PartialEq)]
pub struct TraceCreateResult {
	/// Gas used by create.
	pub gas_used: U256,
	/// Code of the newly created contract.
	pub code: Bytes,
	/// Address of the newly created contract.
	pub address: Address,
}

/// Description of a _call_ action, either a `CALL` operation or a message transction.
#[derive(Debug, Clone, PartialEq)]
pub struct TraceCall {
	/// The sending account.
	pub from: Address,
	/// The destination account.
	pub to: Address,
	/// The value transferred to the destination account.
	pub value: U256,
	/// The gas available for executing the call.
	pub gas: U256,
	/// The input data provided to the call.
	pub input: Bytes,
}

impl From<ActionParams> for TraceCall {
	fn from(p: ActionParams) -> Self {
		TraceCall {
			from: p.sender,
			to: p.address,
			value: p.value.value(),
			gas: p.gas,
			input: p.data.unwrap_or_else(Vec::new),
		}
	}
}

/// Description of a _create_ action, either a `CREATE` operation or a create transction.
#[derive(Debug, Clone, PartialEq)]
pub struct TraceCreate {
	/// The address of the creator.
	pub from: Address,
	/// The value with which the new account is endowed.
	pub value: U256,
	/// The gas available for the creation init code.
	pub gas: U256,
	/// The init code.
	pub init: Bytes,
}

impl From<ActionParams> for TraceCreate {
	fn from(p: ActionParams) -> Self {
		TraceCreate {
			from: p.sender,
			value: p.value.value(),
			gas: p.gas,
			init: p.code.unwrap_or_else(Vec::new),
		}
	}
}

/// Description of an action that we trace; will be either a call or a create.
#[derive(Debug, Clone, PartialEq)]
pub enum TraceAction {
	/// It's a call action.
	Call(TraceCall),
	/// It's a create action.
	Create(TraceCreate),
}

/// The result of the performed action.
#[derive(Debug, Clone, PartialEq)]
pub enum TraceResult {
	/// Successful call action result.
	Call(TraceCallResult),
	/// Successful create action result.
	Create(TraceCreateResult),
	/// Failed call.
	FailedCall,
	/// Failed create.
	FailedCreate,
}

#[derive(Debug, Clone, PartialEq)]
/// A trace; includes a description of the action being traced and sub traces of each interior action.
pub struct Trace {
	/// The number of EVM execution environments active when this action happened; 0 if it's
	/// the outer action of the transaction.
	pub depth: usize,
	/// The action being performed.
	pub action: TraceAction,
	/// The sub traces for each interior action performed as part of this call.
	pub subs: Vec<Trace>,
	/// The result of the performed action.
	pub result: TraceResult,
}
