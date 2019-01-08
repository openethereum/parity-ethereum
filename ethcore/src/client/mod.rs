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

//! Blockchain database client.

mod ancient_import;
mod bad_blocks;
mod client;
mod config;
#[cfg(any(test, feature = "test-helpers"))]
mod evm_test_client;
mod io_message;
#[cfg(any(test, feature = "test-helpers"))]
mod test_client;
mod trace;

pub use self::client::*;
pub use self::config::{Mode, ClientConfig, DatabaseCompactionProfile, BlockChainConfig, VMType};
#[cfg(any(test, feature = "test-helpers"))]
pub use self::evm_test_client::{EvmTestClient, EvmTestError, TransactResult};
pub use self::io_message::ClientIoMessage;
#[cfg(any(test, feature = "test-helpers"))]
pub use self::test_client::{TestBlockChainClient, EachBlockWith};
pub use self::chain_notify::{ChainNotify, NewBlocks, ChainRoute, ChainRouteType, ChainMessageType};
pub use self::traits::{
    Nonce, Balance, ChainInfo, BlockInfo, ReopenBlock, PrepareOpenBlock, CallContract, TransactionInfo, RegistryInfo, ScheduleInfo, ImportSealedBlock, BroadcastProposalBlock, ImportBlock,
    StateOrBlock, StateClient, Call, EngineInfo, AccountData, BlockChain, BlockProducer, SealedBlockImporter, BadBlocks,
};
pub use state::StateInfo;
pub use self::traits::{BlockChainClient, EngineClient, ProvingBlockChainClient, IoClient};

pub use types::ids::*;
pub use types::trace_filter::Filter as TraceFilter;
pub use types::pruning_info::PruningInfo;
pub use types::call_analytics::CallAnalytics;

pub use executive::{Executed, Executive, TransactOptions};
pub use vm::{LastHashes, EnvInfo};

pub use error::TransactionImportError;
pub use verification::VerifierType;

mod traits;

mod chain_notify;
mod private_notify;
