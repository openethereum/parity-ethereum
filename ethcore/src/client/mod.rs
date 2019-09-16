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
mod traits;

#[cfg(any(test, feature = "test-helpers"))]
mod evm_test_client;
#[cfg(any(test, feature = "test-helpers"))]
mod test_client;

pub use self::client::Client;
pub use self::config::{ClientConfig, DatabaseCompactionProfile, VMType};
pub use self::traits::{
    ReopenBlock, PrepareOpenBlock, ImportSealedBlock, BroadcastProposalBlock,
    Call, EngineInfo, BlockProducer, SealedBlockImporter,
};

#[cfg(any(test, feature = "test-helpers"))]
pub use self::evm_test_client::{EvmTestClient, EvmTestError, TransactErr, TransactSuccess};
#[cfg(any(test, feature = "test-helpers"))]
pub use self::test_client::{TestBlockChainClient, EachBlockWith, TestState};
