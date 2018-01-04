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

//! Virtual machines support library

extern crate ethcore_util as util;
extern crate ethcore_bigint as bigint;
extern crate ethcore_bytes as bytes;
extern crate common_types as types;
extern crate ethjson;
extern crate rlp;
extern crate keccak_hash as hash;
extern crate patricia_trie as trie;

mod action_params;
mod call_type;
mod env_info;
mod schedule;
mod ext;
mod return_data;
mod error;

pub mod tests;

pub use action_params::{ActionParams, ActionValue, ParamsType};
pub use call_type::CallType;
pub use env_info::{EnvInfo, LastHashes};
pub use schedule::{Schedule, CleanDustMode};
pub use ext::{Ext, MessageCallResult, ContractCreateResult, CreateContractAddress};
pub use return_data::{ReturnData, GasLeft};
pub use error::{Error, Result};

/// Virtual Machine interface
pub trait Vm {
	/// This function should be used to execute transaction.
	/// It returns either an error, a known amount of gas left, or parameters to be used
	/// to compute the final gas left.
	fn exec(&mut self, params: ActionParams, ext: &mut Ext) -> Result<GasLeft>;
}
