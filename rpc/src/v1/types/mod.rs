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

//! RPC types

#[cfg(test)]
mod eth_types;

mod account_info;
mod block;
mod block_number;
mod bytes;
mod call_request;
mod confirmations;
mod consensus_status;
mod derivation;
mod eip191;
mod filter;
mod histogram;
mod index;
mod log;
mod node_kind;
mod provenance;
mod receipt;
mod rpc_settings;
mod secretstore;
mod sync;
mod trace;
mod trace_filter;
mod transaction;
mod transaction_condition;
mod transaction_request;
mod work;

pub mod pubsub;

pub use self::{
    account_info::{AccountInfo, EthAccount, ExtAccountInfo, RecoveredAccount, StorageProof},
    block::{Block, BlockTransactions, Header, Rich, RichBlock, RichHeader},
    block_number::{block_number_to_id, BlockNumber, LightBlockNumber},
    bytes::Bytes,
    call_request::CallRequest,
    confirmations::{
        ConfirmationPayload, ConfirmationRequest, ConfirmationResponse,
        ConfirmationResponseWithToken, DecryptRequest, EIP191SignRequest, Either, EthSignRequest,
        TransactionModification,
    },
    consensus_status::*,
    derivation::{Derive, DeriveHash, DeriveHierarchical},
    eip191::{EIP191Version, PresignedTransaction},
    filter::{Filter, FilterChanges},
    histogram::Histogram,
    index::Index,
    log::Log,
    node_kind::{Availability, Capability, NodeKind},
    provenance::Origin,
    receipt::Receipt,
    rpc_settings::RpcSettings,
    secretstore::EncryptedDocumentKey,
    sync::{
        ChainStatus, EthProtocolInfo, PeerInfo, PeerNetworkInfo, PeerProtocolsInfo, Peers,
        PipProtocolInfo, SyncInfo, SyncStatus, TransactionStats,
    },
    trace::{LocalizedTrace, TraceResults, TraceResultsWithTransactionHash},
    trace_filter::TraceFilter,
    transaction::{LocalTransactionStatus, RichRawTransaction, Transaction},
    transaction_condition::TransactionCondition,
    transaction_request::TransactionRequest,
    work::Work,
};

// TODO [ToDr] Refactor to a proper type Vec of enums?
/// Expected tracing type.
pub type TraceOptions = Vec<String>;
