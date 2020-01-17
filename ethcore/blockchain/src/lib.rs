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

//! Blockchain database.

#![warn(missing_docs)]

extern crate parity_util_mem as util_mem;
extern crate parity_util_mem as malloc_size_of;

mod best_block;
mod blockchain;
mod cache;
mod config;
mod update;

pub mod generator;

pub use crate::{
	blockchain::{BlockProvider, BlockChain, BlockChainDB, BlockChainDBHandler},
	cache::CacheSize,
	config::Config,
	update::ExtrasInsert,
};
pub use ethcore_db::keys::{BlockReceipts, BlockDetails, TransactionAddress, BlockNumberKey};
pub use common_types::tree_route::TreeRoute;

