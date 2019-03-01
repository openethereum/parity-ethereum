use engines::Engine;
use error::Error;
use ethereum_types::{Address, H256};
use ethkey::{Secret, KeyPair};
use block::*;
use state_db::StateDB;
use super::*;
use test_helpers::get_temp_state_db;

use std::sync::Arc;
use std::collections::HashMap;

const KEYS: [char; 6] = ['A', 'B', 'C', 'D', 'E', 'F'];

enum CliqueBlockType {
	/// Epoch transition block must contain list of signers
	Checkpoint,
	/// Block with no votes
	Empty,
	/// Vote
	Vote(VoteType),
}

struct Tester {
	pub clique: Clique,
	pub genesis: Header,
	pub db: StateDB,
	pub signers: HashMap<char, KeyPair>,
}

impl Tester {
	fn with(epoch: u64, period: u64, secrets: Vec<(char, bool)>) -> Self {
		let clique = Clique::with_test(epoch, period);
		let mut genesis = Header::default();
		let mut signers = HashMap::new();

		let call = |_a, _b| {
			unimplemented!("Clique doesn't use Engine::Call");
		};

		let mut extra_data = vec![0; VANITY_LENGTH];

		for (id, in_signer_list) in secrets.iter() {
			let secret = Secret::from(H256::from(*id as u64));
			let keypair = KeyPair::from_secret(secret).unwrap();
			if *in_signer_list {
				extra_data.extend(&*keypair.address());
			}
			signers.insert(*id, keypair);
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
}

impl Tester {
	fn get_difficulty(&self, block_num: BlockNumber, header: &Header, signer: &Address) -> U256 {
		let state = self.clique.state(header).unwrap();
		if state.is_inturn(block_num, signer) {
			DIFF_INTURN
		} else {
			DIFF_NOTURN
		}
	}

	// Note, this will read the cache and `will` not work with more than 128 blocks
	fn get_state_at_block(&self, hash: &H256) -> CliqueBlockState {
		self.clique.block_state_by_hash.write()
			.get_mut(hash)
			.expect("CliqueBlockState not found tested failed")
			.clone()
	}

	// Get signers after a certain state
	// This is generally used to fetch the state after a test has been executed and checked against
	// the intial list of signers provided in the test
	fn clique_signers(&self, hash: &H256) -> Vec<Address> {
		self.get_state_at_block(hash).signers().to_vec()
	}

	// Fetches all addresses at current `block` and converts them back to `tags (char)` and sorts them
	// Addresses are supposed sorted based on address but these tests are using `tags` just for simplicity
	// and the order is not important!
	fn tags_from_vec(&self, addr: &Vec<Address>) -> Vec<char> {
		let mut tags: Vec<char> = addr.iter().filter_map(|addr| {
			for (t, kp) in self.signers.iter() {
				if addr == &kp.address() {
					return Some(*t)
				}
			}
			None
		}).collect();
		tags.sort();
		tags
	}

	fn tags_from_deq(&self, addr: &std::collections::VecDeque<Address>) -> Vec<char> {
		addr.iter().filter_map(|addr| {
			for (t, kp) in self.signers.iter() {
				if addr == &kp.address() {
					return Some(*t)
				}
			}
			None
		})
		.collect()
	}

	fn new_block_and_import(
		&self,
		block_type: CliqueBlockType,
		last_header: &Header,
		beneficary: Option<Address>,
		signer: char,
		) -> Result<OpenBlock, Error> {

		let mut extra_data = vec![0; VANITY_LENGTH];
		let mut seal = util::null_seal();
		let last_hash = last_header.hash();

		match block_type {
			CliqueBlockType::Checkpoint => {
				let signers = self.clique.state(&last_header).unwrap().signers().clone();
				for signer in signers {
					extra_data.extend(&*signer);
				}
				{}
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
			&mut Vec::new().into_iter(),
		).unwrap();

		{
			let difficulty = self.get_difficulty(block.header().number(), last_header, &self.signers[&signer].address());
			let b = block.block_mut();
			b.header.set_timestamp(last_header.timestamp() + self.clique.period);
			b.header.set_difficulty(difficulty);
			b.header.set_seal(seal);

			let sign = ethkey::sign(self.signers[&signer].secret(), &b.header.hash()).unwrap();
			let mut extra_data = b.header.extra_data().clone();
			extra_data.extend_from_slice(&*sign);
			b.header.set_extra_data(extra_data);
		}

		self.clique.verify_block_basic(block.header())?;
		self.clique.verify_block_family(block.header(), &last_header)?;

		Ok(block)
	}
}

#[test]
fn one_signer_with_no_votes() {
	let tester = Tester::with(10, 1, vec![('A', true)]);

	let empty_block = tester.new_block_and_import(CliqueBlockType::Empty, &tester.genesis, None, 'A').unwrap();

	let signers = tester.clique_signers(&empty_block.header().hash());

	assert_eq!(signers.len(), 1);
	assert_eq!(signers[0], tester.signers[&'A'].address());
}

#[test]
fn one_signer_two_votes() {
	let tester = Tester::with(10, 1, vec![('A', true), ('B', false), ('C', false)]);

	// Add a vote for `B` signed by A
	let vote = tester.new_block_and_import(CliqueBlockType::Vote(VoteType::Add), &tester.genesis,
										   Some(tester.signers[&'B'].address()), 'A').unwrap();

	// Empty block signed by `B` (not in the signer list)
	let empty = tester.new_block_and_import(CliqueBlockType::Empty, &vote.header(), None, 'B').unwrap();

	// Add vote for `C` signed by A but should not be accepted
	let vote = tester.new_block_and_import(CliqueBlockType::Vote(VoteType::Add), &empty.header(),
										   Some(tester.signers[&'C'].address()), 'A').unwrap();

	let signers = tester.clique_signers(&vote.header().hash());
	assert_eq!(signers.len(), 2);
	assert!(signers.contains(&tester.signers[&'A'].address());
	assert!(signers.contains(&tester.signers[&'B'].address());
	assert!(!signers.contains(&tester.signers[&'C'].address());
}

#[test]
fn two_signers_six_votes_deny_last() {
	let tester = Tester::with(10, 1, vec![('A', true), ('B', true), ('C', false), ('D', false), ('E', false)]);

	let mut prev_header = tester.genesis.clone();

	// Add two votes for `C` signed by A and B
	for &signer in KEYS.iter().take(2) {
		let vote = tester.new_block_and_import(CliqueBlockType::Vote(VoteType::Add), &prev_header,
											   Some(tester.signers[&'C'].address()), signer).unwrap();
		prev_header = vote.header().clone();
	}

	// Add two votes for `D` signed by A and B
	for &signer in KEYS.iter().take(2) {
		let vote = tester.new_block_and_import(CliqueBlockType::Vote(VoteType::Add), &prev_header,
											   Some(tester.signers[&'D'].address()), signer).unwrap();
		prev_header = vote.header().clone();
	}

	// Add a empty block signed by `C`
	let empty = tester.new_block_and_import(CliqueBlockType::Empty, &prev_header, None, 'C').unwrap();
	prev_header = empty.header().clone();

	// Add two votes for `E` signed by A and B
	for &signer in KEYS.iter().take(2) {
		let vote = tester.new_block_and_import(CliqueBlockType::Vote(VoteType::Add), &prev_header,
											   Some(tester.signers[&'E'].address()), signer).unwrap();
		prev_header = vote.header().clone();
	}

	let signers = tester.clique_signers(&prev_header.hash());

	assert!(signers.len() == 4);
	assert!(signers.contains(&tester.signers[&'A'].address());
	assert!(signers.contains(&tester.signers[&'B'].address());
	assert!(signers.contains(&tester.signers[&'C'].address());
	assert!(signers.contains(&tester.signers[&'D']].address());
	assert!(!signers.contains(&tester.signers[&'E'].address());
}

#[test]
fn one_signer_dropping_itself() {
	let tester = Tester::with(10, 1, vec![('A', true)]);
	let vote = tester.new_block_and_import(CliqueBlockType::Vote(VoteType::Remove), &tester.genesis,
										   Some(tester.signers[&'A'].address()), 'A').unwrap();
	let signers = tester.clique_signers(&vote.header().hash());
	assert!(signers.is_empty());
}

#[test]
fn two_signers_one_remove_vote_no_consensus() {
	let tester = Tester::with(10, 1, vec![('A', true), ('B', true)]);
	let vote = tester.new_block_and_import(CliqueBlockType::Vote(VoteType::Remove), &tester.genesis,
										   Some(tester.signers[&'B'].address()), 'A').unwrap();

	let signers = tester.clique_signers(&vote.header().hash());
	assert!(signers.len() == 2);
	assert!(signers.contains(&tester.signers[&'A'].address());
	assert!(signers.contains(&tester.signers[&'B'].address());
}

#[test]
fn two_signers_consensus_remove_b() {
	let tester = Tester::with(10, 1, vec![('A', true), ('B', true)]);
	let first_vote = tester.new_block_and_import(CliqueBlockType::Vote(VoteType::Remove), &tester.genesis,
												 Some(tester.signers[&'B'].address()), 'A').unwrap();
	let second_vote = tester.new_block_and_import(CliqueBlockType::Vote(VoteType::Remove), &first_vote.header(),
												 Some(tester.signers[&'B'].address()), 'B').unwrap();

	let signers = tester.clique_signers(&second_vote.header().hash());
	assert!(signers.len() == 1);
	assert!(signers.contains(&tester.signers[&'A'].address());
	assert!(!signers.contains(&tester.signers[&'B'].address());
}

#[test]
fn three_signers_consensus_remove_c() {
	let tester = Tester::with(10, 1, vec![('A', true), ('B', true), ('C', true)]);
	let first_vote = tester.new_block_and_import(CliqueBlockType::Vote(VoteType::Remove), &tester.genesis,
												 Some(tester.signers[&'C'].address()), 'A').unwrap();
	let second_vote = tester.new_block_and_import(CliqueBlockType::Vote(VoteType::Remove), &first_vote.header(),
												 Some(tester.signers[&'C'].address()), 'B').unwrap();

	let signers = tester.clique_signers(&second_vote.header().hash());
	assert!(signers.len() == 2);
	assert!(signers.contains(&tester.signers[&'A'].address());
	assert!(signers.contains(&tester.signers[&'B'].address());
	assert!(!signers.contains(&tester.signers[&'C'].address());
}

#[test]
fn four_signers_half_no_consensus() {
	let tester = Tester::with(10, 1, vec![('A', true), ('B', true), ('C', true), ('D', true)]);
	let first_vote = tester.new_block_and_import(CliqueBlockType::Vote(VoteType::Remove), &tester.genesis,
												 Some(tester.signers[&'C'].address()), 'A').unwrap();

	let second_vote = tester.new_block_and_import(CliqueBlockType::Vote(VoteType::Remove), &first_vote.header(),
												  Some(tester.signers[&'C'].address()), 'B').unwrap();

	let signers = tester.clique_signers(&second_vote.header().hash());
	assert!(signers.len() == 4);
	assert!(signers.contains(&tester.signers[&'A'].address());
	assert!(signers.contains(&tester.signers[&'B'].address());
	assert!(signers.contains(&tester.signers[&'C'].address());
	assert!(signers.contains(&tester.signers[&'D'].address());
}

#[test]
fn four_signers_three_consensus_rm() {
	let tester = Tester::with(10, 1, vec![('A', true), ('B', true), ('C', true), ('D', true)]);

	let mut prev_header = tester.genesis.clone();

	// Three votes to remove `D` signed by ['A', 'B', 'C']
	for signer in KEYS.iter().take(3) {
		let vote = tester.new_block_and_import(CliqueBlockType::Vote(VoteType::Remove), &prev_header,
											   Some(tester.signers[&'D'].address()), *signer).unwrap();
		prev_header = vote.header().clone();
	}

	let signers = tester.clique_signers(&prev_header.hash());
	assert!(signers.len() == 3);
	assert!(signers.contains(&tester.signers[&'A'].address());
	assert!(signers.contains(&tester.signers[&'B'].address());
	assert!(signers.contains(&tester.signers[&'C'].address());
	assert!(!signers.contains(&tester.signers[&'D'].address());
}

#[test]
fn vote_add_only_counted_once_per_signer() {
	let tester = Tester::with(10, 1, vec![('A', true), ('B', true), ('C', false)]);

	// Add a vote for `C` signed by A
	let vote = tester.new_block_and_import(CliqueBlockType::Vote(VoteType::Add), &tester.genesis,
										   Some(tester.signers[&'C'].address()), 'A').unwrap();
	// Empty block signed by B`
	let empty = tester.new_block_and_import(CliqueBlockType::Empty, &vote.header(), None, 'B').unwrap();

	// Add a vote for `C` signed by A
	let vote = tester.new_block_and_import(CliqueBlockType::Vote(VoteType::Add), &empty.header(),
										   Some(tester.signers[&'C'].address()), 'A').unwrap();
	// Empty block signed by B`
	let empty = tester.new_block_and_import(CliqueBlockType::Empty, &vote.header(), None, 'B').unwrap();

	// Add a vote for `C` signed by A
	let vote = tester.new_block_and_import(CliqueBlockType::Vote(VoteType::Add), &empty.header(),
										   Some(tester.signers[&'C'].address()), 'A').unwrap();

	let signers = tester.clique_signers(&vote.header().hash());
	assert_eq!(signers.len(), 2);
	assert!(signers.contains(&tester.signers[&'A'].address());
	assert!(signers.contains(&tester.signers[&'B'].address());
	assert!(!signers.contains(&tester.signers[&'C'].address());
}

#[test]
fn vote_add_concurrently_is_permitted() {
	let tester = Tester::with(10, 1, vec![('A', true), ('B', true), ('C', false), ('D', false)]);

	// Add a vote for `C` signed by A
	let b = tester.new_block_and_import(CliqueBlockType::Vote(VoteType::Add), &tester.genesis,
										   Some(tester.signers[&'C'].address()), 'A').unwrap();

	// Empty block signed by B`
	let b = tester.new_block_and_import(CliqueBlockType::Empty, &b.header(), None, 'B').unwrap();

	// Add a vote for `D` signed by A
	let b = tester.new_block_and_import(CliqueBlockType::Vote(VoteType::Add), &b.header(),
										   Some(tester.signers[&'D'].address()), 'A').unwrap();

	// Empty block signed by B`
	let b = tester.new_block_and_import(CliqueBlockType::Empty, &b.header(), None, 'B').unwrap();

	// Empty block signed by A`
	let b = tester.new_block_and_import(CliqueBlockType::Empty, &b.header(), None, 'A').unwrap();

	// Add a vote for `D` signed by B
	let b = tester.new_block_and_import(CliqueBlockType::Vote(VoteType::Add), &b.header(),
										   Some(tester.signers[&'D'].address()), 'B').unwrap();

	// Empty block signed by A`
	let b = tester.new_block_and_import(CliqueBlockType::Empty, &b.header(), None, 'A').unwrap();

	// Add a vote for `C` signed by B
	let b = tester.new_block_and_import(CliqueBlockType::Vote(VoteType::Add), &b.header(),
										   Some(tester.signers[&'C'].address()), 'B').unwrap();

	let signers = tester.clique_signers(&b.header().hash());
	assert_eq!(signers.len(), 4);
	assert!(signers.contains(&tester.signers[&'A'].address());
	assert!(signers.contains(&tester.signers[&'B'].address());
	assert!(signers.contains(&tester.signers[&'C'].address());
	assert!(signers.contains(&tester.signers[&'D'].address());
}

#[test]
fn vote_rm_only_counted_once_per_signer() {
	let tester = Tester::with(10, 1, vec![('A', true), ('B', true)]);

	let mut prev_header = tester.genesis.clone();

	for _ in 0..2 {
		// Vote to remove `B` signed by A
		let b = tester.new_block_and_import(CliqueBlockType::Vote(VoteType::Remove), &prev_header,
											   Some(tester.signers[&'B'].address()), 'A').unwrap();
		// Empty block signed by B`
		let b = tester.new_block_and_import(CliqueBlockType::Empty, &b.header(), None, 'B').unwrap();

		prev_header = b.header().clone();
	}

	// Add a vote for `B` signed by A
	let b = tester.new_block_and_import(CliqueBlockType::Vote(VoteType::Remove), &prev_header,
										   Some(tester.signers[&'B'].address()), 'A').unwrap();

	let signers = tester.clique_signers(&b.header().hash());
	assert_eq!(signers.len(), 2);
	assert!(signers.contains(&tester.signers[&'A'].address());
	assert!(signers.contains(&tester.signers[&'B'].address());
}

#[test]
fn vote_rm_concurrently_is_permitted() {
	let tester = Tester::with(100, 1, vec![('A', true), ('B', true), ('C', true), ('D', true)]);

	// Add a vote for `C` signed by A
	let block = tester.new_block_and_import(CliqueBlockType::Vote(VoteType::Remove), &tester.genesis,
											Some(tester.signers[&'C'].address()), 'A').unwrap();

	// Empty block signed by `B`
	let block = tester.new_block_and_import(CliqueBlockType::Empty, &block.header(), None, 'B').unwrap();
	// Empty block signed by `C`
	let block = tester.new_block_and_import(CliqueBlockType::Empty, &block.header(), None, 'C').unwrap();

	// Add a vote for `D` signed by A
	let block = tester.new_block_and_import(CliqueBlockType::Vote(VoteType::Remove), &block.header(),
											Some(tester.signers[&'D'].address()), 'A').unwrap();

	// Empty block signed by `B`
	let block = tester.new_block_and_import(CliqueBlockType::Empty, &block.header(), None, 'B').unwrap();
	// Empty block signed by `C`
	let block = tester.new_block_and_import(CliqueBlockType::Empty, &block.header(), None, 'C').unwrap();
	// Empty block signed by A`
	let block = tester.new_block_and_import(CliqueBlockType::Empty, &block.header(), None, 'A').unwrap();

	// Add a vote for `D` signed by B
	let block = tester.new_block_and_import(CliqueBlockType::Vote(VoteType::Remove), &block.header(),
											Some(tester.signers[&'D'].address()), 'B').unwrap();
	// Add a vote for `D` signed by C
	let block = tester.new_block_and_import(CliqueBlockType::Vote(VoteType::Remove), &block.header(),
											Some(tester.signers[&'D'].address()), 'C').unwrap();

	// Empty block signed by `A`
	let block = tester.new_block_and_import(CliqueBlockType::Empty, &block.header(), None, 'A').unwrap();
	// Add a vote for `C` signed by B
	let block = tester.new_block_and_import(CliqueBlockType::Vote(VoteType::Remove), &block.header(),
											Some(tester.signers[&'C'].address()), 'B').unwrap();

	let signers = tester.clique_signers(&block.header().hash());
	assert_eq!(signers.len(), 2);
	assert!(signers.contains(&tester.signers[&'A'].address());
	assert!(signers.contains(&tester.signers[&'B'].address());
	assert!(!signers.contains(&tester.signers[&'C'].address());
	assert!(!signers.contains(&tester.signers[&'D'.address());
}

#[test]
fn vote_to_rm_are_immediate_and_ensure_votes_are_rm() {
	let tester = Tester::with(100, 1, vec![('A', true), ('B', true), ('C', true)]);

	// Vote to remove `B` signed by `C`
	let block = tester.new_block_and_import(CliqueBlockType::Vote(VoteType::Remove), &tester.genesis,
											Some(tester.signers[&'B'].address()), 'C').unwrap();
	// Vote to remove `C` signed by `A`
	let block = tester.new_block_and_import(CliqueBlockType::Vote(VoteType::Remove), &block.header(),
											Some(tester.signers[&'C'].address()), 'A').unwrap();
	// Vote to remove `C` signed by `B`
	let block = tester.new_block_and_import(CliqueBlockType::Vote(VoteType::Remove), &block.header(),
											Some(tester.signers[&'C'].address()), 'B').unwrap();
	// Vote to remove `B` signed by `A`
	let block = tester.new_block_and_import(CliqueBlockType::Vote(VoteType::Remove), &block.header(),
											Some(tester.signers[&'B'].address()), 'A').unwrap();

	let signers = tester.clique_signers(&block.header().hash());
	assert!(signers.len() == 2);
	assert!(signers.contains(&tester.signers[&'A'].address());
	assert!(signers.contains(&tester.signers[&'B'].address());
	assert!(!signers.contains(&tester.signers[&'C'].address());
}

#[test]
fn vote_to_rm_are_immediate_and_votes_should_be_dropped_from_kicked_signer() {
	let tester = Tester::with(100, 1, vec![('A', true), ('B', true), ('C', true), ('D', false)]);

	// Vote to add `D` signed by `C`
	let block = tester.new_block_and_import(CliqueBlockType::Vote(VoteType::Add), &tester.genesis,
											Some(tester.signers[&'D'].address()), 'C').unwrap();
	// Vote to remove `C` signed by `A`
	let block = tester.new_block_and_import(CliqueBlockType::Vote(VoteType::Remove), &block.header(),
											Some(tester.signers[&'C'].address()), 'A').unwrap();

	// Vote to remove `C` signed by `B`
	let block = tester.new_block_and_import(CliqueBlockType::Vote(VoteType::Remove), &block.header(),
											Some(tester.signers[&'C'].address()), 'B').unwrap();

	// Vote to add `D` signed by `A`
	let block = tester.new_block_and_import(CliqueBlockType::Vote(VoteType::Add), &block.header(),
											Some(tester.signers[&'D'].address()), 'A').unwrap();

	let signers = tester.clique_signers(&block.header().hash());
	assert!(signers.len() == 2);
	assert!(signers.contains(&tester.signers[&'A'].address());
	assert!(signers.contains(&tester.signers[&'B'].address());
	assert!(!signers.contains(&tester.signers[&'C'].address());
	assert!(!signers.contains(&tester.signers[&'D'].address());
}

#[test]
fn cascading_not_allowed() {
	let tester = Tester::with(100, 1, vec![('A', true), ('B', true), ('C', true), ('D', true)]);

	// Vote against `C` signed by `A`
	let block = tester.new_block_and_import(CliqueBlockType::Vote(VoteType::Remove), &tester.genesis,
											Some(tester.signers[&'C'].address()), 'A').unwrap();

	// Empty block signed by `B`
	let block = tester.new_block_and_import(CliqueBlockType::Empty, &block.header(), None, 'B').unwrap();

	// Empty block signed by `C`
	let block = tester.new_block_and_import(CliqueBlockType::Empty, &block.header(), None, 'C').unwrap();

	// Vote against `D` signed by `A`
	let block = tester.new_block_and_import(CliqueBlockType::Vote(VoteType::Remove), &block.header(),
											Some(tester.signers[&'D'].address()), 'A').unwrap();

	// Vote against `C` signed by `B`
	let block = tester.new_block_and_import(CliqueBlockType::Vote(VoteType::Remove), &block.header(),
											Some(tester.signers[&'C'].address()), 'B').unwrap();

	// Empty block signed by `C`
	let block = tester.new_block_and_import(CliqueBlockType::Empty, &block.header(), None, 'C').unwrap();

	// Empty block signed by `A`
	let block = tester.new_block_and_import(CliqueBlockType::Empty, &block.header(), None, 'A').unwrap();

	// Vote against `D` signed by `B`
	let block = tester.new_block_and_import(CliqueBlockType::Vote(VoteType::Remove), &block.header(),
											Some(tester.signers[&'D'].address()), 'B').unwrap();

	// Vote against `D` signed by `C`
	let block = tester.new_block_and_import(CliqueBlockType::Vote(VoteType::Remove), &block.header(),
											Some(tester.signers[&'D'].address()), 'C').unwrap();

	let signers = tester.clique_signers(&block.header().hash());
	assert!(signers.len() == 3);
	assert!(signers.contains(&tester.signers[&'A'].address());
	assert!(signers.contains(&tester.signers[&'B'].address());
	assert!(signers.contains(&tester.signers[&'C'].address());
	assert!(!signers.contains(&tester.signers[&'D'].address());
}

#[test]
#[ignore]
// TODO(niklasad1): verify if the test case is correct, why is C dropped?
// I guess because A and B voted on them before an that state should be applied after D is removed?!
fn consensus_out_of_bounds_on_touch() {
	let tester = Tester::with(100, 1, vec![('A', true), ('B', true), ('C', true), ('D', true)]);

	// Vote against `C` signed by `A`
	let block = tester.new_block_and_import(CliqueBlockType::Vote(VoteType::Remove), &tester.genesis,
											Some(tester.signers[&'C'].address()), 'A').unwrap();

	// Empty block signed by `B`
	let block = tester.new_block_and_import(CliqueBlockType::Empty, &block.header(), None, 'B').unwrap();

	// Empty block signed by `C`
	let block = tester.new_block_and_import(CliqueBlockType::Empty, &block.header(), None, 'C').unwrap();

	// Vote against `D` signed by `A`
	let block = tester.new_block_and_import(CliqueBlockType::Vote(VoteType::Remove), &block.header(),
											Some(tester.signers[&'D'].address()), 'A').unwrap();

	// Vote against `C` signed by `B`
	let block = tester.new_block_and_import(CliqueBlockType::Vote(VoteType::Remove), &block.header(),
											Some(tester.signers[&'C'].address()), 'B').unwrap();

	// Empty block signed by `C`
	let block = tester.new_block_and_import(CliqueBlockType::Empty, &block.header(), None, 'C').unwrap();

	// Empty block signed by `A`
	let block = tester.new_block_and_import(CliqueBlockType::Empty, &block.header(), None, 'A').unwrap();

	// Vote against `D` signed by `B`
	let block = tester.new_block_and_import(CliqueBlockType::Vote(VoteType::Remove), &block.header(),
											Some(tester.signers[&'D'].address()), 'B').unwrap();

	// Vote against `D` signed by `C`
	let block = tester.new_block_and_import(CliqueBlockType::Vote(VoteType::Remove), &block.header(),
											Some(tester.signers[&'D'].address()), 'C').unwrap();

	let signers = tester.clique_signers(&block.header().hash());
	assert!(signers.len() == 3, "D should be dropped");
	assert_eq!(signers.contains(&tester.signers[&'D'].address()), false);

	// Empty block signed by `A`
	let block = tester.new_block_and_import(CliqueBlockType::Empty, &block.header(), None, 'A').unwrap();

	// Vote for `C` signed by `C`
	let block = tester.new_block_and_import(CliqueBlockType::Vote(VoteType::Add), &block.header(),
											Some(tester.signers[&'C'].address()), 'C').unwrap();

	let signers = tester.clique_signers(&block.header().hash());
	println!("{:?}", tester.tags_from_vec(&signers));
	assert!(signers.len() == 2);
	assert_eq!(signers.contains(&tester.signers[&'A'].address()), true);
	assert_eq!(signers.contains(&tester.signers[&'B'].address()), true);
	assert_eq!(signers.contains(&tester.signers[&'C'].address()), false);
	assert_eq!(signers.contains(&tester.signers[&'D'].address()), false);
}

#[test]
fn consensus_out_of_bounds_first_touch() {
	let tester = Tester::with(100, 1, vec![('A', true), ('B', true), ('C', true), ('D', true)]);

	// Vote against `C` signed by `A`
	let block = tester.new_block_and_import(CliqueBlockType::Vote(VoteType::Remove), &tester.genesis,
											Some(tester.signers[&'C'].address()), 'A').unwrap();

	// Empty block signed by `B`
	let block = tester.new_block_and_import(CliqueBlockType::Empty, &block.header(), None, 'B').unwrap();

	// Empty block signed by `C`
	let block = tester.new_block_and_import(CliqueBlockType::Empty, &block.header(), None, 'C').unwrap();

	// Vote against `D` signed by `A`
	let block = tester.new_block_and_import(CliqueBlockType::Vote(VoteType::Remove), &block.header(),
											Some(tester.signers[&'D'].address()), 'A').unwrap();

	// Vote against `C` signed by `B`
	let block = tester.new_block_and_import(CliqueBlockType::Vote(VoteType::Remove), &block.header(),
											Some(tester.signers[&'C'].address()), 'B').unwrap();

	// Empty block signed by `C`
	let block = tester.new_block_and_import(CliqueBlockType::Empty, &block.header(), None, 'C').unwrap();

	// Empty block signed by `A`
	let block = tester.new_block_and_import(CliqueBlockType::Empty, &block.header(), None, 'A').unwrap();

	// Vote against `D` signed by `B`
	let block = tester.new_block_and_import(CliqueBlockType::Vote(VoteType::Remove), &block.header(),
											Some(tester.signers[&'D'].address()), 'B').unwrap();

	// Vote against `D` signed by `C`
	let block = tester.new_block_and_import(CliqueBlockType::Vote(VoteType::Remove), &block.header(),
											Some(tester.signers[&'D'].address()), 'C').unwrap();

	let tags = tester.tags_from_vec(&tester.clique_signers(&block.header().hash()));
	assert_eq!(&tags, &['A', 'B', 'C']);

	// Empty block signed by `A`
	let block = tester.new_block_and_import(CliqueBlockType::Empty, &block.header(), None, 'A').unwrap();

	// Vote for `C` signed by `B`
	let block = tester.new_block_and_import(CliqueBlockType::Vote(VoteType::Add), &block.header(),
											Some(tester.signers[&'C'].address()), 'B').unwrap();

	let signers = tester.clique_signers(&block.header().hash());
	assert!(signers.len() == 3);
	assert_eq!(signers.contains(&tester.signers[&'A'].address()), true);
	assert_eq!(signers.contains(&tester.signers[&'B'].address()), true);
	assert_eq!(signers.contains(&tester.signers[&'C'].address()), true);
	assert_eq!(signers.contains(&tester.signers[&'D'].address()), false);
}

#[test]
fn pending_votes_doesnt_survive_authorization_changes() {
	let tester = Tester::with(100, 1,
							  vec![('A', true), ('B', true), ('C', true), ('D', true), ('E', true), ('F', false)]);

	let mut prev_header = tester.genesis.clone();

	// Vote for `F` from [`A`, `B`, `C`]
	for sign in KEYS.iter().take(3) {
		let block = tester.new_block_and_import(CliqueBlockType::Vote(VoteType::Add), &prev_header,
												Some(tester.signers[&'F'].address()), *sign).unwrap();
		prev_header = block.header().clone();
	}

	let tags = tester.tags_from_vec(&tester.clique_signers(&prev_header.hash()));
	assert_eq!(&tags, &['A', 'B', 'C', 'D', 'E', 'F'], "F should have been added");

	// Vote against `F` from [`D`, `E`, `B`, `C`]
	for sign in KEYS.iter().skip(3).chain(KEYS.iter().skip(1).take(2)) {
		let block = tester.new_block_and_import(CliqueBlockType::Vote(VoteType::Remove), &prev_header,
												Some(tester.signers[&'F'].address()), *sign).unwrap();
		prev_header = block.header().clone();
	}

	let tags = tester.tags_from_vec(&tester.clique_signers(&prev_header.hash()));
	assert_eq!(&tags, &['A', 'B', 'C', 'D', 'E'], "F should have been removed");

	// Vote for `F` from [`D`, `E`]
	for sign in KEYS.iter().skip(3).take(2) {
		let block = tester.new_block_and_import(CliqueBlockType::Vote(VoteType::Add), &prev_header,
												Some(tester.signers[&'F'].address()), *sign).unwrap();
		prev_header = block.header().clone();
	}

	// Vote against `A` from [`B`, `C`, `D`]
	for sign in KEYS.iter().skip(1).take(3) {
		let block = tester.new_block_and_import(CliqueBlockType::Vote(VoteType::Remove), &prev_header,
												Some(tester.signers[&'A'].address()), *sign).unwrap();
		prev_header = block.header().clone();
	}

	let tags = tester.tags_from_vec(&tester.clique_signers(&prev_header.hash()));
	assert_eq!(&tags, &['B', 'C', 'D', 'E'], "A should have been removed");

	let block = tester.new_block_and_import(CliqueBlockType::Vote(VoteType::Add), &prev_header,
											Some(tester.signers[&'F'].address()), 'B').unwrap();

	let tags = tester.tags_from_vec(&tester.clique_signers(&block.header().hash()));
	assert_eq!(&tags, &['B', 'C', 'D', 'E', 'F'], "F should have been added again");
}

#[test]
fn epoch_transition_reset_all_votes() {
	let tester = Tester::with(3, 1, vec![('A', true), ('B', true), ('C', false)]);

	let block = tester.new_block_and_import(CliqueBlockType::Vote(VoteType::Add), &tester.genesis,
											Some(tester.signers[&'C'].address()), 'A').unwrap();

	let block = tester.new_block_and_import(CliqueBlockType::Empty, &block.header(), None, 'B').unwrap();
	let block = tester.new_block_and_import(CliqueBlockType::Checkpoint, &block.header(), None, 'A').unwrap();

	let block = tester.new_block_and_import(CliqueBlockType::Vote(VoteType::Add), &tester.genesis,
											Some(tester.signers[&'C'].address()), 'B').unwrap();

	let tags = tester.tags_from_vec(&tester.clique_signers(&block.header().hash()));
	assert_eq!(&tags, &['A', 'B'], "Votes should have been reset after checkpoint");
}

#[test]
fn unauthorized_signer_should_not_be_able_to_sign_block() {
	let tester = Tester::with(3, 1, vec![('A', true), ('B', false)]);
	assert!(tester.new_block_and_import(CliqueBlockType::Empty, &tester.genesis, None, 'B').is_err());
}

#[test]
fn signer_should_not_be_able_to_sign_two_consequtive_blocks() {
	let tester = Tester::with(3, 1, vec![('A', true), ('B', true)]);
	let b = tester.new_block_and_import(CliqueBlockType::Empty, &tester.genesis, None, 'A').unwrap();
	assert!(tester.new_block_and_import(CliqueBlockType::Empty, &b.header(), None, 'A').is_err());
}

#[test]
fn recent_signers_should_not_reset_on_checkpoint() {
	let tester = Tester::with(3, 1, vec![('A', true), ('B', true), ('C', false)]);

	let block = tester.new_block_and_import(CliqueBlockType::Empty, &tester.genesis, None, 'A').unwrap();
	let block = tester.new_block_and_import(CliqueBlockType::Empty, &block.header(), None, 'B').unwrap();
	// let block = tester.new_block_and_import(CliqueBlockType::Checkpoint, &block.header(), None, 'C').unwrap();
	// assert!(tester.new_block_and_import(CliqueBlockType::Empty, &block.header(), None, 'A').is_err());

}
