// Copyright 2015-2019 Parity Technologies (UK) Ltd.
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

//! Ethereum rpc interface implementation.

mod debug;
mod eth;
mod eth_filter;
mod eth_pubsub;
mod net;
mod parity;
#[cfg(any(test, feature = "accounts"))]
mod parity_accounts;
mod parity_set;
#[cfg(any(test, feature = "accounts"))]
mod personal;
mod pubsub;
#[cfg(any(test, feature = "accounts"))]
mod secretstore;
mod signer;
mod signing;
mod signing_unsafe;
mod traces;
mod web3;

pub mod light;

#[cfg(any(test, feature = "accounts"))]
pub use self::parity_accounts::ParityAccountsClient;
#[cfg(any(test, feature = "accounts"))]
pub use self::parity_set::accounts::ParitySetAccountsClient;
#[cfg(any(test, feature = "accounts"))]
pub use self::personal::PersonalClient;
#[cfg(any(test, feature = "accounts"))]
pub use self::secretstore::SecretStoreClient;
pub use self::{
    debug::DebugClient,
    eth::{EthClient, EthClientOptions},
    eth_filter::EthFilterClient,
    eth_pubsub::EthPubSubClient,
    net::NetClient,
    parity::ParityClient,
    parity_set::ParitySetClient,
    pubsub::PubSubClient,
    signer::SignerClient,
    signing::SigningQueueClient,
    signing_unsafe::SigningUnsafeClient,
    traces::TracesClient,
    web3::Web3Client,
};
