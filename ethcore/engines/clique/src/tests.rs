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

//! Consensus tests for `PoA Clique Engine`, see http://eips.ethereum.org/EIPS/eip-225 for more information

use std::sync::Arc;
use std::collections::HashMap;

use common_types::errors::{EthcoreError as Error, EngineError};
use ethcore::{
	block::*,
	test_helpers::get_temp_state_db,
};
use engine::Engine;
use ethereum_types::{Address, H256};
use parity_crypto::publickey::{Secret, KeyPair};
use state_db::StateDB;

use super::*;

/// Possible signers
pub const SIGNER_TAGS: [char; 6] = ['A', 'B', 'C', 'D', 'E', 'F'];

/// Clique block types
pub enum CliqueBlockType {
	/// Epoch transition block must contain list of signers
	Checkpoint,
	/// Block with no votes
	Empty,
	/// Vote
	Vote(VoteType),
}

/// Clique tester
pub struct CliqueTester {
	/// Mocked Clique
	pub clique: Clique,
	/// Mocked genesis state
	pub genesis: Header,
	/// StateDB
	pub db: StateDB,
	/// List of signers
	pub signers: HashMap<char, KeyPair>,
}

impl CliqueTester {
	/// Create a `Clique` tester with settings
	pub fn with(epoch: u64, period: u64, initial_signers: Vec<char>) -> Self {
		assert_eq!(initial_signers.iter().all(|s| SIGNER_TAGS.contains(s)), true,
				   "Not all the initial signers is in SIGNER_TAGS, possible keys are 'A' ..= 'F'");

		let clique = Clique::with_test(epoch, period);
		let mut genesis = Header::default();
		let mut signers = HashMap::new();

		let call = |_a, _b| {
			unimplemented!("Clique doesn't use Engine::Call");
		};

		let mut extra_data = vec![0; VANITY_LENGTH];

		for &signer in SIGNER_TAGS.iter() {
			let secret = Secret::from(H256::from_low_u64_be(signer as u64));
			let keypair = KeyPair::from_secret(secret).unwrap();
			if initial_signers.contains(&signer) {
				extra_data.extend(keypair.address().as_bytes());
			}
			signers.insert(signer, keypair);
		}

		// append dummy signature
		extra_data.extend(std::iter::repeat(0).take(SIGNATURE_LENGTH));

		genesis.set_extra_data(extra_data);
		genesis.set_gas_limit(U256::from(0xa00000));
		genesis.set_difficulty(U256::from(1));
		genesis.set_seal(util::null_seal());

		clique.genesis_epoch_data(&genesis, &call).expect("Create genesis failed");
		Self {clique, genesis, db: get_temp_state_db(), signers}
	}

	/// Get difficulty for a given block
	pub fn get_difficulty(&self, block_num: BlockNumber, header: &Header, signer: &Address) -> U256 {
		let state = self.clique.state(header).unwrap();
		if state.is_inturn(block_num, signer) {
			DIFF_INTURN
		} else {
			DIFF_NOTURN
		}
	}

	/// Get the state of a given block
	// Note, this will read the cache and `will` not work with more than 128 blocks
	pub fn get_state_at_block(&self, hash: &H256) -> CliqueBlockState {
		self.clique.block_state_by_hash.write()
			.get_mut(hash)
			.expect("CliqueBlockState not found tested failed")
			.clone()
	}

	/// Get signers after a certain state
	// This is generally used to fetch the state after a test has been executed and checked against
	// the initial list of signers provided in the test
	pub fn clique_signers(&self, hash: &H256) -> impl Iterator<Item = Address> {
		self.get_state_at_block(hash).signers().clone().into_iter()
	}

	/// Fetches all addresses at current `block` and converts them back to `tags (char)` and sorts them
	/// Addresses are supposed sorted based on address but these tests are using `tags` just for simplicity
	/// and the order is not important!
	pub fn into_tags<T: Iterator<Item = Address>>(&self, addr: T) -> Vec<char> {
		let mut tags: Vec<char> = addr.filter_map(|addr| {
			for (t, kp) in self.signers.iter() {
				if addr == kp.address() {
					return Some(*t)
				}
			}
			None
		})
		.collect();

		tags.sort();
		tags
	}

	/// Create a new `Clique` block and import
	pub fn new_block_and_import(
		&self,
		block_type: CliqueBlockType,
		last_header: &Header,
		beneficary: Option<Address>,
		signer: char,
		) -> Result<Header, Error> {

		let mut extra_data = vec![0; VANITY_LENGTH];
		let mut seal = util::null_seal();
		let last_hash = last_header.hash();

		match block_type {
			CliqueBlockType::Checkpoint => {
				let signers = self.clique.state(&last_header).unwrap().signers().clone();
				for signer in signers {
					extra_data.extend(signer.as_bytes());
				}
			}
			CliqueBlockType::Vote(v) => seal = v.as_rlp(),
			CliqueBlockType::Empty => (),
		};

		let db = self.db.boxed_clone();

		let mut block = OpenBlock::new(
			&self.clique,
			Default::default(),
			false,
			db,
			&last_header.clone(),
			Arc::new(vec![last_hash]),
			beneficary.unwrap_or_default(),
			(3141562.into(), 31415620.into()),
			extra_data,
			false,
		).unwrap();

		{
			let difficulty = self.get_difficulty(block.header.number(), last_header, &self.signers[&signer].address());
			let b = block.block_mut();
			b.header.set_timestamp(last_header.timestamp() + self.clique.period);
			b.header.set_difficulty(difficulty);
			b.header.set_seal(seal);

			let sign = parity_crypto::publickey::sign(self.signers[&signer].secret(), &b.header.hash()).unwrap();
			let mut extra_data = b.header.extra_data().clone();
			extra_data.extend_from_slice(&*sign);
			b.header.set_extra_data(extra_data);
		}

		let current_header = &block.header;
		self.clique.verify_block_basic(current_header)?;
		self.clique.verify_block_family(current_header, &last_header)?;

		Ok(current_header.clone())
	}
}

#[test]
fn one_signer_with_no_votes() {
	let tester = CliqueTester::with(10, 1, vec!['A']);

	let empty_block = tester.new_block_and_import(CliqueBlockType::Empty, &tester.genesis, None, 'A').unwrap();

	let tags = tester.into_tags(tester.clique_signers(&empty_block.hash()));
	assert_eq!(&tags, &['A']);
}

#[test]
fn one_signer_two_votes() {
	let tester = CliqueTester::with(10, 1, vec!['A']);

	// Add a vote for `B` signed by `A`
	let vote = tester.new_block_and_import(CliqueBlockType::Vote(VoteType::Add), &tester.genesis,
										   Some(tester.signers[&'B'].address()), 'A').unwrap();
	let tags = tester.into_tags(tester.clique_signers(&vote.hash()));
	assert_eq!(&tags, &['A', 'B']);

	// Add a empty block signed by `B`
	let empty = tester.new_block_and_import(CliqueBlockType::Empty, &vote, None, 'B').unwrap();

	// Add vote for `C` signed by A but should not be accepted
	let vote = tester.new_block_and_import(CliqueBlockType::Vote(VoteType::Add), &empty,
										   Some(tester.signers[&'C'].address()), 'A').unwrap();

	let tags = tester.into_tags(tester.clique_signers(&vote.hash()));
	assert_eq!(&tags, &['A', 'B']);
}

#[test]
fn two_signers_six_votes_deny_last() {
	let tester = CliqueTester::with(10, 1, vec!['A', 'B']);

	let mut prev_header = tester.genesis.clone();

	// Add two votes for `C` signed by `A` and `B`
	for &signer in SIGNER_TAGS.iter().take(2) {
		let vote = tester.new_block_and_import(CliqueBlockType::Vote(VoteType::Add), &prev_header,
											   Some(tester.signers[&'C'].address()), signer).unwrap();
		prev_header = vote.clone();
	}

	// Add two votes for `D` signed by `A` and `B`
	for &signer in SIGNER_TAGS.iter().take(2) {
		let vote = tester.new_block_and_import(CliqueBlockType::Vote(VoteType::Add), &prev_header,
											   Some(tester.signers[&'D'].address()), signer).unwrap();
		prev_header = vote.clone();
	}

	// Add a empty block signed by `C`
	let empty = tester.new_block_and_import(CliqueBlockType::Empty, &prev_header, None, 'C').unwrap();
	prev_header = empty.clone();

	// Add two votes for `E` signed by `A` and `B`
	for &signer in SIGNER_TAGS.iter().take(2) {
		let vote = tester.new_block_and_import(CliqueBlockType::Vote(VoteType::Add), &prev_header,
											   Some(tester.signers[&'E'].address()), signer).unwrap();
		prev_header = vote.clone();
	}

	let tags = tester.into_tags(tester.clique_signers(&prev_header.hash()));
	assert_eq!(&tags, &['A', 'B', 'C', 'D']);
}

#[test]
fn one_signer_dropping_itself() {
	let tester = CliqueTester::with(10, 1, vec!['A']);
	let vote = tester.new_block_and_import(CliqueBlockType::Vote(VoteType::Remove), &tester.genesis,
										   Some(tester.signers[&'A'].address()), 'A').unwrap();
	let signers = tester.clique_signers(&vote.hash());
	assert!(signers.count() == 0);
}

#[test]
fn two_signers_one_remove_vote_no_consensus() {
	let tester = CliqueTester::with(10, 1, vec!['A', 'B']);
	let vote = tester.new_block_and_import(CliqueBlockType::Vote(VoteType::Remove), &tester.genesis,
										   Some(tester.signers[&'B'].address()), 'A').unwrap();

	let tags = tester.into_tags(tester.clique_signers(&vote.hash()));
	assert_eq!(&tags, &['A', 'B']);
}

#[test]
fn two_signers_consensus_remove_b() {
	let tester = CliqueTester::with(10, 1, vec!['A', 'B']);
	let first_vote = tester.new_block_and_import(CliqueBlockType::Vote(VoteType::Remove), &tester.genesis,
												 Some(tester.signers[&'B'].address()), 'A').unwrap();
	let second_vote = tester.new_block_and_import(CliqueBlockType::Vote(VoteType::Remove), &first_vote,
												 Some(tester.signers[&'B'].address()), 'B').unwrap();

	let tags = tester.into_tags(tester.clique_signers(&second_vote.hash()));
	assert_eq!(&tags, &['A']);
}

#[test]
fn three_signers_consensus_remove_c() {
	let tester = CliqueTester::with(10, 1, vec!['A', 'B', 'C']);
	let first_vote = tester.new_block_and_import(CliqueBlockType::Vote(VoteType::Remove), &tester.genesis,
												 Some(tester.signers[&'C'].address()), 'A').unwrap();
	let second_vote = tester.new_block_and_import(CliqueBlockType::Vote(VoteType::Remove), &first_vote,
												 Some(tester.signers[&'C'].address()), 'B').unwrap();

	let tags = tester.into_tags(tester.clique_signers(&second_vote.hash()));
	assert_eq!(&tags, &['A', 'B']);
}

#[test]
fn four_signers_half_no_consensus() {
	let tester = CliqueTester::with(10, 1, vec!['A', 'B', 'C', 'D']);
	let first_vote = tester.new_block_and_import(CliqueBlockType::Vote(VoteType::Remove), &tester.genesis,
												 Some(tester.signers[&'C'].address()), 'A').unwrap();

	let second_vote = tester.new_block_and_import(CliqueBlockType::Vote(VoteType::Remove), &first_vote,
												  Some(tester.signers[&'C'].address()), 'B').unwrap();

	let tags = tester.into_tags(tester.clique_signers(&second_vote.hash()));
	assert_eq!(&tags, &['A', 'B', 'C', 'D']);
}

#[test]
fn four_signers_three_consensus_rm() {
	let tester = CliqueTester::with(10, 1, vec!['A', 'B', 'C', 'D']);

	let mut prev_header = tester.genesis.clone();

	// Three votes to remove `D` signed by ['A', 'B', 'C']
	for signer in SIGNER_TAGS.iter().take(3) {
		let vote = tester.new_block_and_import(CliqueBlockType::Vote(VoteType::Remove), &prev_header,
											   Some(tester.signers[&'D'].address()), *signer).unwrap();
		prev_header = vote.clone();
	}

	let tags = tester.into_tags(tester.clique_signers(&prev_header.hash()));
	assert_eq!(&tags, &['A', 'B', 'C']);
}

#[test]
fn vote_add_only_counted_once_per_signer() {
	let tester = CliqueTester::with(10, 1, vec!['A', 'B']);

	// Add a vote for `C` signed by `A`
	let vote = tester.new_block_and_import(CliqueBlockType::Vote(VoteType::Add), &tester.genesis,
										   Some(tester.signers[&'C'].address()), 'A').unwrap();
	// Empty block signed by B`
	let empty = tester.new_block_and_import(CliqueBlockType::Empty, &vote, None, 'B').unwrap();

	// Add a vote for `C` signed by `A`
	let vote = tester.new_block_and_import(CliqueBlockType::Vote(VoteType::Add), &empty,
										   Some(tester.signers[&'C'].address()), 'A').unwrap();
	// Empty block signed by `B`
	let empty = tester.new_block_and_import(CliqueBlockType::Empty, &vote, None, 'B').unwrap();

	// Add a vote for `C` signed by `A`
	let vote = tester.new_block_and_import(CliqueBlockType::Vote(VoteType::Add), &empty,
										   Some(tester.signers[&'C'].address()), 'A').unwrap();

	let tags = tester.into_tags(tester.clique_signers(&vote.hash()));
	assert_eq!(&tags, &['A', 'B']);
}

#[test]
fn vote_add_concurrently_is_permitted() {
	let tester = CliqueTester::with(10, 1, vec!['A', 'B']);

	// Add a vote for `C` signed by `A`
	let b = tester.new_block_and_import(CliqueBlockType::Vote(VoteType::Add), &tester.genesis,
										   Some(tester.signers[&'C'].address()), 'A').unwrap();

	// Empty block signed by `B`
	let b = tester.new_block_and_import(CliqueBlockType::Empty, &b, None, 'B').unwrap();

	// Add a vote for `D` signed by `A`
	let b = tester.new_block_and_import(CliqueBlockType::Vote(VoteType::Add), &b,
										   Some(tester.signers[&'D'].address()), 'A').unwrap();

	// Empty block signed by `B`
	let b = tester.new_block_and_import(CliqueBlockType::Empty, &b, None, 'B').unwrap();

	// Empty block signed by `A`
	let b = tester.new_block_and_import(CliqueBlockType::Empty, &b, None, 'A').unwrap();

	// Add a vote for `D` signed by `B`
	let b = tester.new_block_and_import(CliqueBlockType::Vote(VoteType::Add), &b,
										   Some(tester.signers[&'D'].address()), 'B').unwrap();

	// Empty block signed by `A`
	let b = tester.new_block_and_import(CliqueBlockType::Empty, &b, None, 'A').unwrap();

	// Add a vote for `C` signed by `B`
	let b = tester.new_block_and_import(CliqueBlockType::Vote(VoteType::Add), &b,
										   Some(tester.signers[&'C'].address()), 'B').unwrap();

	let tags = tester.into_tags(tester.clique_signers(&b.hash()));
	assert_eq!(&tags, &['A', 'B', 'C', 'D']);
}

#[test]
fn vote_rm_only_counted_once_per_signer() {
	let tester = CliqueTester::with(10, 1, vec!['A', 'B']);

	let mut prev_header = tester.genesis.clone();

	for _ in 0..2 {
		// Vote to remove `B` signed by `A`
		let b = tester.new_block_and_import(CliqueBlockType::Vote(VoteType::Remove), &prev_header,
											   Some(tester.signers[&'B'].address()), 'A').unwrap();
		// Empty block signed by `B`
		let b = tester.new_block_and_import(CliqueBlockType::Empty, &b, None, 'B').unwrap();

		prev_header = b.clone();
	}

	// Add a vote for `B` signed by `A`
	let b = tester.new_block_and_import(CliqueBlockType::Vote(VoteType::Remove), &prev_header,
										   Some(tester.signers[&'B'].address()), 'A').unwrap();

	let tags = tester.into_tags(tester.clique_signers(&b.hash()));
	assert_eq!(&tags, &['A', 'B']);
}

#[test]
fn vote_rm_concurrently_is_permitted() {
	let tester = CliqueTester::with(100, 1, vec!['A', 'B', 'C', 'D']);

	// Add a vote for `C` signed by `A`
	let block = tester.new_block_and_import(CliqueBlockType::Vote(VoteType::Remove), &tester.genesis,
											Some(tester.signers[&'C'].address()), 'A').unwrap();

	// Empty block signed by `B`
	let block = tester.new_block_and_import(CliqueBlockType::Empty, &block, None, 'B').unwrap();
	// Empty block signed by `C`
	let block = tester.new_block_and_import(CliqueBlockType::Empty, &block, None, 'C').unwrap();

	// Add a vote for `D` signed by `A`
	let block = tester.new_block_and_import(CliqueBlockType::Vote(VoteType::Remove), &block,
											Some(tester.signers[&'D'].address()), 'A').unwrap();

	// Empty block signed by `B`
	let block = tester.new_block_and_import(CliqueBlockType::Empty, &block, None, 'B').unwrap();
	// Empty block signed by `C`
	let block = tester.new_block_and_import(CliqueBlockType::Empty, &block, None, 'C').unwrap();
	// Empty block signed by `A`
	let block = tester.new_block_and_import(CliqueBlockType::Empty, &block, None, 'A').unwrap();

	// Add a vote for `D` signed by `B`
	let block = tester.new_block_and_import(CliqueBlockType::Vote(VoteType::Remove), &block,
											Some(tester.signers[&'D'].address()), 'B').unwrap();
	// Add a vote for `D` signed by `C`
	let block = tester.new_block_and_import(CliqueBlockType::Vote(VoteType::Remove), &block,
											Some(tester.signers[&'D'].address()), 'C').unwrap();

	// Empty block signed by `A`
	let block = tester.new_block_and_import(CliqueBlockType::Empty, &block, None, 'A').unwrap();
	// Add a vote for `C` signed by `B`
	let block = tester.new_block_and_import(CliqueBlockType::Vote(VoteType::Remove), &block,
											Some(tester.signers[&'C'].address()), 'B').unwrap();

	let tags = tester.into_tags(tester.clique_signers(&block.hash()));
	assert_eq!(&tags, &['A', 'B']);
}

#[test]
fn vote_to_rm_are_immediate_and_ensure_votes_are_rm() {
	let tester = CliqueTester::with(100, 1, vec!['A', 'B', 'C']);

	// Vote to remove `B` signed by `C`
	let block = tester.new_block_and_import(CliqueBlockType::Vote(VoteType::Remove), &tester.genesis,
											Some(tester.signers[&'B'].address()), 'C').unwrap();
	// Vote to remove `C` signed by `A`
	let block = tester.new_block_and_import(CliqueBlockType::Vote(VoteType::Remove), &block,
											Some(tester.signers[&'C'].address()), 'A').unwrap();
	// Vote to remove `C` signed by `B`
	let block = tester.new_block_and_import(CliqueBlockType::Vote(VoteType::Remove), &block,
											Some(tester.signers[&'C'].address()), 'B').unwrap();
	// Vote to remove `B` signed by `A`
	let block = tester.new_block_and_import(CliqueBlockType::Vote(VoteType::Remove), &block,
											Some(tester.signers[&'B'].address()), 'A').unwrap();

	let tags = tester.into_tags(tester.clique_signers(&block.hash()));
	assert_eq!(&tags, &['A', 'B']);
}

#[test]
fn vote_to_rm_are_immediate_and_votes_should_be_dropped_from_kicked_signer() {
	let tester = CliqueTester::with(100, 1, vec!['A', 'B', 'C']);

	// Vote to add `D` signed by `C`
	let block = tester.new_block_and_import(CliqueBlockType::Vote(VoteType::Add), &tester.genesis,
											Some(tester.signers[&'D'].address()), 'C').unwrap();
	// Vote to remove `C` signed by `A`
	let block = tester.new_block_and_import(CliqueBlockType::Vote(VoteType::Remove), &block,
											Some(tester.signers[&'C'].address()), 'A').unwrap();

	// Vote to remove `C` signed by `B`
	let block = tester.new_block_and_import(CliqueBlockType::Vote(VoteType::Remove), &block,
											Some(tester.signers[&'C'].address()), 'B').unwrap();

	// Vote to add `D` signed by `A`
	let block = tester.new_block_and_import(CliqueBlockType::Vote(VoteType::Add), &block,
											Some(tester.signers[&'D'].address()), 'A').unwrap();

	let tags = tester.into_tags(tester.clique_signers(&block.hash()));
	assert_eq!(&tags, &['A', 'B']);
}

#[test]
fn cascading_not_allowed() {
	let tester = CliqueTester::with(100, 1, vec!['A', 'B', 'C', 'D']);

	// Vote against `C` signed by `A`
	let block = tester.new_block_and_import(CliqueBlockType::Vote(VoteType::Remove), &tester.genesis,
											Some(tester.signers[&'C'].address()), 'A').unwrap();

	// Empty block signed by `B`
	let block = tester.new_block_and_import(CliqueBlockType::Empty, &block, None, 'B').unwrap();

	// Empty block signed by `C`
	let block = tester.new_block_and_import(CliqueBlockType::Empty, &block, None, 'C').unwrap();

	// Vote against `D` signed by `A`
	let block = tester.new_block_and_import(CliqueBlockType::Vote(VoteType::Remove), &block,
											Some(tester.signers[&'D'].address()), 'A').unwrap();

	// Vote against `C` signed by `B`
	let block = tester.new_block_and_import(CliqueBlockType::Vote(VoteType::Remove), &block,
											Some(tester.signers[&'C'].address()), 'B').unwrap();

	// Empty block signed by `C`
	let block = tester.new_block_and_import(CliqueBlockType::Empty, &block, None, 'C').unwrap();

	// Empty block signed by `A`
	let block = tester.new_block_and_import(CliqueBlockType::Empty, &block, None, 'A').unwrap();

	// Vote against `D` signed by `B`
	let block = tester.new_block_and_import(CliqueBlockType::Vote(VoteType::Remove), &block,
											Some(tester.signers[&'D'].address()), 'B').unwrap();

	// Vote against `D` signed by `C`
	let block = tester.new_block_and_import(CliqueBlockType::Vote(VoteType::Remove), &block,
											Some(tester.signers[&'D'].address()), 'C').unwrap();

	let tags = tester.into_tags(tester.clique_signers(&block.hash()));
	assert_eq!(&tags, &['A', 'B', 'C']);
}

#[test]
fn consensus_out_of_bounds_consensus_execute_on_touch() {
	let tester = CliqueTester::with(100, 1, vec!['A', 'B', 'C', 'D']);

	// Vote against `C` signed by `A`
	let block = tester.new_block_and_import(CliqueBlockType::Vote(VoteType::Remove), &tester.genesis,
											Some(tester.signers[&'C'].address()), 'A').unwrap();

	// Empty block signed by `B`
	let block = tester.new_block_and_import(CliqueBlockType::Empty, &block, None, 'B').unwrap();

	// Empty block signed by `C`
	let block = tester.new_block_and_import(CliqueBlockType::Empty, &block, None, 'C').unwrap();

	// Vote against `D` signed by `A`
	let block = tester.new_block_and_import(CliqueBlockType::Vote(VoteType::Remove), &block,
											Some(tester.signers[&'D'].address()), 'A').unwrap();

	// Vote against `C` signed by `B`
	let block = tester.new_block_and_import(CliqueBlockType::Vote(VoteType::Remove), &block,
											Some(tester.signers[&'C'].address()), 'B').unwrap();

	// Empty block signed by `C`
	let block = tester.new_block_and_import(CliqueBlockType::Empty, &block, None, 'C').unwrap();

	// Empty block signed by `A`
	let block = tester.new_block_and_import(CliqueBlockType::Empty, &block, None, 'A').unwrap();

	// Vote against `D` signed by `B`
	let block = tester.new_block_and_import(CliqueBlockType::Vote(VoteType::Remove), &block,
											Some(tester.signers[&'D'].address()), 'B').unwrap();

	// Vote against `D` signed by `C`
	let block = tester.new_block_and_import(CliqueBlockType::Vote(VoteType::Remove), &block,
											Some(tester.signers[&'D'].address()), 'C').unwrap();

	let tags = tester.into_tags(tester.clique_signers(&block.hash()));
	assert_eq!(&tags, &['A', 'B', 'C'], "D should have been removed after 3/4 remove votes");

	// Empty block signed by `A`
	let block = tester.new_block_and_import(CliqueBlockType::Empty, &block, None, 'A').unwrap();

	// Vote for `C` signed by `C`
	let block = tester.new_block_and_import(CliqueBlockType::Vote(VoteType::Add), &block,
											Some(tester.signers[&'C'].address()), 'C').unwrap();

	let tags = tester.into_tags(tester.clique_signers(&block.hash()));
	assert_eq!(&tags, &['A', 'B']);
}

#[test]
fn consensus_out_of_bounds_first_touch() {
	let tester = CliqueTester::with(100, 1, vec!['A', 'B', 'C', 'D']);

	// Vote against `C` signed by `A`
	let block = tester.new_block_and_import(CliqueBlockType::Vote(VoteType::Remove), &tester.genesis,
											Some(tester.signers[&'C'].address()), 'A').unwrap();

	// Empty block signed by `B`
	let block = tester.new_block_and_import(CliqueBlockType::Empty, &block, None, 'B').unwrap();

	// Empty block signed by `C`
	let block = tester.new_block_and_import(CliqueBlockType::Empty, &block, None, 'C').unwrap();

	// Vote against `D` signed by `A`
	let block = tester.new_block_and_import(CliqueBlockType::Vote(VoteType::Remove), &block,
											Some(tester.signers[&'D'].address()), 'A').unwrap();

	// Vote against `C` signed by `B`
	let block = tester.new_block_and_import(CliqueBlockType::Vote(VoteType::Remove), &block,
											Some(tester.signers[&'C'].address()), 'B').unwrap();

	// Empty block signed by `C`
	let block = tester.new_block_and_import(CliqueBlockType::Empty, &block, None, 'C').unwrap();

	// Empty block signed by `A`
	let block = tester.new_block_and_import(CliqueBlockType::Empty, &block, None, 'A').unwrap();

	// Vote against `D` signed by `B`
	let block = tester.new_block_and_import(CliqueBlockType::Vote(VoteType::Remove), &block,
											Some(tester.signers[&'D'].address()), 'B').unwrap();

	// Vote against `D` signed by `C`
	let block = tester.new_block_and_import(CliqueBlockType::Vote(VoteType::Remove), &block,
											Some(tester.signers[&'D'].address()), 'C').unwrap();

	let tags = tester.into_tags(tester.clique_signers(&block.hash()));
	assert_eq!(&tags, &['A', 'B', 'C']);

	// Empty block signed by `A`
	let block = tester.new_block_and_import(CliqueBlockType::Empty, &block, None, 'A').unwrap();

	// Vote for `C` signed by `B`
	let block = tester.new_block_and_import(CliqueBlockType::Vote(VoteType::Add), &block,
											Some(tester.signers[&'C'].address()), 'B').unwrap();

	let tags = tester.into_tags(tester.clique_signers(&block.hash()));
	assert_eq!(&tags, &['A', 'B', 'C']);
}

#[test]
fn pending_votes_doesnt_survive_authorization_changes() {
	let tester = CliqueTester::with(100, 1, vec!['A', 'B', 'C', 'D', 'E']);

	let mut prev_header = tester.genesis.clone();

	// Vote for `F` from [`A`, `B`, `C`]
	for sign in SIGNER_TAGS.iter().take(3) {
		let block = tester.new_block_and_import(CliqueBlockType::Vote(VoteType::Add), &prev_header,
												Some(tester.signers[&'F'].address()), *sign).unwrap();
		prev_header = block.clone();
	}

	let tags = tester.into_tags(tester.clique_signers(&prev_header.hash()));
	assert_eq!(&tags, &['A', 'B', 'C', 'D', 'E', 'F'], "F should have been added");

	// Vote against `F` from [`D`, `E`, `B`, `C`]
	for sign in SIGNER_TAGS.iter().skip(3).chain(SIGNER_TAGS.iter().skip(1).take(2)) {
		let block = tester.new_block_and_import(CliqueBlockType::Vote(VoteType::Remove), &prev_header,
												Some(tester.signers[&'F'].address()), *sign).unwrap();
		prev_header = block.clone();
	}

	let tags = tester.into_tags(tester.clique_signers(&prev_header.hash()));
	assert_eq!(&tags, &['A', 'B', 'C', 'D', 'E'], "F should have been removed");

	// Vote for `F` from [`D`, `E`]
	for sign in SIGNER_TAGS.iter().skip(3).take(2) {
		let block = tester.new_block_and_import(CliqueBlockType::Vote(VoteType::Add), &prev_header,
												Some(tester.signers[&'F'].address()), *sign).unwrap();
		prev_header = block.clone();
	}

	// Vote against `A` from [`B`, `C`, `D`]
	for sign in SIGNER_TAGS.iter().skip(1).take(3) {
		let block = tester.new_block_and_import(CliqueBlockType::Vote(VoteType::Remove), &prev_header,
												Some(tester.signers[&'A'].address()), *sign).unwrap();
		prev_header = block.clone();
	}

	let tags = tester.into_tags(tester.clique_signers(&prev_header.hash()));
	assert_eq!(&tags, &['B', 'C', 'D', 'E'], "A should have been removed");

	let block = tester.new_block_and_import(CliqueBlockType::Vote(VoteType::Add), &prev_header,
											Some(tester.signers[&'F'].address()), 'B').unwrap();

	let tags = tester.into_tags(tester.clique_signers(&block.hash()));
	assert_eq!(&tags, &['B', 'C', 'D', 'E', 'F'], "F should have been added again");
}

#[test]
fn epoch_transition_reset_all_votes() {
	let tester = CliqueTester::with(3, 1, vec!['A', 'B']);

	let block = tester.new_block_and_import(CliqueBlockType::Vote(VoteType::Add), &tester.genesis,
											Some(tester.signers[&'C'].address()), 'A').unwrap();

	let block = tester.new_block_and_import(CliqueBlockType::Empty, &block, None, 'B').unwrap();
	let block = tester.new_block_and_import(CliqueBlockType::Checkpoint, &block, None, 'A').unwrap();

	let block = tester.new_block_and_import(CliqueBlockType::Vote(VoteType::Add), &block,
											Some(tester.signers[&'C'].address()), 'B').unwrap();

	let tags = tester.into_tags(tester.clique_signers(&block.hash()));
	assert_eq!(&tags, &['A', 'B'], "Votes should have been reset after checkpoint");
}

#[test]
fn unauthorized_signer_should_not_be_able_to_sign_block() {
	let tester = CliqueTester::with(3, 1, vec!['A']);
	let err = tester.new_block_and_import(CliqueBlockType::Empty, &tester.genesis, None, 'B').unwrap_err();

	match err {
		Error::Engine(EngineError::NotAuthorized(_)) => (),
		_ => assert!(true == false, "Wrong error kind"),
	}
}

#[test]
fn signer_should_not_be_able_to_sign_two_consequtive_blocks() {
	let tester = CliqueTester::with(3, 1, vec!['A', 'B']);
	let b = tester.new_block_and_import(CliqueBlockType::Empty, &tester.genesis, None, 'A').unwrap();
	let err = tester.new_block_and_import(CliqueBlockType::Empty, &b, None, 'A').unwrap_err();

	match err {
		Error::Engine(EngineError::CliqueTooRecentlySigned(_)) => (),
		_ => assert!(true == false, "Wrong error kind"),
	}
}


#[test]
fn recent_signers_should_not_reset_on_checkpoint() {
	let tester = CliqueTester::with(3, 1, vec!['A', 'B', 'C']);

	let block = tester.new_block_and_import(CliqueBlockType::Empty, &tester.genesis, None, 'A').unwrap();
	let block = tester.new_block_and_import(CliqueBlockType::Empty, &block, None, 'B').unwrap();
	let block = tester.new_block_and_import(CliqueBlockType::Checkpoint, &block, None, 'A').unwrap();

	let err = tester.new_block_and_import(CliqueBlockType::Empty, &block, None, 'A').unwrap_err();

	match err {
		Error::Engine(EngineError::CliqueTooRecentlySigned(_)) => (),
		_ => assert!(true == false, "Wrong error kind"),
	}
}

// Not part of http://eips.ethereum.org/EIPS/eip-225
#[test]
fn bonus_consensus_should_keep_track_of_votes_before_latest_per_signer() {
	let tester = CliqueTester::with(100, 1, vec!['A', 'B', 'C', 'D']);

	// Add a vote for `E` signed by `A`
	let vote = tester.new_block_and_import(CliqueBlockType::Vote(VoteType::Add), &tester.genesis,
										   Some(tester.signers[&'E'].address()), 'A').unwrap();
	// Empty block signed by `B`
	let vote = tester.new_block_and_import(CliqueBlockType::Empty, &vote, None, 'B').unwrap();

	// Empty block signed by `C`
	let vote = tester.new_block_and_import(CliqueBlockType::Empty, &vote, None, 'C').unwrap();

	// Empty block signed by `D`
	let vote = tester.new_block_and_import(CliqueBlockType::Empty, &vote, None, 'D').unwrap();

	// Add a vote for `F` signed by `A`
	let vote = tester.new_block_and_import(CliqueBlockType::Vote(VoteType::Add), &vote,
										   Some(tester.signers[&'F'].address()), 'A').unwrap();
	// Empty block signed by `C`
	let vote = tester.new_block_and_import(CliqueBlockType::Empty, &vote, None, 'C').unwrap();

	// Empty block signed by `D`
	let vote = tester.new_block_and_import(CliqueBlockType::Empty, &vote, None, 'D').unwrap();

	// Add a vote for `E` signed by `B`
	let vote = tester.new_block_and_import(CliqueBlockType::Vote(VoteType::Add), &vote,
										   Some(tester.signers[&'E'].address()), 'B').unwrap();
	// Empty block signed by `A`
	let vote = tester.new_block_and_import(CliqueBlockType::Empty, &vote, None, 'A').unwrap();

	// Empty block signed by `C`
	let vote = tester.new_block_and_import(CliqueBlockType::Empty, &vote, None, 'C').unwrap();

	// Empty block signed by `D`
	let vote = tester.new_block_and_import(CliqueBlockType::Empty, &vote, None, 'D').unwrap();

	// Add a vote for `F` signed by `B`
	let vote = tester.new_block_and_import(CliqueBlockType::Vote(VoteType::Add), &vote,
										   Some(tester.signers[&'F'].address()), 'B').unwrap();

	// Empty block signed by A`
	let vote = tester.new_block_and_import(CliqueBlockType::Empty, &vote, None, 'A').unwrap();

	// Add a vote for `E` signed by `C`
	let vote = tester.new_block_and_import(CliqueBlockType::Vote(VoteType::Add), &vote,
										   Some(tester.signers[&'E'].address()), 'C').unwrap();

	let tags = tester.into_tags(tester.clique_signers(&vote.hash()));
	assert_eq!(&tags, &['A', 'B', 'C', 'D', 'E']);
}
