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

//! Helpers for submit a POW work.

use std::sync::Arc;

use rlp;
use ethcore::miner::{BlockChainClient, MinerService};
use ethereum_types::{H64, H256};
use jsonrpc_core::Error;
use v1::helpers::errors;

// Submit a POW work and return the block's hash
pub fn submit_work_detail<C: BlockChainClient, M: MinerService>(client: &Arc<C>, miner: &Arc<M>, nonce: H64, pow_hash: H256, mix_hash: H256) -> Result<H256, Error> {
	// TODO [ToDr] Should disallow submissions in case of PoA?
	trace!(target: "miner", "submit_work_detail: Decoded: nonce={}, pow_hash={}, mix_hash={}", nonce, pow_hash, mix_hash);
	let seal = vec![rlp::encode(&mix_hash), rlp::encode(&nonce)];
	miner.submit_seal(pow_hash, seal)
		.and_then(|block| client.import_sealed_block(block))
		.map_err(|e| {
			warn!(target: "miner", "Cannot submit work - {:?}.", e);
			errors::cannot_submit_work(e)
		})
}
