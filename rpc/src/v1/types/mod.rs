// Copyright 2015-2020 Parity Technologies (UK) Ltd.
// This file is part of Open Ethereum.

// Open Ethereum is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Open Ethereum is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Open Ethereum.  If not, see <http://www.gnu.org/licenses/>.

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
mod filter;
mod histogram;
mod index;
mod log;
mod node_kind;
mod private_receipt;
mod private_log;
mod provenance;
mod receipt;
mod rpc_settings;
mod secretstore;
mod sync;
mod trace;
mod trace_filter;
mod transaction;
mod transaction_request;
mod transaction_condition;
mod work;
mod eip191;

pub mod pubsub;

pub use self::eip191::{EIP191Version, PresignedTransaction};
pub use self::account_info::{AccountInfo, ExtAccountInfo, EthAccount, StorageProof, RecoveredAccount};
pub use self::bytes::Bytes;
pub use self::block::{RichBlock, Block, BlockTransactions, Header, RichHeader, Rich};
pub use self::block_number::{BlockNumber, LightBlockNumber, block_number_to_id};
pub use self::call_request::CallRequest;
pub use self::confirmations::{
	ConfirmationPayload, ConfirmationRequest, ConfirmationResponse, ConfirmationResponseWithToken,
	TransactionModification, EIP191SignRequest, EthSignRequest, DecryptRequest, Either
};
pub use self::consensus_status::*;
pub use self::derivation::{DeriveHash, DeriveHierarchical, Derive};
pub use self::filter::{Filter, FilterChanges};
pub use self::histogram::Histogram;
pub use self::index::Index;
pub use self::log::Log;
pub use self::node_kind::{NodeKind, Availability, Capability};
pub use self::private_receipt::{PrivateTransactionReceipt, PrivateTransactionReceiptAndTransaction};
pub use self::private_log::PrivateTransactionLog;
pub use self::provenance::Origin;
pub use self::receipt::Receipt;
pub use self::rpc_settings::RpcSettings;
pub use self::secretstore::EncryptedDocumentKey;
pub use self::sync::{
	SyncStatus, SyncInfo, Peers, PeerInfo, PeerNetworkInfo, PeerProtocolsInfo,
	TransactionStats, ChainStatus, EthProtocolInfo, PipProtocolInfo,
};
pub use self::trace::{LocalizedTrace, TraceResults, TraceResultsWithTransactionHash};
pub use self::trace_filter::TraceFilter;
pub use self::transaction::{Transaction, RichRawTransaction, LocalTransactionStatus};
pub use self::transaction_request::TransactionRequest;
pub use self::transaction_condition::TransactionCondition;
pub use self::work::Work;

// TODO [ToDr] Refactor to a proper type Vec of enums?
/// Expected tracing type.
pub type TraceOptions = Vec<String>;
