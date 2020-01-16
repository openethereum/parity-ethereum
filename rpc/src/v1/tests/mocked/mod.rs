// Copyright 2015-2020 Parity Technologies (UK) Ltd.
// This file is part of Parity Ethereum.

// Parity Ethereum is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity Ethereum is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity Ethereum.  If not, see <http://www.gnu.org/licenses/>.

//! RPC mocked tests. Most of these test that the RPC server is serializing and forwarding
//! method calls properly.

mod debug;
mod eth;
mod eth_pubsub;
mod manage_network;
mod net;
mod parity;
#[cfg(any(test, feature = "accounts"))]
mod parity_accounts;
mod parity_set;
#[cfg(any(test, feature = "accounts"))]
mod personal;
mod pubsub;
mod rpc;
#[cfg(any(test, feature = "accounts"))]
mod secretstore;
mod signer;
#[cfg(any(test, feature = "accounts"))]
mod signing;
#[cfg(any(test, feature = "accounts"))]
mod signing_unsafe;
mod traces;
mod web3;
