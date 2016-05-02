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

//! Ethereum rpc interfaces.

macro_rules! rpc_unimplemented {
	() => (Err(Error::internal_error()))
}

pub mod web3;
pub mod eth;
pub mod net;
pub mod personal;
pub mod ethcore;
pub mod traces;
pub mod rpc;

pub use self::web3::Web3;
pub use self::eth::{Eth, EthFilter};
pub use self::net::Net;
pub use self::personal::Personal;
pub use self::ethcore::Ethcore;
pub use self::traces::Traces;
pub use self::rpc::Rpc;
