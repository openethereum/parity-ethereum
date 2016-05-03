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

//! Ethereum rpc interface implementation.

macro_rules! take_weak {
	($weak: expr) => {
		match $weak.upgrade() {
			Some(arc) => arc,
			None => return Err(Error::internal_error())
		}
	}
}

mod web3;
mod eth;
mod net;
mod personal;
mod ethcore;
mod traces;
mod rpc;

pub use self::web3::Web3Client;
pub use self::eth::{EthClient, EthFilterClient};
pub use self::net::NetClient;
pub use self::personal::PersonalClient;
pub use self::ethcore::EthcoreClient;
pub use self::traces::TracesClient;
pub use self::rpc::RpcClient;

