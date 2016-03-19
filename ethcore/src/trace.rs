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
	/// The result of the operation; the gas used and the output data of the call.
	pub result: Option<(U256, Bytes)>,
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
	/// The result of the operation; tuple of the gas used, the address of the newly created account and its code.
	/// NOTE: Presently failed operations are not reported so this will always be `Some`.
	pub result: Option<(U256, Address, Bytes)>,
//	pub output: Bytes,
}

/// Description of an action that we trace; will be either a call or a create.
#[derive(Debug, Clone, PartialEq)]
pub enum TraceAction {
	/// Action isn't yet known.
	Unknown,
	/// It's a call action.
	Call(TraceCall),
	/// It's a create action.
	Create(TraceCreate),
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
}

impl Default for Trace {
	fn default() -> Trace {
		Trace {
			depth: 0,
			action: TraceAction::Unknown,
			subs: vec![],
		}
	}
}

impl TraceAction {
	/// Compose a `TraceAction` from an `ActionParams`, knowing that the action is a call.
	pub fn from_call(p: &ActionParams) -> TraceAction {
		TraceAction::Call(TraceCall {
			from: p.sender.clone(),
			to: p.address.clone(),
			value: match p.value { ActionValue::Transfer(ref x) | ActionValue::Apparent(ref x) => x.clone() },
			gas: p.gas.clone(),
			input: p.data.clone().unwrap_or(vec![]),
			result: None,
		})
	}

	/// Compose a `TraceAction` from an `ActionParams`, knowing that the action is a create.
	pub fn from_create(p: &ActionParams) -> TraceAction {
		TraceAction::Create(TraceCreate {
			from: p.sender.clone(),
			value: match p.value { ActionValue::Transfer(ref x) | ActionValue::Apparent(ref x) => x.clone() },
			gas: p.gas.clone(),
			init: p.code.clone().unwrap_or(vec![]),
			result: None,
		})
	}
}

