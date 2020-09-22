// Copyright 2015-2020 Parity Technologies (UK) Ltd.
// This file is part of OpenEthereum.

// OpenEthereum is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// OpenEthereum is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with OpenEthereum.  If not, see <http://www.gnu.org/licenses/>.

//! Virtual machines support library

extern crate ethereum_types;
extern crate ethjson;
extern crate keccak_hash as hash;
extern crate parity_bytes as bytes;
extern crate patricia_trie_ethereum as ethtrie;
extern crate rlp;

mod action_params;
mod call_type;
mod env_info;
mod error;
mod ext;
mod return_data;
mod schedule;

pub mod tests;

pub use action_params::{ActionParams, ActionValue, ParamsType};
pub use call_type::CallType;
pub use env_info::{EnvInfo, LastHashes};
pub use error::{Error, ExecTrapError, ExecTrapResult, Result, TrapError, TrapKind, TrapResult};
pub use ext::{ContractCreateResult, CreateContractAddress, Ext, MessageCallResult};
pub use return_data::{GasLeft, ReturnData};
pub use schedule::{CleanDustMode, Schedule, WasmCosts};

/// Virtual Machine interface
pub trait Exec: Send {
    /// This function should be used to execute transaction.
    /// It returns either an error, a known amount of gas left, or parameters to be used
    /// to compute the final gas left.
    fn exec(self: Box<Self>, ext: &mut dyn Ext) -> ExecTrapResult<GasLeft>;
}

/// Resume call interface
pub trait ResumeCall: Send {
    /// Resume an execution for call, returns back the Vm interface.
    fn resume_call(self: Box<Self>, result: MessageCallResult) -> Box<dyn Exec>;
}

/// Resume create interface
pub trait ResumeCreate: Send {
    /// Resume an execution from create, returns back the Vm interface.
    fn resume_create(self: Box<Self>, result: ContractCreateResult) -> Box<dyn Exec>;
}
