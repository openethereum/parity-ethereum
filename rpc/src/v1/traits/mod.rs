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

//! Ethereum rpc interfaces.

pub mod debug;
pub mod eth;
pub mod eth_pubsub;
pub mod eth_signing;
pub mod net;
pub mod parity;
pub mod parity_accounts;
pub mod parity_set;
pub mod parity_signing;
pub mod personal;
pub mod pubsub;
pub mod secretstore;
pub mod signer;
pub mod traces;
pub mod web3;

pub use self::{
    debug::Debug,
    eth::{Eth, EthFilter},
    eth_pubsub::EthPubSub,
    eth_signing::EthSigning,
    net::Net,
    parity::Parity,
    parity_accounts::{ParityAccounts, ParityAccountsInfo},
    parity_set::{ParitySet, ParitySetAccounts},
    parity_signing::ParitySigning,
    personal::Personal,
    pubsub::PubSub,
    secretstore::SecretStore,
    signer::Signer,
    traces::Traces,
    web3::Web3,
};
