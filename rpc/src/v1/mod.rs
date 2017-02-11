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

// Upgrade a weak pointer, returning an error on failure.
macro_rules! take_weak {
	($weak: expr) => {
		match $weak.upgrade() {
			Some(arc) => arc,
			None => return Err(Error::internal_error()),
		}
	}
}

// Upgrade a weak pointer, returning an error leaf-future on failure.
macro_rules! take_weakf {
	($weak: expr) => {
		match $weak.upgrade() {
			Some(arc) => arc,
			None => return ::futures::future::err(Error::internal_error()).boxed(),
		}
	}
}

// short for "try_boxfuture"
// unwrap a result, returning a BoxFuture<_, Err> on failure.
macro_rules! try_bf {
	($res: expr) => {
		match $res {
			Ok(val) => val,
			Err(e) => return ::futures::future::err(e.into()).boxed(),
		}
	}
}

#[macro_use]
mod helpers;
mod impls;
mod metadata;

pub mod traits;
pub mod tests;
pub mod types;

pub use self::traits::{Web3, Eth, EthFilter, EthSigning, Net, Parity, ParityAccounts, ParitySet, ParitySigning, Signer, Personal, Traces, Rpc};
pub use self::impls::*;
pub use self::helpers::{SigningQueue, SignerService, ConfirmationsQueue, NetworkSettings, block_import, informant, dispatch};
pub use self::metadata::Metadata;
pub use self::types::Origin;
