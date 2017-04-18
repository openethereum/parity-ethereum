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

/// Validator set changing at fork blocks.

use std::collections::BTreeMap;
use std::sync::Weak;
use engines::{Call, EpochChange};
use util::{H256, Address, RwLock, U256};
use ids::BlockId;
use header::{BlockNumber, Header};
use client::{Client, BlockChainClient};
use super::ValidatorSet;

type BlockNumberLookup = Box<Fn(BlockId) -> Result<BlockNumber, String> + Send + Sync + 'static>;

pub struct Multi {
	sets: BTreeMap<BlockNumber, Box<ValidatorSet>>,
	block_number: RwLock<BlockNumberLookup>,
}

impl Multi {
	pub fn new(set_map: BTreeMap<BlockNumber, Box<ValidatorSet>>) -> Self {
		assert!(set_map.get(&0u64).is_some(), "ValidatorSet has to be specified from block 0.");
		Multi {
			sets: set_map,
			block_number: RwLock::new(Box::new(move |_| Err("No client!".into()))),
		}
	}

	fn correct_set(&self, id: BlockId) -> Option<&ValidatorSet> {
		match self.block_number.read()(id).map(|parent_block| self.correct_set_by_number(parent_block)) {
			Ok((_, set)) => Some(set),
			Err(e) => {
				debug!(target: "engine", "ValidatorSet could not be recovered: {}", e);
				None
			},
		}
	}

	// get correct set by block number, along with block number at which
	// this set was activated.
	fn correct_set_by_number(&self, parent_block: BlockNumber) -> (BlockNumber, &ValidatorSet) {
		let (block, set) = self.sets.iter()
			.rev()
			.find(|&(block, _)| *block <= parent_block + 1)
			.expect("constructor validation ensures that there is at least one validator set for block 0;
					 block 0 is less than any uint;
					 qed");

		trace!(target: "engine", "Multi ValidatorSet retrieved for block {}.", block);
		(*block, &**set)
	}
}

impl ValidatorSet for Multi {
	fn default_caller(&self, block_id: BlockId) -> Box<Call> {
		self.correct_set(block_id).map(|set| set.default_caller(block_id))
			.unwrap_or(Box::new(|_, _| Err("No validator set for given ID.".into())))
	}

	fn is_epoch_end(&self, header: &Header, block: Option<&[u8]>, receipts: Option<&[::receipt::Receipt]>)
		-> EpochChange
	{
		self.correct_set_by_number(header.number()).1.is_epoch_end(header, block, receipts)
	}

	fn epoch_proof(&self, header: &Header, caller: &Call) -> Result<Vec<u8>, String> {
		self.correct_set_by_number(header.number()).1.epoch_proof(header, caller)
	}

	fn epoch_set(&self, header: &Header, proof: &[u8]) -> Result<(U256, super::SimpleList), ::error::Error> {
		// "multi" epoch is the inner set's epoch plus the transition block to that set.
		// ensures epoch increases monotonically.
		let (set_block, set) = self.correct_set_by_number(header.number());
		let (inner_epoch, list) = set.epoch_set(header, proof)?;
		Ok((U256::from(set_block) + inner_epoch, list))
	}

	fn contains_with_caller(&self, bh: &H256, address: &Address, caller: &Call) -> bool {
		self.correct_set(BlockId::Hash(*bh))
			.map_or(false, |set| set.contains_with_caller(bh, address, caller))
	}

	fn get_with_caller(&self, bh: &H256, nonce: usize, caller: &Call) -> Address {
		self.correct_set(BlockId::Hash(*bh))
			.map_or_else(Default::default, |set| set.get_with_caller(bh, nonce, caller))
	}

	fn count_with_caller(&self, bh: &H256, caller: &Call) -> usize {
		self.correct_set(BlockId::Hash(*bh))
			.map_or_else(usize::max_value, |set| set.count_with_caller(bh, caller))
	}

	fn report_malicious(&self, validator: &Address) {
		for set in self.sets.values() {
			set.report_malicious(validator);
		}
	}

	fn report_benign(&self, validator: &Address) {
		for set in self.sets.values() {
			set.report_benign(validator);
		}
	}

	fn register_contract(&self, client: Weak<Client>) {
		for set in self.sets.values() {
			set.register_contract(client.clone());
		}
		*self.block_number.write() = Box::new(move |id| client
			.upgrade()
			.ok_or("No client!".into())
			.and_then(|c| c.block_number(id).ok_or("Unknown block".into())));
	}
}

#[cfg(test)]
mod tests {
	use util::*;
	use types::ids::BlockId;
	use spec::Spec;
	use account_provider::AccountProvider;
	use client::{BlockChainClient, EngineClient};
	use ethkey::Secret;
	use miner::MinerService;
	use tests::helpers::{generate_dummy_client_with_spec_and_accounts, generate_dummy_client_with_spec_and_data};

	#[test]
	fn uses_current_set() {
		::env_logger::init().unwrap();
		let tap = Arc::new(AccountProvider::transient_provider());
		let s0 = Secret::from_slice(&"0".sha3()).unwrap();
		let v0 = tap.insert_account(s0.clone(), "").unwrap();
		let v1 = tap.insert_account(Secret::from_slice(&"1".sha3()).unwrap(), "").unwrap();
		let client = generate_dummy_client_with_spec_and_accounts(Spec::new_validator_multi, Some(tap));
		client.engine().register_client(Arc::downgrade(&client));

		// Make sure txs go through.
		client.miner().set_gas_floor_target(1_000_000.into());

		// Wrong signer for the first block.
		client.miner().set_engine_signer(v1, "".into()).unwrap();
		client.transact_contract(Default::default(), Default::default()).unwrap();
		client.update_sealing();
		assert_eq!(client.chain_info().best_block_number, 0);
		// Right signer for the first block.
		client.miner().set_engine_signer(v0, "".into()).unwrap();
		client.update_sealing();
		assert_eq!(client.chain_info().best_block_number, 1);
		// This time v0 is wrong.
		client.transact_contract(Default::default(), Default::default()).unwrap();
		client.update_sealing();
		assert_eq!(client.chain_info().best_block_number, 1);
		client.miner().set_engine_signer(v1, "".into()).unwrap();
		client.update_sealing();
		assert_eq!(client.chain_info().best_block_number, 2);
		// v1 is still good.
		client.transact_contract(Default::default(), Default::default()).unwrap();
		client.update_sealing();
		assert_eq!(client.chain_info().best_block_number, 3);

		// Check syncing.
		let sync_client = generate_dummy_client_with_spec_and_data(Spec::new_validator_multi, 0, 0, &[]);
		sync_client.engine().register_client(Arc::downgrade(&sync_client));
		for i in 1..4 {
			sync_client.import_block(client.block(BlockId::Number(i)).unwrap().into_inner()).unwrap();
		}
		sync_client.flush_queue();
		assert_eq!(sync_client.chain_info().best_block_number, 3);
	}
}
