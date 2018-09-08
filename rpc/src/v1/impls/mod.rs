// Copyright 2015-2018 Parity Technologies (UK) Ltd.
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

mod debug;
mod eth;
mod eth_filter;
mod eth_pubsub;
mod net;
mod parity;
mod parity_accounts;
mod parity_set;
mod personal;
mod private;
mod pubsub;
mod rpc;
mod secretstore;
mod signer;
mod signing;
mod signing_unsafe;
mod traces;
mod web3;

pub mod light;

pub use self::debug::DebugClient;
pub use self::eth::{EthClient, EthClientOptions};
pub use self::eth_filter::EthFilterClient;
pub use self::eth_pubsub::EthPubSubClient;
pub use self::net::NetClient;
pub use self::parity::ParityClient;
pub use self::parity_accounts::ParityAccountsClient;
pub use self::parity_set::ParitySetClient;
pub use self::personal::PersonalClient;
pub use self::private::PrivateClient;
pub use self::pubsub::PubSubClient;
pub use self::rpc::RpcClient;
pub use self::secretstore::SecretStoreClient;
pub use self::signer::SignerClient;
pub use self::signing::SigningQueueClient;
pub use self::signing_unsafe::SigningUnsafeClient;
pub use self::traces::TracesClient;
pub use self::web3::Web3Client;
