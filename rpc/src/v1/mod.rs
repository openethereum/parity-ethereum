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

//! Ethcore rpc v1.
//!
//! Compliant with ethereum rpc.

#[macro_use]
mod helpers;
mod impls;
mod metadata;

pub mod traits;
pub mod tests;
pub mod types;

pub use self::traits::{Web3, Eth, EthFilter, EthSigning, Net, Parity, ParityAccounts, ParitySet, ParitySigning, Signer, Personal, Traces, Rpc};
pub use self::impls::*;
pub use self::helpers::{SigningQueue, SignerService, ConfirmationsQueue, NetworkSettings, block_import};
pub use self::metadata::{Metadata, Origin};
