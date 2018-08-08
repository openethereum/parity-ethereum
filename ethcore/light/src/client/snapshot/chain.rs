// Copyright 2015-2018 Parity Technologies (UK) Ltd.
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

//! This module contains a struct `LightChain`, which is
//! a wrapper around `HeaderChain` that keeps track of
//! pending changes made to it.
//!
//! The trait `RestorationTargetChain` is implemented for `HeaderChain`,
//! allowing the light client to restore from a snapshot (warp sync).


use super::super::HeaderChain;
use client::header_chain::PendingChanges;
use ethcore::engines::EpochTransition;
use ethcore::ids::BlockId;
use ethcore::header::{Header, BlockNumber};
use ethcore::encoded;
use ethcore::snapshot::RestorationTargetChain;
use ethereum_types::{H256, U256};
use parking_lot::RwLock;
use kvdb::DBTransaction;
use ethcore::receipt::Receipt;

/// Wrapper for `HeaderChain` along with pending changes.
pub struct LightChain {
	chain: HeaderChain,
	pending: RwLock<Option<PendingChanges>>
}

impl LightChain {
	/// Create a wrapper for `HeaderChain` implementing `RestorationTargetChain`
	pub fn new(chain: HeaderChain) -> Self {
		LightChain {
			chain: chain,
			pending: RwLock::new(None)
		}
	}

	/// Get a reference to the underlying chain
	pub fn chain(&self) -> &HeaderChain {
		&self.chain
	}
}

impl RestorationTargetChain for LightChain {
	fn genesis_hash(&self) -> H256 {
		self.chain.genesis_hash()
	}

	fn genesis_header(&self) -> Header {
		self.chain.genesis_header().decode().expect("genesis header is always decodable; qed")
	}

	fn block_hash(&self, index: BlockNumber) -> Option<H256> {
		self.chain.block_hash(BlockId::Number(index))
	}

	fn block_header_data(&self, hash: &H256) -> Option<Header> {
		self.chain.block_header(BlockId::Hash(hash.clone())).and_then(|h| h.decode().ok())
	}

	fn add_child(&self, _batch: &mut DBTransaction, _block_hash: H256, _child_hash: H256) {
		// We don't store parent <-> child relationship in the light client.
	}

	fn insert_epoch_transition(
		&self,
		batch: &mut DBTransaction,
		header: Header,
		transition: EpochTransition,
	) {
		let result = if header.number() == 0 {
			let td = self.chain.genesis_header().difficulty();
			self.chain.insert_with_td(batch, header, td, Some(transition.proof))
		} else {
			self.chain.insert(batch, header, Some(transition.proof))
		};
		let pending = result.expect("we either supply the total difficulty, or the parent is present; qed");
		*self.pending.write() = Some(pending);
	}

	fn insert_unordered_block(
		&self,
		batch: &mut DBTransaction,
		block: encoded::Block,
		_receipts: Vec<Receipt>,
		parent_td: Option<U256>,
		_is_best: bool,
		_is_ancient: bool,
	) -> bool {
		let td = parent_td.map(|pd| pd + block.header().difficulty());
		let result = self.chain.insert_inner(batch, block.decode_header(), td, None, false);
		let pending = result.expect("we either supply the total difficulty, or the parent is present; qed");
		*self.pending.write() = Some(pending);
		parent_td.is_some()
	}

	fn commit(&self) {
		if let Some(pending) = self.pending.write().take() {
			self.chain.apply_pending(pending)
		}
	}
}
