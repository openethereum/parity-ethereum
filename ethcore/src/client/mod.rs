// Copyright 2015-2017 Parity Technologies (UK) Ltd.
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

//! Blockchain database client.

mod ancient_import;
mod config;
mod error;
mod evm_test_client;
mod test_client;
mod trace;
mod client;

pub use self::client::*;
pub use self::config::{Mode, ClientConfig, DatabaseCompactionProfile, BlockChainConfig, VMType};
pub use self::error::Error;
pub use self::evm_test_client::{EvmTestClient, EvmTestError};
pub use self::test_client::{TestBlockChainClient, EachBlockWith};
pub use self::chain_notify::ChainNotify;
pub use self::traits::{BlockChainClient, MiningBlockChainClient, EngineClient};

pub use self::traits::ProvingBlockChainClient;

pub use types::ids::*;
pub use types::trace_filter::Filter as TraceFilter;
pub use types::pruning_info::PruningInfo;
pub use types::call_analytics::CallAnalytics;

pub use executive::{Executed, Executive, TransactOptions};
pub use vm::{LastHashes, EnvInfo};

pub use error::{BlockImportError, TransactionImportError, TransactionImportResult};
pub use verification::VerifierType;

/// IPC interfaces
#[cfg(feature="ipc")]
pub mod remote {
	pub use super::traits::RemoteClient;
	pub use super::chain_notify::ChainNotifyClient;
}

mod traits {
	#![allow(dead_code, unused_assignments, unused_variables, missing_docs)] // codegen issues
	include!(concat!(env!("OUT_DIR"), "/traits.rs"));
}

pub mod chain_notify {
	//! Chain notify interface
	#![allow(dead_code, unused_assignments, unused_variables, missing_docs)] // codegen issues
	include!(concat!(env!("OUT_DIR"), "/chain_notify.rs"));
}

