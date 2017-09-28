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

/// Tendermint BFT consensus engine with round robin proof-of-authority.
/// At each blockchain `Height` there can be multiple `View`s of voting.
/// Signatures always sign `Height`, `View`, `Step` and `BlockHash` which is a block hash without seal.
/// First a block with `Seal::Proposal` is issued by the designated proposer.
/// Next the `View` proceeds through `Prevote` and `Precommit` `Step`s.
/// Block is issued when there is enough `Precommit` votes collected on a particular block at the end of a `View`.
/// Once enough votes have been gathered the proposer issues that block in the `Commit` step.

mod message;
mod params;

use std::sync::{Weak, Arc};
use std::sync::atomic::{AtomicUsize, Ordering as AtomicOrdering};
use std::collections::{HashSet, BTreeMap};
use hash::keccak;
use bigint::prelude::{U128, U256};
use bigint::hash::{H256, H520};
use parking_lot::RwLock;
use util::*;
use unexpected::{OutOfBounds, Mismatch};
use client::EngineClient;
use bytes::Bytes;
use error::{Error, BlockError};
use header::{Header, BlockNumber};
use rlp::UntrustedRlp;
use ethkey::{Message, public_to_address, recover, Signature};
use account_provider::AccountProvider;
use block::*;
use engines::{Engine, Seal, EngineError, ConstructedVerifier};
use io::IoService;
use super::signer::EngineSigner;
use super::validator_set::{ValidatorSet, SimpleList};
use super::transition::TransitionHandler;
use super::vote_collector::VoteCollector;
use self::message::*;
use self::params::TendermintParams;
use semantic_version::SemanticVersion;
use machine::{AuxiliaryData, EthereumMachine};

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum Step {
	Propose,
	Prevote,
	Precommit,
	Commit
}

impl Step {
	pub fn is_pre(self) -> bool {
		match self {
			Step::Prevote | Step::Precommit => true,
			_ => false,
		}
	}
}

pub type Height = usize;
pub type View = usize;
pub type BlockHash = H256;

/// Engine using `Tendermint` consensus algorithm, suitable for EVM chain.
pub struct Tendermint {
	step_service: IoService<Step>,
	client: RwLock<Option<Weak<EngineClient>>>,
	/// Blockchain height.
	height: AtomicUsize,
	/// Consensus view.
	view: AtomicUsize,
	/// Consensus step.
	step: RwLock<Step>,
	/// Vote accumulator.
	votes: VoteCollector<ConsensusMessage>,
	/// Used to sign messages and proposals.
	signer: RwLock<EngineSigner>,
	/// Message for the last PoLC.
	lock_change: RwLock<Option<ConsensusMessage>>,
	/// Last lock view.
	last_lock: AtomicUsize,
	/// Bare hash of the proposed block, used for seal submission.
	proposal: RwLock<Option<H256>>,
	/// Hash of the proposal parent block.
	proposal_parent: RwLock<H256>,
	/// Last block proposed by this validator.
	last_proposed: RwLock<H256>,
	/// Set used to determine the current validators.
	validators: Box<ValidatorSet>,
	/// Reward per block, in base units.
	block_reward: U256,
	/// ethereum machine descriptor
	machine: EthereumMachine,
}

struct EpochVerifier<F>
	where F: Fn(&Signature, &Message) -> Result<Address, Error> + Send + Sync
{
	subchain_validators: SimpleList,
	recover: F
}

impl <F> super::EpochVerifier<EthereumMachine> for EpochVerifier<F>
	where F: Fn(&Signature, &Message) -> Result<Address, Error> + Send + Sync
{
	fn verify_light(&self, header: &Header) -> Result<(), Error> {
		let message = header.bare_hash();

		let mut addresses = HashSet::new();
		let ref header_signatures_field = header.seal().get(2).ok_or(BlockError::InvalidSeal)?;
		for rlp in UntrustedRlp::new(header_signatures_field).iter() {
			let signature: H520 = rlp.as_val()?;
			let address = (self.recover)(&signature.into(), &message)?;

			if !self.subchain_validators.contains(header.parent_hash(), &address) {
				return Err(EngineError::NotAuthorized(address.to_owned()).into());
			}
			addresses.insert(address);
		}

		let n = addresses.len();
		let threshold = self.subchain_validators.len() * 2/3;
		if n > threshold {
			Ok(())
		} else {
			Err(EngineError::BadSealFieldSize(OutOfBounds {
				min: Some(threshold),
				max: None,
				found: n
			}).into())
		}
	}

	fn check_finality_proof(&self, proof: &[u8]) -> Option<Vec<H256>> {
		let header: Header = ::rlp::decode(proof);
		self.verify_light(&header).ok().map(|_| vec![header.hash()])
	}
}

fn combine_proofs(signal_number: BlockNumber, set_proof: &[u8], finality_proof: &[u8]) -> Vec<u8> {
	let mut stream = ::rlp::RlpStream::new_list(3);
	stream.append(&signal_number).append(&set_proof).append(&finality_proof);
	stream.out()
}

fn destructure_proofs(combined: &[u8]) -> Result<(BlockNumber, &[u8], &[u8]), Error> {
	let rlp = UntrustedRlp::new(combined);
	Ok((
		rlp.at(0)?.as_val()?,
		rlp.at(1)?.data()?,
		rlp.at(2)?.data()?,
	))
}

impl Tendermint {
	/// Create a new instance of Tendermint engine
	pub fn new(our_params: TendermintParams, machine: EthereumMachine) -> Result<Arc<Self>, Error> {
		let engine = Arc::new(
			Tendermint {
				client: RwLock::new(None),
				step_service: IoService::<Step>::start()?,
				height: AtomicUsize::new(1),
				view: AtomicUsize::new(0),
				step: RwLock::new(Step::Propose),
				votes: Default::default(),
				signer: Default::default(),
				lock_change: RwLock::new(None),
				last_lock: AtomicUsize::new(0),
				proposal: RwLock::new(None),
				proposal_parent: Default::default(),
				last_proposed: Default::default(),
				validators: our_params.validators,
				block_reward: our_params.block_reward,
				machine: machine,
			});

		let handler = TransitionHandler::new(Arc::downgrade(&engine) as Weak<Engine<_>>, Box::new(our_params.timeouts));
		engine.step_service.register_handler(Arc::new(handler))?;

		Ok(engine)
	}

	fn update_sealing(&self) {
		if let Some(ref weak) = *self.client.read() {
			if let Some(c) = weak.upgrade() {
				c.update_sealing();
			}
		}
	}

	fn submit_seal(&self, block_hash: H256, seal: Vec<Bytes>) {
		if let Some(ref weak) = *self.client.read() {
			if let Some(c) = weak.upgrade() {
				c.submit_seal(block_hash, seal);
			}
		}
	}

	fn broadcast_message(&self, message: Bytes) {
		if let Some(ref weak) = *self.client.read() {
			if let Some(c) = weak.upgrade() {
				c.broadcast_consensus_message(message);
			}
		}
	}

	fn generate_message(&self, block_hash: Option<BlockHash>) -> Option<Bytes> {
		let h = self.height.load(AtomicOrdering::SeqCst);
		let r = self.view.load(AtomicOrdering::SeqCst);
		let s = *self.step.read();
		let vote_info = message_info_rlp(&VoteStep::new(h, r, s), block_hash);
		match (self.signer.read().address(), self.sign(keccak(&vote_info)).map(Into::into)) {
			(Some(validator), Ok(signature)) => {
				let message_rlp = message_full_rlp(&signature, &vote_info);
				let message = ConsensusMessage::new(signature, h, r, s, block_hash);
				self.votes.vote(message.clone(), &validator);
				debug!(target: "engine", "Generated {:?} as {}.", message, validator);
				self.handle_valid_message(&message);

				Some(message_rlp)
			},
			(None, _) => {
				trace!(target: "engine", "No message, since there is no engine signer.");
				None
			},
			(Some(v), Err(e)) => {
				trace!(target: "engine", "{} could not sign the message {}", v, e);
				None
			},
		}
	}

	fn generate_and_broadcast_message(&self, block_hash: Option<BlockHash>) {
		if let Some(message) = self.generate_message(block_hash) {
			self.broadcast_message(message);
		}
	}

	/// Broadcast all messages since last issued block to get the peers up to speed.
	fn broadcast_old_messages(&self) {
		for m in self.votes.get_up_to(&VoteStep::new(self.height.load(AtomicOrdering::SeqCst), self.view.load(AtomicOrdering::SeqCst), Step::Precommit)).into_iter() {
			self.broadcast_message(m);
		}
	}

	fn to_next_height(&self, height: Height) {
		let new_height = height + 1;
		debug!(target: "engine", "Received a Commit, transitioning to height {}.", new_height);
		self.last_lock.store(0, AtomicOrdering::SeqCst);
		self.height.store(new_height, AtomicOrdering::SeqCst);
		self.view.store(0, AtomicOrdering::SeqCst);
		*self.lock_change.write() = None;
		*self.proposal.write() = None;
	}

	/// Use via step_service to transition steps.
	fn to_step(&self, step: Step) {
		if let Err(io_err) = self.step_service.send_message(step) {
			warn!(target: "engine", "Could not proceed to step {}.", io_err)
		}
		*self.step.write() = step;
		match step {
			Step::Propose => {
				self.update_sealing()
			},
			Step::Prevote => {
				let block_hash = match *self.lock_change.read() {
					Some(ref m) if !self.should_unlock(m.vote_step.view) => m.block_hash,
					_ => self.proposal.read().clone(),
				};
				self.generate_and_broadcast_message(block_hash);
			},
			Step::Precommit => {
				trace!(target: "engine", "to_step: Precommit.");
				let block_hash = match *self.lock_change.read() {
					Some(ref m) if self.is_view(m) && m.block_hash.is_some() => {
						trace!(target: "engine", "Setting last lock: {}", m.vote_step.view);
						self.last_lock.store(m.vote_step.view, AtomicOrdering::SeqCst);
						m.block_hash
					},
					_ => None,
				};
				self.generate_and_broadcast_message(block_hash);
			},
			Step::Commit => {
				trace!(target: "engine", "to_step: Commit.");
			},
		}
	}

	fn is_authority(&self, address: &Address) -> bool {
		self.validators.contains(&*self.proposal_parent.read(), address)
	}

	fn check_above_threshold(&self, n: usize) -> Result<(), EngineError> {
		let threshold = self.validators.count(&*self.proposal_parent.read()) * 2/3;
		if n > threshold {
			Ok(())
		} else {
			Err(EngineError::BadSealFieldSize(OutOfBounds {
				min: Some(threshold),
				max: None,
				found: n
			}))
		}
	}

	/// Find the designated for the given view.
	fn view_proposer(&self, bh: &H256, height: Height, view: View) -> Address {
		let proposer_nonce = height + view;
		trace!(target: "engine", "Proposer nonce: {}", proposer_nonce);
		self.validators.get(bh, proposer_nonce)
	}

	/// Check if address is a proposer for given view.
	fn check_view_proposer(&self, bh: &H256, height: Height, view: View, address: &Address) -> Result<(), EngineError> {
		let proposer = self.view_proposer(bh, height, view);
		if proposer == *address {
			Ok(())
		} else {
			Err(EngineError::NotProposer(Mismatch { expected: proposer, found: address.clone() }))
		}
	}

	/// Check if current signer is the current proposer.
	fn is_signer_proposer(&self, bh: &H256) -> bool {
		let proposer = self.view_proposer(bh, self.height.load(AtomicOrdering::SeqCst), self.view.load(AtomicOrdering::SeqCst));
		self.signer.read().is_address(&proposer)
	}

	fn is_height(&self, message: &ConsensusMessage) -> bool {
		message.vote_step.is_height(self.height.load(AtomicOrdering::SeqCst))
	}

	fn is_view(&self, message: &ConsensusMessage) -> bool {
		message.vote_step.is_view(self.height.load(AtomicOrdering::SeqCst), self.view.load(AtomicOrdering::SeqCst))
	}

	fn increment_view(&self, n: View) {
		trace!(target: "engine", "increment_view: New view.");
		self.view.fetch_add(n, AtomicOrdering::SeqCst);
	}

	fn should_unlock(&self, lock_change_view: View) -> bool {
		self.last_lock.load(AtomicOrdering::SeqCst) < lock_change_view
			&& lock_change_view < self.view.load(AtomicOrdering::SeqCst)
	}


	fn has_enough_any_votes(&self) -> bool {
		let step_votes = self.votes.count_round_votes(&VoteStep::new(self.height.load(AtomicOrdering::SeqCst), self.view.load(AtomicOrdering::SeqCst), *self.step.read()));
		self.check_above_threshold(step_votes).is_ok()
	}

	fn has_enough_future_step_votes(&self, vote_step: &VoteStep) -> bool {
		if vote_step.view > self.view.load(AtomicOrdering::SeqCst) {
			let step_votes = self.votes.count_round_votes(vote_step);
			self.check_above_threshold(step_votes).is_ok()
		} else {
			false
		}
	}

	fn has_enough_aligned_votes(&self, message: &ConsensusMessage) -> bool {
		let aligned_count = self.votes.count_aligned_votes(&message);
		self.check_above_threshold(aligned_count).is_ok()
	}

	fn handle_valid_message(&self, message: &ConsensusMessage) {
		let ref vote_step = message.vote_step;
		let is_newer_than_lock = match *self.lock_change.read() {
			Some(ref lock) => vote_step > &lock.vote_step,
			None => true,
		};
		let lock_change = is_newer_than_lock
			&& vote_step.step == Step::Prevote
			&& message.block_hash.is_some()
			&& self.has_enough_aligned_votes(message);
		if lock_change {
			trace!(target: "engine", "handle_valid_message: Lock change.");
			*self.lock_change.write() = Some(message.clone());
		}
		// Check if it can affect the step transition.
		if self.is_height(message) {
			let next_step = match *self.step.read() {
				Step::Precommit if message.block_hash.is_none() && self.has_enough_aligned_votes(message) => {
					self.increment_view(1);
					Some(Step::Propose)
				},
				Step::Precommit if self.has_enough_aligned_votes(message) => {
					let bh = message.block_hash.expect("previous guard ensures is_some; qed");
					if *self.last_proposed.read() == bh {
						// Commit the block using a complete signature set.
						// Generate seal and remove old votes.
						let precommits = self.votes.round_signatures(vote_step, &bh);
						trace!(target: "engine", "Collected seal: {:?}", precommits);
						let seal = vec![
							::rlp::encode(&vote_step.view).into_vec(),
							::rlp::NULL_RLP.to_vec(),
							::rlp::encode_list(&precommits).into_vec()
						];
						self.submit_seal(bh, seal);
						self.votes.throw_out_old(&vote_step);
					}
					self.to_next_height(self.height.load(AtomicOrdering::SeqCst));
					Some(Step::Commit)
				},
				Step::Precommit if self.has_enough_future_step_votes(&vote_step) => {
					self.increment_view(vote_step.view - self.view.load(AtomicOrdering::SeqCst));
					Some(Step::Precommit)
				},
				// Avoid counting votes twice.
				Step::Prevote if lock_change => Some(Step::Precommit),
				Step::Prevote if self.has_enough_aligned_votes(message) => Some(Step::Precommit),
				Step::Prevote if self.has_enough_future_step_votes(&vote_step) => {
					self.increment_view(vote_step.view - self.view.load(AtomicOrdering::SeqCst));
					Some(Step::Prevote)
				},
				_ => None,
			};

			if let Some(step) = next_step {
				trace!(target: "engine", "Transition to {:?} triggered.", step);
				self.to_step(step);
			}
		}
	}
}

impl Engine<EthereumMachine> for Tendermint {
	fn name(&self) -> &str { "Tendermint" }

	fn version(&self) -> SemanticVersion { SemanticVersion::new(1, 0, 0) }

	/// (consensus view, proposal signature, authority signatures)
	fn seal_fields(&self) -> usize { 3 }

	fn machine(&self) -> &EthereumMachine { &self.machine }

	fn maximum_uncle_count(&self) -> usize { 0 }

	fn maximum_uncle_age(&self) -> usize { 0 }

	/// Additional engine-specific information for the user/developer concerning `header`.
	fn extra_info(&self, header: &Header) -> BTreeMap<String, String> {
		let message = ConsensusMessage::new_proposal(header).expect("Invalid header.");
		map![
			"signature".into() => message.signature.to_string(),
			"height".into() => message.vote_step.height.to_string(),
			"view".into() => message.vote_step.view.to_string(),
			"block_hash".into() => message.block_hash.as_ref().map(ToString::to_string).unwrap_or("".into())
		]
	}

	fn populate_from_parent(&self, header: &mut Header, parent: &Header) {
		// Chain scoring: total weight is sqrt(U256::max_value())*height - view
		let new_difficulty = U256::from(U128::max_value())
			+ consensus_view(parent).expect("Header has been verified; qed").into()
			- self.view.load(AtomicOrdering::SeqCst).into();

		header.set_difficulty(new_difficulty);
	}

	/// Should this node participate.
	fn seals_internally(&self) -> Option<bool> {
		Some(self.signer.read().is_some())
	}

	/// Attempt to seal generate a proposal seal.
	///
	/// This operation is synchronous and may (quite reasonably) not be available, in which case
	/// `Seal::None` will be returned.
	fn generate_seal(&self, block: &ExecutedBlock) -> Seal {
		let header = block.header();
		let author = header.author();
		// Only proposer can generate seal if None was generated.
		if !self.is_signer_proposer(header.parent_hash()) || self.proposal.read().is_some() {
			return Seal::None;
		}

		let height = header.number() as Height;
		let view = self.view.load(AtomicOrdering::SeqCst);
		let bh = Some(header.bare_hash());
		let vote_info = message_info_rlp(&VoteStep::new(height, view, Step::Propose), bh.clone());
		if let Ok(signature) = self.sign(keccak(&vote_info)).map(Into::into) {
			// Insert Propose vote.
			debug!(target: "engine", "Submitting proposal {} at height {} view {}.", header.bare_hash(), height, view);
			self.votes.vote(ConsensusMessage::new(signature, height, view, Step::Propose, bh), author);
			// Remember the owned block.
			*self.last_proposed.write() = header.bare_hash();
			// Remember proposal for later seal submission.
			*self.proposal.write() = bh;
			*self.proposal_parent.write() = header.parent_hash().clone();
			Seal::Proposal(vec![
				::rlp::encode(&view).into_vec(),
				::rlp::encode(&signature).into_vec(),
				::rlp::EMPTY_LIST_RLP.to_vec()
			])
		} else {
			warn!(target: "engine", "generate_seal: FAIL: accounts secret key unavailable");
			Seal::None
		}
	}

	fn handle_message(&self, rlp: &[u8]) -> Result<(), EngineError> {
		fn fmt_err<T: ::std::fmt::Debug>(x: T) -> EngineError {
			EngineError::MalformedMessage(format!("{:?}", x))
		}

		let rlp = UntrustedRlp::new(rlp);
		let message: ConsensusMessage = rlp.as_val().map_err(fmt_err)?;
		if !self.votes.is_old_or_known(&message) {
			let msg_hash = keccak(rlp.at(1).map_err(fmt_err)?.as_raw());
			let sender = public_to_address(
				&recover(&message.signature.into(), &msg_hash).map_err(fmt_err)?
			);

			if !self.is_authority(&sender) {
				return Err(EngineError::NotAuthorized(sender));
			}
			self.broadcast_message(rlp.as_raw().to_vec());
			if let Some(double) = self.votes.vote(message.clone(), &sender) {
				let height = message.vote_step.height as BlockNumber;
				self.validators.report_malicious(&sender, height, height, ::rlp::encode(&double).into_vec());
				return Err(EngineError::DoubleVote(sender));
			}
			trace!(target: "engine", "Handling a valid {:?} from {}.", message, sender);
			self.handle_valid_message(&message);
		}
		Ok(())
	}

	fn on_new_block(&self, block: &mut ExecutedBlock, epoch_begin: bool) -> Result<(), Error> {
		if !epoch_begin { return Ok(()) }

		// genesis is never a new block, but might as well check.
		let header = block.fields().header.clone();
		let first = header.number() == 0;

		let mut call = |to, data| {
			let result = self.machine.execute_as_system(
				block,
				to,
				U256::max_value(), // unbounded gas? maybe make configurable.
				Some(data),
			);

			result.map_err(|e| format!("{}", e))
		};

		self.validators.on_epoch_begin(first, &header, &mut call)
	}

	/// Apply the block reward on finalisation of the block.
	fn on_close_block(&self, block: &mut ExecutedBlock) -> Result<(), Error>{
		::engines::common::bestow_block_reward(block, self.block_reward)
	}

	fn verify_local_seal(&self, _header: &Header) -> Result<(), Error> {
		Ok(())
	}

	fn verify_block_basic(&self, header: &Header) -> Result<(), Error> {
		let seal_length = header.seal().len();
		if seal_length == self.seal_fields() {
			// Either proposal or commit.
			if (header.seal()[1] == ::rlp::NULL_RLP)
				!= (header.seal()[2] == ::rlp::EMPTY_LIST_RLP) {
				Ok(())
			} else {
				warn!(target: "engine", "verify_block_basic: Block is neither a Commit nor Proposal.");
				Err(BlockError::InvalidSeal.into())
			}
		} else {
			Err(BlockError::InvalidSealArity(
				Mismatch { expected: self.seal_fields(), found: seal_length }
			).into())
		}
	}

	fn verify_block_external(&self, header: &Header) -> Result<(), Error> {
		if let Ok(proposal) = ConsensusMessage::new_proposal(header) {
			let proposer = proposal.verify()?;
			if !self.is_authority(&proposer) {
				return Err(EngineError::NotAuthorized(proposer).into());
			}
			self.check_view_proposer(
				header.parent_hash(),
				proposal.vote_step.height,
				proposal.vote_step.view,
				&proposer
			).map_err(Into::into)
		} else {
			let vote_step = VoteStep::new(header.number() as usize, consensus_view(header)?, Step::Precommit);
			let precommit_hash = message_hash(vote_step.clone(), header.bare_hash());
			let ref signatures_field = header.seal().get(2).expect("block went through verify_block_basic; block has .seal_fields() fields; qed");
			let mut origins = HashSet::new();
			for rlp in UntrustedRlp::new(signatures_field).iter() {
				let precommit = ConsensusMessage {
					signature: rlp.as_val()?,
					block_hash: Some(header.bare_hash()),
					vote_step: vote_step.clone(),
				};
				let address = match self.votes.get(&precommit) {
					Some(a) => a,
					None => public_to_address(&recover(&precommit.signature.into(), &precommit_hash)?),
				};
				if !self.validators.contains(header.parent_hash(), &address) {
					return Err(EngineError::NotAuthorized(address.to_owned()).into());
				}

				if !origins.insert(address) {
					warn!(target: "engine", "verify_block_unordered: Duplicate signature from {} on the seal.", address);
					return Err(BlockError::InvalidSeal.into());
				}
			}

			self.check_above_threshold(origins.len()).map_err(Into::into)
		}
	}

	fn signals_epoch_end(&self, header: &Header, aux: AuxiliaryData)
		-> super::EpochChange<EthereumMachine>
	{
		let first = header.number() == 0;
		self.validators.signals_epoch_end(first, header, aux)
	}

	fn is_epoch_end(
		&self,
		chain_head: &Header,
		_chain: &super::Headers<Header>,
		transition_store: &super::PendingTransitionStore,
	) -> Option<Vec<u8>> {
		let first = chain_head.number() == 0;

		if let Some(change) = self.validators.is_epoch_end(first, chain_head) {
			let change = combine_proofs(chain_head.number(), &change, &[]);
			return Some(change)
		} else if let Some(pending) = transition_store(chain_head.hash()) {
			let signal_number = chain_head.number();
			let finality_proof = ::rlp::encode(chain_head);
			return Some(combine_proofs(signal_number, &pending.proof, &finality_proof))
		}

		None
	}

	fn epoch_verifier<'a>(&self, _header: &Header, proof: &'a [u8]) -> ConstructedVerifier<'a, EthereumMachine> {
		let (signal_number, set_proof, finality_proof) = match destructure_proofs(proof) {
			Ok(x) => x,
			Err(e) => return ConstructedVerifier::Err(e),
		};

		let first = signal_number == 0;
		match self.validators.epoch_set(first, &self.machine, signal_number, set_proof) {
			Ok((list, finalize)) => {
				let verifier = Box::new(EpochVerifier {
					subchain_validators: list,
					recover: |signature: &Signature, message: &Message| {
						Ok(public_to_address(&::ethkey::recover(&signature, &message)?))
					},
				});

				match finalize {
					Some(finalize) => ConstructedVerifier::Unconfirmed(verifier, finality_proof, finalize),
					None => ConstructedVerifier::Trusted(verifier),
				}
			}
			Err(e) => ConstructedVerifier::Err(e),
		}
	}

	fn set_signer(&self, ap: Arc<AccountProvider>, address: Address, password: String) {
		{
			self.signer.write().set(ap, address, password);
		}
		self.to_step(Step::Propose);
	}

	fn sign(&self, hash: H256) -> Result<Signature, Error> {
		self.signer.read().sign(hash).map_err(Into::into)
	}

	fn snapshot_components(&self) -> Option<Box<::snapshot::SnapshotComponents>> {
		Some(Box::new(::snapshot::PoaSnapshot))
	}

	fn stop(&self) {
		self.step_service.stop()
	}

	fn is_proposal(&self, header: &Header) -> bool {
		let signatures_len = header.seal()[2].len();
		// Signatures have to be an empty list rlp.
		if signatures_len != 1 {
			// New Commit received, skip to next height.
			trace!(target: "engine", "Received a commit: {:?}.", header.number());
			self.to_next_height(header.number() as usize);
			self.to_step(Step::Commit);
			return false;
		}
		let proposal = ConsensusMessage::new_proposal(header).expect("block went through full verification; this Engine verifies new_proposal creation; qed");
		let proposer = proposal.verify().expect("block went through full verification; this Engine tries verify; qed");
		debug!(target: "engine", "Received a new proposal {:?} from {}.", proposal.vote_step, proposer);
		if self.is_view(&proposal) {
			*self.proposal.write() = proposal.block_hash.clone();
			*self.proposal_parent.write() = header.parent_hash().clone();
		}
		self.votes.vote(proposal, &proposer);
		true
	}

	/// Equivalent to a timeout: to be used for tests.
	fn step(&self) {
		let next_step = match *self.step.read() {
			Step::Propose => {
				trace!(target: "engine", "Propose timeout.");
				if self.proposal.read().is_none() {
					// Report the proposer if no proposal was received.
					let height = self.height.load(AtomicOrdering::SeqCst);
					let current_proposer = self.view_proposer(&*self.proposal_parent.read(), height, self.view.load(AtomicOrdering::SeqCst));
					self.validators.report_benign(&current_proposer, height as BlockNumber, height as BlockNumber);
				}
				Step::Prevote
			},
			Step::Prevote if self.has_enough_any_votes() => {
				trace!(target: "engine", "Prevote timeout.");
				Step::Precommit
			},
			Step::Prevote => {
				trace!(target: "engine", "Prevote timeout without enough votes.");
				self.broadcast_old_messages();
				Step::Prevote
			},
			Step::Precommit if self.has_enough_any_votes() => {
				trace!(target: "engine", "Precommit timeout.");
				self.increment_view(1);
				Step::Propose
			},
			Step::Precommit => {
				trace!(target: "engine", "Precommit timeout without enough votes.");
				self.broadcast_old_messages();
				Step::Precommit
			},
			Step::Commit => {
				trace!(target: "engine", "Commit timeout.");
				Step::Propose
			},
		};
		self.to_step(next_step);
	}

	fn register_client(&self, client: Weak<EngineClient>) {
		if let Some(c) = client.upgrade() {
			self.height.store(c.chain_info().best_block_number as usize + 1, AtomicOrdering::SeqCst);
		}
		*self.client.write() = Some(client.clone());
		self.validators.register_client(client);
	}
}

#[cfg(test)]
mod tests {
	use std::str::FromStr;
	use rustc_hex::FromHex;
	use util::*;
	use bytes::Bytes;
	use block::*;
	use error::{Error, BlockError};
	use header::Header;
	use client::chain_notify::ChainNotify;
	use miner::MinerService;
	use tests::helpers::*;
	use account_provider::AccountProvider;
	use spec::Spec;
	use engines::{EthEngine, EngineError, Seal};
	use engines::epoch::EpochVerifier;
	use super::*;

	/// Accounts inserted with "0" and "1" are validators. First proposer is "0".
	fn setup() -> (Spec, Arc<AccountProvider>) {
		let tap = Arc::new(AccountProvider::transient_provider());
		let spec = Spec::new_test_tendermint();
		(spec, tap)
	}

	fn propose_default(spec: &Spec, proposer: Address) -> (ClosedBlock, Vec<Bytes>) {
		let db = get_temp_state_db();
		let db = spec.ensure_db_good(db, &Default::default()).unwrap();
		let genesis_header = spec.genesis_header();
		let last_hashes = Arc::new(vec![genesis_header.hash()]);
		let b = OpenBlock::new(spec.engine.as_ref(), Default::default(), false, db.boxed_clone(), &genesis_header, last_hashes, proposer, (3141562.into(), 31415620.into()), vec![], false).unwrap();
		let b = b.close();
		if let Seal::Proposal(seal) = spec.engine.generate_seal(b.block()) {
			(b, seal)
		} else {
			panic!()
		}
	}

	fn vote<F>(engine: &EthEngine, signer: F, height: usize, view: usize, step: Step, block_hash: Option<H256>) -> Bytes where F: FnOnce(H256) -> Result<H520, ::account_provider::SignError> {
		let mi = message_info_rlp(&VoteStep::new(height, view, step), block_hash);
		let m = message_full_rlp(&signer(keccak(&mi)).unwrap().into(), &mi);
		engine.handle_message(&m).unwrap();
		m
	}

	fn proposal_seal(tap: &Arc<AccountProvider>, header: &Header, view: View) -> Vec<Bytes> {
		let author = header.author();
		let vote_info = message_info_rlp(&VoteStep::new(header.number() as Height, view, Step::Propose), Some(header.bare_hash()));
		let signature = tap.sign(*author, None, keccak(vote_info)).unwrap();
		vec![
			::rlp::encode(&view).into_vec(),
			::rlp::encode(&H520::from(signature)).into_vec(),
			::rlp::EMPTY_LIST_RLP.to_vec()
		]
	}

	fn insert_and_unlock(tap: &Arc<AccountProvider>, acc: &str) -> Address {
		let addr = tap.insert_account(keccak(acc).into(), acc).unwrap();
		tap.unlock_account_permanently(addr, acc.into()).unwrap();
		addr
	}

	fn insert_and_register(tap: &Arc<AccountProvider>, engine: &EthEngine, acc: &str) -> Address {
		let addr = insert_and_unlock(tap, acc);
		engine.set_signer(tap.clone(), addr.clone(), acc.into());
		addr
	}

	#[derive(Default)]
	struct TestNotify {
		messages: RwLock<Vec<Bytes>>,
	}

	impl ChainNotify for TestNotify {
		fn broadcast(&self, data: Vec<u8>) {
			self.messages.write().push(data);
		}
	}

	#[test]
	fn has_valid_metadata() {
		let engine = Spec::new_test_tendermint().engine;
		assert!(!engine.name().is_empty());
		assert!(engine.version().major >= 1);
	}

	#[test]
	fn can_return_schedule() {
		let engine = Spec::new_test_tendermint().engine;
		let schedule = engine.schedule(10000000);

		assert!(schedule.stack_limit > 0);
	}

	#[test]
	fn verification_fails_on_short_seal() {
		let engine = Spec::new_test_tendermint().engine;
		let header = Header::default();

		let verify_result = engine.verify_block_basic(&header);

		match verify_result {
			Err(Error::Block(BlockError::InvalidSealArity(_))) => {},
			Err(_) => { panic!("should be block seal-arity mismatch error (got {:?})", verify_result); },
			_ => { panic!("Should be error, got Ok"); },
		}
	}

	#[test]
	fn allows_correct_proposer() {
		let (spec, tap) = setup();
		let engine = spec.engine;

		let mut parent_header: Header = Header::default();
		parent_header.set_gas_limit(U256::from_str("222222").unwrap());

		let mut header = Header::default();
		header.set_number(1);
		header.set_gas_limit(U256::from_str("222222").unwrap());
		let validator = insert_and_unlock(&tap, "1");
		header.set_author(validator);
		let seal = proposal_seal(&tap, &header, 0);
		header.set_seal(seal);
		// Good proposer.
		assert!(engine.verify_block_external(&header).is_ok());

		let validator = insert_and_unlock(&tap, "0");
		header.set_author(validator);
		let seal = proposal_seal(&tap, &header, 0);
		header.set_seal(seal);
		// Bad proposer.
		match engine.verify_block_external(&header) {
			Err(Error::Engine(EngineError::NotProposer(_))) => {},
			_ => panic!(),
		}

		let random = insert_and_unlock(&tap, "101");
		header.set_author(random);
		let seal = proposal_seal(&tap, &header, 0);
		header.set_seal(seal);
		// Not authority.
		match engine.verify_block_external(&header) {
			Err(Error::Engine(EngineError::NotAuthorized(_))) => {},
			_ => panic!(),
		};
		engine.stop();
	}

	#[test]
	fn seal_signatures_checking() {
		let (spec, tap) = setup();
		let engine = spec.engine;

		let mut parent_header: Header = Header::default();
		parent_header.set_gas_limit(U256::from_str("222222").unwrap());

		let mut header = Header::default();
		header.set_number(2);
		header.set_gas_limit(U256::from_str("222222").unwrap());
		let proposer = insert_and_unlock(&tap, "1");
		header.set_author(proposer);
		let mut seal = proposal_seal(&tap, &header, 0);

		let vote_info = message_info_rlp(&VoteStep::new(2, 0, Step::Precommit), Some(header.bare_hash()));
		let signature1 = tap.sign(proposer, None, keccak(&vote_info)).unwrap();

		seal[1] = ::rlp::NULL_RLP.to_vec();
		seal[2] = ::rlp::encode_list(&vec![H520::from(signature1.clone())]).into_vec();
		header.set_seal(seal.clone());

		// One good signature is not enough.
		match engine.verify_block_external(&header) {
			Err(Error::Engine(EngineError::BadSealFieldSize(_))) => {},
			_ => panic!(),
		}

		let voter = insert_and_unlock(&tap, "0");
		let signature0 = tap.sign(voter, None, keccak(&vote_info)).unwrap();

		seal[2] = ::rlp::encode_list(&vec![H520::from(signature1.clone()), H520::from(signature0.clone())]).into_vec();
		header.set_seal(seal.clone());

		assert!(engine.verify_block_external(&header).is_ok());

		let bad_voter = insert_and_unlock(&tap, "101");
		let bad_signature = tap.sign(bad_voter, None, keccak(vote_info)).unwrap();

		seal[2] = ::rlp::encode_list(&vec![H520::from(signature1), H520::from(bad_signature)]).into_vec();
		header.set_seal(seal);

		// One good and one bad signature.
		match engine.verify_block_external(&header) {
			Err(Error::Engine(EngineError::NotAuthorized(_))) => {},
			_ => panic!(),
		};
		engine.stop();
	}

	#[test]
	fn can_generate_seal() {
		let (spec, tap) = setup();

		let proposer = insert_and_register(&tap, spec.engine.as_ref(), "1");

		let (b, seal) = propose_default(&spec, proposer);
		assert!(b.lock().try_seal(spec.engine.as_ref(), seal).is_ok());
	}

	#[test]
	fn can_recognize_proposal() {
		let (spec, tap) = setup();

		let proposer = insert_and_register(&tap, spec.engine.as_ref(), "1");

		let (b, seal) = propose_default(&spec, proposer);
		let sealed = b.lock().seal(spec.engine.as_ref(), seal).unwrap();
		assert!(spec.engine.is_proposal(sealed.header()));
	}

	#[test]
	fn relays_messages() {
		let (spec, tap) = setup();
		let engine = spec.engine.clone();

		let v0 = insert_and_unlock(&tap, "0");
		let v1 = insert_and_register(&tap, engine.as_ref(), "1");

		let h = 1;
		let r = 0;

		// Propose
		let (b, _) = propose_default(&spec, v1.clone());
		let proposal = Some(b.header().bare_hash());

		let client = generate_dummy_client(0);
		let notify = Arc::new(TestNotify::default());
		client.add_notify(notify.clone());
		engine.register_client(Arc::downgrade(&client) as _);

		let prevote_current = vote(engine.as_ref(), |mh| tap.sign(v0, None, mh).map(H520::from), h, r, Step::Prevote, proposal);

		let precommit_current = vote(engine.as_ref(), |mh| tap.sign(v0, None, mh).map(H520::from), h, r, Step::Precommit, proposal);

		let prevote_future = vote(engine.as_ref(), |mh| tap.sign(v0, None, mh).map(H520::from), h + 1, r, Step::Prevote, proposal);

		// Relays all valid present and future messages.
		assert!(notify.messages.read().contains(&prevote_current));
		assert!(notify.messages.read().contains(&precommit_current));
		assert!(notify.messages.read().contains(&prevote_future));
	}

	#[test]
	fn seal_submission() {
		use ethkey::{Generator, Random};
		use transaction::{Transaction, Action};

		let tap = Arc::new(AccountProvider::transient_provider());
		// Accounts for signing votes.
		let v0 = insert_and_unlock(&tap, "0");
		let v1 = insert_and_unlock(&tap, "1");
		let client = generate_dummy_client_with_spec_and_accounts(Spec::new_test_tendermint, Some(tap.clone()));
		let engine = client.engine();

		client.miner().set_engine_signer(v1.clone(), "1".into()).unwrap();

		let notify = Arc::new(TestNotify::default());
		client.add_notify(notify.clone());
		engine.register_client(Arc::downgrade(&client) as _);

		let keypair = Random.generate().unwrap();
		let transaction = Transaction {
			action: Action::Create,
			value: U256::zero(),
			data: "3331600055".from_hex().unwrap(),
			gas: U256::from(100_000),
			gas_price: U256::zero(),
			nonce: U256::zero(),
		}.sign(keypair.secret(), None);
		client.miner().import_own_transaction(client.as_ref(), transaction.into()).unwrap();

		// Propose
		let proposal = Some(client.miner().pending_block(0).unwrap().header.bare_hash());
		// Propose timeout
		engine.step();

		let h = 1;
		let r = 0;

		// Prevote.
		vote(engine, |mh| tap.sign(v1, None, mh).map(H520::from), h, r, Step::Prevote, proposal);
		vote(engine, |mh| tap.sign(v0, None, mh).map(H520::from), h, r, Step::Prevote, proposal);
		vote(engine, |mh| tap.sign(v1, None, mh).map(H520::from), h, r, Step::Precommit, proposal);

		assert_eq!(client.chain_info().best_block_number, 0);
		// Last precommit.
		vote(engine, |mh| tap.sign(v0, None, mh).map(H520::from), h, r, Step::Precommit, proposal);
		assert_eq!(client.chain_info().best_block_number, 1);
	}

	#[test]
	fn epoch_verifier_verify_light() {
		use ethkey::Error as EthkeyError;

		let (spec, tap) = setup();
		let engine = spec.engine;

		let mut parent_header: Header = Header::default();
		parent_header.set_gas_limit(U256::from_str("222222").unwrap());

		let mut header = Header::default();
		header.set_number(2);
		header.set_gas_limit(U256::from_str("222222").unwrap());
		let proposer = insert_and_unlock(&tap, "1");
		header.set_author(proposer);
		let mut seal = proposal_seal(&tap, &header, 0);

		let vote_info = message_info_rlp(&VoteStep::new(2, 0, Step::Precommit), Some(header.bare_hash()));
		let signature1 = tap.sign(proposer, None, keccak(&vote_info)).unwrap();

		let voter = insert_and_unlock(&tap, "0");
		let signature0 = tap.sign(voter, None, keccak(&vote_info)).unwrap();

		seal[1] = ::rlp::NULL_RLP.to_vec();
		seal[2] = ::rlp::encode_list(&vec![H520::from(signature1.clone())]).into_vec();
		header.set_seal(seal.clone());

		let epoch_verifier = super::EpochVerifier {
			subchain_validators: SimpleList::new(vec![proposer.clone(), voter.clone()]),
			recover: {
				let signature1 = signature1.clone();
				let signature0 = signature0.clone();
				let proposer = proposer.clone();
				let voter = voter.clone();
				move |s: &Signature, _: &Message| {
					if *s == signature1 {
						Ok(proposer)
					} else if *s == signature0 {
						Ok(voter)
					} else {
						Err(Error::Ethkey(EthkeyError::InvalidSignature))
					}
				}
			},
		};

		// One good signature is not enough.
		match epoch_verifier.verify_light(&header) {
			Err(Error::Engine(EngineError::BadSealFieldSize(_))) => {},
			_ => panic!(),
		}

		seal[2] = ::rlp::encode_list(&vec![H520::from(signature1.clone()), H520::from(signature0.clone())]).into_vec();
		header.set_seal(seal.clone());

		assert!(epoch_verifier.verify_light(&header).is_ok());

		let bad_voter = insert_and_unlock(&tap, "101");
		let bad_signature = tap.sign(bad_voter, None, keccak(&vote_info)).unwrap();

		seal[2] = ::rlp::encode_list(&vec![H520::from(signature1), H520::from(bad_signature)]).into_vec();
		header.set_seal(seal);

		// One good and one bad signature.
		match epoch_verifier.verify_light(&header) {
			Err(Error::Ethkey(EthkeyError::InvalidSignature)) => {},
			_ => panic!(),
		};

		engine.stop();
	}
}
