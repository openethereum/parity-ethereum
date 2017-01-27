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

//! Ethereum virtual machine.

pub mod ext;
pub mod evm;
pub mod interpreter;
#[macro_use]
pub mod factory;
pub mod schedule;
mod instructions;
#[cfg(feature = "jit" )]
mod jit;

#[cfg(test)]
mod tests;
#[cfg(all(feature="benches", test))]
mod benches;

pub use self::evm::{Evm, Error, Finalize, GasLeft, Result, CostType};
pub use self::ext::{Ext, ContractCreateResult, MessageCallResult};
pub use self::factory::{Factory, VMType};
pub use self::schedule::Schedule;
pub use types::executed::CallType;
