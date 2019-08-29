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

//! Debug APIs RPC implementation

use std::sync::Arc;

use client_traits::BlockChainClient;
use types::header::Header;
use types::transaction::LocalizedTransaction;

use jsonrpc_core::Result;
use v1::traits::Debug;
use v1::types::{Block, Bytes, RichBlock, BlockTransactions, Transaction};

/// Debug rpc implementation.
pub struct DebugClient<C> {
	client: Arc<C>,
}

impl<C> DebugClient<C> {
	/// Creates new debug client.
	pub fn new(client: Arc<C>) -> Self {
		Self {
			client,
		}
	}
}

impl<C: BlockChainClient + 'static> Debug for DebugClient<C> {
	fn bad_blocks(&self) -> Result<Vec<RichBlock>> {
		fn cast<O, T: Copy + Into<O>>(t: &T) -> O {
			(*t).into()
		}

		Ok(self.client.bad_blocks().into_iter().map(|(block, reason)| {
			let number = block.header.number();
			let hash = block.header.hash();
			RichBlock {
				inner: Block {
					hash: Some(hash),
					size: Some(block.bytes.len().into()),
					parent_hash: cast(block.header.parent_hash()),
					uncles_hash: cast(block.header.uncles_hash()),
					author: cast(block.header.author()),
					miner: cast(block.header.author()),
					state_root: cast(block.header.state_root()),
					receipts_root: cast(block.header.receipts_root()),
					number: Some(number.into()),
					gas_used: cast(block.header.gas_used()),
					gas_limit: cast(block.header.gas_limit()),
					logs_bloom: Some(cast(block.header.log_bloom())),
					timestamp: block.header.timestamp().into(),
					difficulty: cast(block.header.difficulty()),
					total_difficulty: None,
					seal_fields: block.header.seal().iter().cloned().map(Into::into).collect(),
					uncles: block.uncles.iter().map(Header::hash).collect(),
					transactions: BlockTransactions::Full(block.transactions
						.into_iter()
						.enumerate()
						.map(|(transaction_index, signed)| Transaction::from_localized(LocalizedTransaction {
							block_number: number,
							block_hash: hash,
							transaction_index,
							signed,
							cached_sender: None,
						})).collect()
					),
					transactions_root: cast(block.header.transactions_root()),
					extra_data: block.header.extra_data().clone().into(),
				},
				extra_info: vec![
					("reason".to_owned(), reason),
					("rlp".to_owned(), serialize(&Bytes(block.bytes))),
					("hash".to_owned(), format!("{:#x}", hash)),
				].into_iter().collect(),
			}
		}).collect())
	}
}

fn serialize<T: ::serde::Serialize>(t: &T) -> String {
	::serde_json::to_string(t).expect("RPC types serialization is non-fallible.")
}
