// Copyright 2015-2020 Parity Technologies (UK) Ltd.
// This file is part of OpenEthereum.

// OpenEthereum is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// OpenEthereum is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with OpenEthereum.  If not, see <http://www.gnu.org/licenses/>.

//! Ethcore rpc v1.
//!
//! Compliant with ethereum rpc.

// short for "try_boxfuture"
// unwrap a result, returning a BoxFuture<_, Err> on failure.
macro_rules! try_bf {
    ($res: expr) => {
        match $res {
            Ok(val) => val,
            Err(e) => return Box::new(::jsonrpc_core::futures::future::err(e.into())),
        }
    };
}

#[macro_use]
mod helpers;
mod impls;
#[cfg(test)]
mod tests;
mod types;

pub mod extractors;
pub mod informant;
pub mod metadata;
pub mod traits;

pub use self::{
    extractors::{RpcExtractor, WsDispatcher, WsExtractor, WsStats},
    helpers::{block_import, dispatch, NetworkSettings},
    impls::*,
    metadata::Metadata,
    traits::{
        Debug, Eth, EthFilter, EthPubSub, EthSigning, Net, Parity, ParityAccounts,
        ParityAccountsInfo, ParitySet, ParitySetAccounts, ParitySigning, Personal, PubSub,
        SecretStore, Signer, Traces, Web3,
    },
    types::Origin,
};

/// Signer utilities
pub mod signer {
    #[cfg(any(test, feature = "accounts"))]
    pub use super::helpers::engine_signer::EngineSigner;
    pub use super::{
        helpers::external_signer::{ConfirmationsQueue, SignerService},
        types::{ConfirmationRequest, TransactionCondition, TransactionModification},
    };
}
