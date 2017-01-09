// Copyright 2015, 2016 Ethcore (UK) Ltd.
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
/// At each blockchain `Height` there can be multiple `Round`s of voting.
/// Signatures always sign `Height`, `Round`, `Step` and `BlockHash` which is a block hash without seal.
/// First a block with `Seal::Proposal` is issued by the designated proposer.
/// Next the `Round` proceeds through `Prevote` and `Precommit` `Step`s.
/// Block is issued when there is enough `Precommit` votes collected on a particular block at the end of a `Round`.
/// Once enough votes have been gathered the proposer issues that block in the `Commit` step.

mod message;
mod transition;
mod params;
mod vote_collector;

use std::sync::Weak;
use std::sync::atomic::{AtomicUsize, Ordering as AtomicOrdering};
use util::*;
use client::{Client, EngineClient};
use error::{Error, BlockError};
use header::Header;
use builtin::Builtin;
use env_info::EnvInfo;
use transaction::SignedTransaction;
use rlp::{UntrustedRlp, View};
use ethkey::{recover, public_to_address};
use account_provider::AccountProvider;
use block::*;
use spec::CommonParams;
use engines::{Engine, Seal, EngineError};
use blockchain::extras::BlockDetails;
use views::HeaderView;
use evm::Schedule;
use state::CleanupMode;
use io::IoService;
use super::validator_set::{ValidatorSet, new_validator_set};
use self::message::*;
use self::transition::TransitionHandler;
use self::params::TendermintParams;
use self::vote_collector::VoteCollector;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
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
pub type Round = usize;
pub type BlockHash = H256;

/// Engine using `Tendermint` consensus algorithm, suitable for EVM chain.
pub struct Tendermint {
	params: CommonParams,
	gas_limit_bound_divisor: U256,
	builtins: BTreeMap<Address, Builtin>,
	step_service: IoService<Step>,
	client: RwLock<Option<Weak<EngineClient>>>,
	block_reward: U256,
	/// Address to be used as authority.
	authority: RwLock<Address>,
	/// Password used for signing messages.
	password: RwLock<Option<String>>,
	/// Blockchain height.
	height: AtomicUsize,
	/// Consensus round.
	round: AtomicUsize,
	/// Consensus step.
	step: RwLock<Step>,
	/// Vote accumulator.
	votes: VoteCollector,
	/// Used to sign messages and proposals.
	account_provider: Mutex<Option<Arc<AccountProvider>>>,
	/// Message for the last PoLC.
	lock_change: RwLock<Option<ConsensusMessage>>,
	/// Last lock round.
	last_lock: AtomicUsize,
	/// Bare hash of the proposed block, used for seal submission.
	proposal: RwLock<Option<H256>>,
	/// Set used to determine the current validators.
	validators: Box<ValidatorSet + Send + Sync>,
}

impl Tendermint {
	/// Create a new instance of Tendermint engine
	pub fn new(params: CommonParams, our_params: TendermintParams, builtins: BTreeMap<Address, Builtin>) -> Result<Arc<Self>, Error> {
		let engine = Arc::new(
			Tendermint {
				params: params,
				gas_limit_bound_divisor: our_params.gas_limit_bound_divisor,
				builtins: builtins,
				client: RwLock::new(None),
				step_service: IoService::<Step>::start()?,
				block_reward: our_params.block_reward,
				authority: RwLock::new(Address::default()),
				password: RwLock::new(None),
				height: AtomicUsize::new(1),
				round: AtomicUsize::new(0),
				step: RwLock::new(Step::Propose),
				votes: VoteCollector::new(),
				account_provider: Mutex::new(None),
				lock_change: RwLock::new(None),
				last_lock: AtomicUsize::new(0),
				proposal: RwLock::new(None),
				validators: new_validator_set(our_params.validators),
			});
		let handler = TransitionHandler::new(Arc::downgrade(&engine), our_params.timeouts);
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
		if let Some(ref ap) = *self.account_provider.lock() {
			let h = self.height.load(AtomicOrdering::SeqCst);
			let r = self.round.load(AtomicOrdering::SeqCst);
			let s = self.step.read();
			let vote_info = message_info_rlp(h, r, *s, block_hash);
			let authority = self.authority.read();
			match ap.sign(*authority, self.password.read().clone(), vote_info.sha3()).map(Into::into) {
				Ok(signature) => {
					let message_rlp = message_full_rlp(&signature, &vote_info);
					let message = ConsensusMessage::new(signature, h, r, *s, block_hash);
					self.votes.vote(message.clone(), *authority);
					debug!(target: "poa", "Generated {:?} as {}.", message, *authority);
					self.handle_valid_message(&message);

					Some(message_rlp)
				},
				Err(e) => {
					trace!(target: "poa", "Could not sign the message {}", e);
					None
				},
			}
		} else {
			warn!(target: "poa", "No AccountProvider available.");
			None
		}
	}

	fn generate_and_broadcast_message(&self, block_hash: Option<BlockHash>) {
		if let Some(message) = self.generate_message(block_hash) {
			self.broadcast_message(message);
		}
	}

	/// Broadcast all messages since last issued block to get the peers up to speed.
	fn broadcast_old_messages(&self) {
		for m in self.votes.get_up_to(self.height.load(AtomicOrdering::SeqCst)).into_iter() {
			self.broadcast_message(m);
		}
	}

	fn to_next_height(&self, height: Height) {
		let new_height = height + 1;
		debug!(target: "poa", "Received a Commit, transitioning to height {}.", new_height);
		self.last_lock.store(0, AtomicOrdering::SeqCst);
		self.height.store(new_height, AtomicOrdering::SeqCst);
		self.round.store(0, AtomicOrdering::SeqCst);
		*self.lock_change.write() = None;
	}

	/// Use via step_service to transition steps.
	fn to_step(&self, step: Step) {
		if let Err(io_err) = self.step_service.send_message(step) {
			warn!(target: "poa", "Could not proceed to step {}.", io_err)
		}
		*self.step.write() = step;
		match step {
			Step::Propose => {
				*self.proposal.write() = None;
				self.update_sealing()
			},
			Step::Prevote => {
				let block_hash = match *self.lock_change.read() {
					Some(ref m) if !self.should_unlock(m.round) => m.block_hash,
					_ => self.proposal.read().clone(),
				};
				self.generate_and_broadcast_message(block_hash);
			},
			Step::Precommit => {
				trace!(target: "poa", "to_step: Precommit.");
				let block_hash = match *self.lock_change.read() {
					Some(ref m) if self.is_round(m) && m.block_hash.is_some() => {
						trace!(target: "poa", "Setting last lock: {}", m.round);
						self.last_lock.store(m.round, AtomicOrdering::SeqCst);
						m.block_hash
					},
					_ => None,
				};
				self.generate_and_broadcast_message(block_hash);
			},
			Step::Commit => {
				trace!(target: "poa", "to_step: Commit.");
				// Commit the block using a complete signature set.
				let round = self.round.load(AtomicOrdering::SeqCst);
				let height = self.height.load(AtomicOrdering::SeqCst);
				if let Some(block_hash) = *self.proposal.read() {
					// Generate seal and remove old votes.
					if self.is_proposer(&*self.authority.read()).is_ok() {
						if let Some(seal) = self.votes.seal_signatures(height, round, block_hash) {
							trace!(target: "poa", "Collected seal: {:?}", seal);
							let seal = vec![
								::rlp::encode(&round).to_vec(),
								::rlp::encode(&seal.proposal).to_vec(),
								::rlp::encode(&seal.votes).to_vec()
							];
							self.submit_seal(block_hash, seal);
							self.to_next_height(height);
						} else {
							warn!(target: "poa", "Not enough votes found!");
						}
					}
				}
			},
		}
	}

	fn is_authority(&self, address: &Address) -> bool {
		self.validators.contains(address)
	}

	fn is_above_threshold(&self, n: usize) -> bool {
		n > self.validators.count() * 2/3
	}

	/// Check if address is a proposer for given round.
	fn is_round_proposer(&self, height: Height, round: Round, address: &Address) -> Result<(), EngineError> {
		let proposer_nonce = height + round;
		trace!(target: "poa", "is_proposer: Proposer nonce: {}", proposer_nonce);
		let proposer = self.validators.get(proposer_nonce);
		if proposer == *address {
			Ok(())
		} else {
			Err(EngineError::NotProposer(Mismatch { expected: proposer, found: address.clone() }))
		}
	}

	/// Check if address is the current proposer.
	fn is_proposer(&self, address: &Address) -> Result<(), EngineError> {
		self.is_round_proposer(self.height.load(AtomicOrdering::SeqCst), self.round.load(AtomicOrdering::SeqCst), address)
	}

	fn is_height(&self, message: &ConsensusMessage) -> bool {
		message.is_height(self.height.load(AtomicOrdering::SeqCst)) 
	}

	fn is_round(&self, message: &ConsensusMessage) -> bool {
		message.is_round(self.height.load(AtomicOrdering::SeqCst), self.round.load(AtomicOrdering::SeqCst)) 
	}

	fn increment_round(&self, n: Round) {
		trace!(target: "poa", "increment_round: New round.");
		self.round.fetch_add(n, AtomicOrdering::SeqCst);
	}

	fn should_unlock(&self, lock_change_round: Round) -> bool { 
		self.last_lock.load(AtomicOrdering::SeqCst) < lock_change_round
			&& lock_change_round < self.round.load(AtomicOrdering::SeqCst)
	}


	fn has_enough_any_votes(&self) -> bool {
		let step_votes = self.votes.count_step_votes(self.height.load(AtomicOrdering::SeqCst), self.round.load(AtomicOrdering::SeqCst), *self.step.read());
		self.is_above_threshold(step_votes)
	}

	fn has_enough_future_step_votes(&self, message: &ConsensusMessage) -> bool {
		if message.round > self.round.load(AtomicOrdering::SeqCst) {
			let step_votes = self.votes.count_step_votes(message.height, message.round, message.step);
			self.is_above_threshold(step_votes)	
		} else {
			false
		}
	}

	fn has_enough_aligned_votes(&self, message: &ConsensusMessage) -> bool {
		let aligned_count = self.votes.count_aligned_votes(&message);
		self.is_above_threshold(aligned_count)
	}

	fn handle_valid_message(&self, message: &ConsensusMessage) {
		let is_newer_than_lock = match *self.lock_change.read() {
			Some(ref lock) => message > lock,
			None => true,
		};
		let lock_change = is_newer_than_lock
			&& message.step == Step::Prevote
			&& message.block_hash.is_some()
			&& self.has_enough_aligned_votes(message);
		if lock_change {
			trace!(target: "poa", "handle_valid_message: Lock change.");
			*self.lock_change.write()	= Some(message.clone());
		}
		// Check if it can affect the step transition.
		if self.is_height(message) {
			let next_step = match *self.step.read() {
				Step::Precommit if self.has_enough_aligned_votes(message) => {
					if message.block_hash.is_none() {
						self.increment_round(1);
						Some(Step::Propose)
					} else {
						Some(Step::Commit)
					}
				},
				Step::Precommit if self.has_enough_future_step_votes(message) => {
					self.increment_round(message.round - self.round.load(AtomicOrdering::SeqCst));
					Some(Step::Precommit)
				},
				// Avoid counting twice.
				Step::Prevote if lock_change => Some(Step::Precommit),
				Step::Prevote if self.has_enough_aligned_votes(message) => Some(Step::Precommit),
				Step::Prevote if self.has_enough_future_step_votes(message) => {
					self.increment_round(message.round - self.round.load(AtomicOrdering::SeqCst));
					Some(Step::Prevote)
				},
				_ => None,
			};

			if let Some(step) = next_step {
				trace!(target: "poa", "Transition to {:?} triggered.", step);
				self.to_step(step);
			}
		}
	}
}

impl Engine for Tendermint {
	fn name(&self) -> &str { "Tendermint" }
	fn version(&self) -> SemanticVersion { SemanticVersion::new(1, 0, 0) }
	/// (consensus round, proposal signature, authority signatures)
	fn seal_fields(&self) -> usize { 3 }

	fn params(&self) -> &CommonParams { &self.params }
	fn builtins(&self) -> &BTreeMap<Address, Builtin> { &self.builtins }

	fn maximum_uncle_count(&self) -> usize { 0 }
	fn maximum_uncle_age(&self) -> usize { 0 }

	/// Additional engine-specific information for the user/developer concerning `header`.
	fn extra_info(&self, header: &Header) -> BTreeMap<String, String> {
		let message = ConsensusMessage::new_proposal(header).expect("Invalid header.");
		map![
			"signature".into() => message.signature.to_string(),
			"height".into() => message.height.to_string(),
			"round".into() => message.round.to_string(),
			"block_hash".into() => message.block_hash.as_ref().map(ToString::to_string).unwrap_or("".into())
		]
	}

	fn schedule(&self, _env_info: &EnvInfo) -> Schedule {
		Schedule::new_post_eip150(usize::max_value(), true, true, true)
	}

	fn populate_from_parent(&self, header: &mut Header, parent: &Header, gas_floor_target: U256, _gas_ceil_target: U256) {
		header.set_difficulty(parent.difficulty().clone());
		header.set_gas_limit({
			let gas_limit = parent.gas_limit().clone();
			let bound_divisor = self.gas_limit_bound_divisor;
			if gas_limit < gas_floor_target {
				min(gas_floor_target, gas_limit + gas_limit / bound_divisor - 1.into())
			} else {
				max(gas_floor_target, gas_limit - gas_limit / bound_divisor + 1.into())
			}
		});
	}

	/// Should this node participate.
	fn is_sealer(&self, address: &Address) -> Option<bool> {
		Some(self.is_authority(address))
	}

	/// Attempt to seal generate a proposal seal.
	fn generate_seal(&self, block: &ExecutedBlock) -> Seal {
		if let Some(ref ap) = *self.account_provider.lock() {
			let header = block.header();
			let author = header.author();
			// Only proposer can generate seal if None was generated.
			if self.is_proposer(author).is_err() || self.proposal.read().is_some() {
				return Seal::None;
			}

			let height = header.number() as Height;
			let round = self.round.load(AtomicOrdering::SeqCst);
			let bh = Some(header.bare_hash());
			let vote_info = message_info_rlp(height, round, Step::Propose, bh.clone());
			if let Ok(signature) = ap.sign(*author, self.password.read().clone(), vote_info.sha3()).map(H520::from) {
				// Insert Propose vote.
				debug!(target: "poa", "Submitting proposal {} at height {} round {}.", header.bare_hash(), height, round);
				self.votes.vote(ConsensusMessage::new(signature, height, round, Step::Propose, bh), *author);
				// Remember proposal for later seal submission.
				*self.proposal.write() = bh;
				Seal::Proposal(vec![
					::rlp::encode(&round).to_vec(),
					::rlp::encode(&signature).to_vec(),
					::rlp::EMPTY_LIST_RLP.to_vec()
				])
			} else {
				warn!(target: "poa", "generate_seal: FAIL: accounts secret key unavailable");
				Seal::None
			}
		} else {
			warn!(target: "poa", "generate_seal: FAIL: accounts not provided");
			Seal::None
		}
	}

	fn handle_message(&self, rlp: &[u8]) -> Result<(), Error> {
		let rlp = UntrustedRlp::new(rlp);
		let message: ConsensusMessage = rlp.as_val()?;
		if !self.votes.is_old_or_known(&message) {
			let sender = public_to_address(&recover(&message.signature.into(), &rlp.at(1)?.as_raw().sha3())?);
			if !self.is_authority(&sender) {
				Err(EngineError::NotAuthorized(sender))?;
			}
			self.broadcast_message(rlp.as_raw().to_vec());
			trace!(target: "poa", "Handling a valid {:?} from {}.", message, sender);
			self.votes.vote(message.clone(), sender);
			self.handle_valid_message(&message);
		}
		Ok(())
	}

	/// Apply the block reward on finalisation of the block.
	fn on_close_block(&self, block: &mut ExecutedBlock) {
		let fields = block.fields_mut();
		// Bestow block reward
		fields.state.add_balance(fields.header.author(), &self.block_reward, CleanupMode::NoEmpty);
		// Commit state so that we can actually figure out the state root.
		if let Err(e) = fields.state.commit() {
			warn!("Encountered error on state commit: {}", e);
		}
	}

	fn verify_block_basic(&self, header: &Header, _block: Option<&[u8]>) -> Result<(), Error> {
		let seal_length = header.seal().len();
		if seal_length == self.seal_fields() {
			let signatures_len = header.seal()[2].len();
			if signatures_len >= 1 {
				Ok(())
			} else {
				Err(From::from(EngineError::BadSealFieldSize(OutOfBounds {
					min: Some(1),
					max: None,
					found: signatures_len
				})))
			}
		} else {
			Err(From::from(BlockError::InvalidSealArity(
				Mismatch { expected: self.seal_fields(), found: seal_length }
			)))
		}

	}

	fn verify_block_unordered(&self, header: &Header, _block: Option<&[u8]>) -> Result<(), Error> {
		let proposal = ConsensusMessage::new_proposal(header)?;	
		let proposer = proposal.verify()?;
		if !self.is_authority(&proposer) {
			Err(EngineError::NotAuthorized(proposer))?
		}

		let precommit_hash = proposal.precommit_hash();
		let ref signatures_field = header.seal()[2];
		let mut signature_count = 0;
		let mut origins = HashSet::new();
		for rlp in UntrustedRlp::new(signatures_field).iter() {
			let precommit: ConsensusMessage = ConsensusMessage::new_commit(&proposal, rlp.as_val()?);
			let address = match self.votes.get(&precommit) {
				Some(a) => a,
				None => public_to_address(&recover(&precommit.signature.into(), &precommit_hash)?),
			};
			if !self.validators.contains(&address) {
				Err(EngineError::NotAuthorized(address.to_owned()))?
			}

			if origins.insert(address) {
				signature_count += 1;
			} else {
				warn!(target: "poa", "verify_block_unordered: Duplicate signature from {} on the seal.", address);
				Err(BlockError::InvalidSeal)?;
			}
		}

		// Check if its a proposal if there is not enough precommits.
		if !self.is_above_threshold(signature_count) {
			let signatures_len = signatures_field.len();
			// Proposal has to have an empty signature list.
			if signatures_len != 1 {
				Err(EngineError::BadSealFieldSize(OutOfBounds {
					min: Some(1),
					max: Some(1),
					found: signatures_len
				}))?;
			}
			self.is_round_proposer(proposal.height, proposal.round, &proposer)?;
		}
		Ok(())
	}

	fn verify_block_family(&self, header: &Header, parent: &Header, _block: Option<&[u8]>) -> Result<(), Error> {
		if header.number() == 0 {
			Err(BlockError::RidiculousNumber(OutOfBounds { min: Some(1), max: None, found: header.number() }))?;
		}

		let gas_limit_divisor = self.gas_limit_bound_divisor;
		let min_gas = parent.gas_limit().clone() - parent.gas_limit().clone() / gas_limit_divisor;
		let max_gas = parent.gas_limit().clone() + parent.gas_limit().clone() / gas_limit_divisor;
		if header.gas_limit() <= &min_gas || header.gas_limit() >= &max_gas {
			Err(BlockError::InvalidGasLimit(OutOfBounds { min: Some(min_gas), max: Some(max_gas), found: header.gas_limit().clone() }))?;
		}

		Ok(())
	}

	fn verify_transaction_basic(&self, t: &SignedTransaction, _header: &Header) -> Result<(), Error> {
		t.check_low_s()?;
		Ok(())
	}

	fn verify_transaction(&self, t: &SignedTransaction, _header: &Header) -> Result<(), Error> {
		t.sender().map(|_|()) // Perform EC recovery and cache sender
	}

	fn set_signer(&self, address: Address, password: String) {
		*self.authority.write()	= address;
		*self.password.write() = Some(password);
		self.to_step(Step::Propose);
	}

	fn stop(&self) {
		self.step_service.stop()
	}

	fn is_new_best_block(&self, _best_total_difficulty: U256, best_header: HeaderView, _parent_details: &BlockDetails, new_header: &HeaderView) -> bool {
		let new_number = new_header.number();
		let best_number = best_header.number();
		trace!(target: "poa", "new_header: {}, best_header: {}", new_number, best_number);
		if new_number != best_number {
			new_number > best_number
		} else {
			let new_seal = new_header.seal();
			let best_seal = best_header.seal();
			let new_signatures = new_seal.get(2).expect("Tendermint seal should have three elements.").len();
			let best_signatures = best_seal.get(2).expect("Tendermint seal should have three elements.").len();
			if new_signatures > best_signatures {
				true
			} else {
				let new_round: Round = ::rlp::Rlp::new(&new_seal.get(0).expect("Tendermint seal should have three elements.")).as_val();
				let best_round: Round = ::rlp::Rlp::new(&best_seal.get(0).expect("Tendermint seal should have three elements.")).as_val();
				new_round > best_round
			}
		}
	}

	fn is_proposal(&self, header: &Header) -> bool {
		let signatures_len = header.seal()[2].len();
		// Signatures have to be an empty list rlp.
		let proposal = ConsensusMessage::new_proposal(header).expect("block went through full verification; this Engine verifies new_proposal creation; qed");
		if signatures_len != 1 {
			// New Commit received, skip to next height.
			trace!(target: "poa", "Received a commit for height {}, round {}.", proposal.height, proposal.round);
			self.to_next_height(proposal.height);
			return false;
		}
		let proposer = proposal.verify().expect("block went through full verification; this Engine tries verify; qed");
		debug!(target: "poa", "Received a new proposal for height {}, round {} from {}.", proposal.height, proposal.round, proposer);
		if self.is_round(&proposal) {
			*self.proposal.write() = proposal.block_hash.clone();
		}
		self.votes.vote(proposal, proposer);
		true
	}

	/// Equivalent to a timeout: to be used for tests.
	fn step(&self) {
		let next_step = match *self.step.read() {
			Step::Propose => {
				trace!(target: "poa", "Propose timeout.");
				Step::Prevote
			},
			Step::Prevote if self.has_enough_any_votes() => {
				trace!(target: "poa", "Prevote timeout.");
				Step::Precommit
			},
			Step::Prevote => {
				trace!(target: "poa", "Prevote timeout without enough votes.");
				self.broadcast_old_messages();
				Step::Prevote
			},
			Step::Precommit if self.has_enough_any_votes() => {
				trace!(target: "poa", "Precommit timeout.");
				self.increment_round(1);
				Step::Propose
			},
			Step::Precommit => {
				trace!(target: "poa", "Precommit timeout without enough votes.");
				self.broadcast_old_messages();
				Step::Precommit
			},
			Step::Commit => {
				trace!(target: "poa", "Commit timeout.");
				Step::Propose
			},
		};
		self.to_step(next_step);
	}

	fn register_client(&self, client: Weak<Client>) {
		*self.client.write() = Some(client.clone());
		self.validators.register_call_contract(client);
	}

	fn register_account_provider(&self, account_provider: Arc<AccountProvider>) {
		*self.account_provider.lock() = Some(account_provider);
	}
}

#[cfg(test)]
mod tests {
	use util::*;
	use block::*;
	use error::{Error, BlockError};
	use header::Header;
	use env_info::EnvInfo;
	use client::chain_notify::ChainNotify;
	use miner::MinerService;
	use tests::helpers::*;
	use account_provider::AccountProvider;
	use spec::Spec;
	use engines::{Engine, EngineError, Seal};
	use super::*;
	use super::message::*;

	/// Accounts inserted with "0" and "1" are validators. First proposer is "0".
	fn setup() -> (Spec, Arc<AccountProvider>) {
		let tap = Arc::new(AccountProvider::transient_provider());
		let spec = Spec::new_test_tendermint();
		spec.engine.register_account_provider(tap.clone());
		(spec, tap)
	}

	fn propose_default(spec: &Spec, proposer: Address) -> (ClosedBlock, Vec<Bytes>) {
		let mut db_result = get_temp_state_db();
		let db = spec.ensure_db_good(db_result.take(), &Default::default()).unwrap();
		let genesis_header = spec.genesis_header();
		let last_hashes = Arc::new(vec![genesis_header.hash()]);
		let b = OpenBlock::new(spec.engine.as_ref(), Default::default(), false, db.boxed_clone(), &genesis_header, last_hashes, proposer, (3141562.into(), 31415620.into()), vec![]).unwrap();
		let b = b.close();
		if let Seal::Proposal(seal) = spec.engine.generate_seal(b.block()) {
			(b, seal)
		} else {
			panic!()
		}
	}

	fn vote<F>(engine: &Engine, signer: F, height: usize, round: usize, step: Step, block_hash: Option<H256>) -> Bytes where F: FnOnce(H256) -> Result<H520, ::account_provider::Error> {
		let mi = message_info_rlp(height, round, step, block_hash);
		let m = message_full_rlp(&signer(mi.sha3()).unwrap().into(), &mi);
		engine.handle_message(&m).unwrap();
		m
	}

	fn proposal_seal(tap: &Arc<AccountProvider>, header: &Header, round: Round) -> Vec<Bytes> {
		let author = header.author();
		let vote_info = message_info_rlp(header.number() as Height, round, Step::Propose, Some(header.bare_hash()));
		let signature = tap.sign(*author, None, vote_info.sha3()).unwrap();
		vec![
			::rlp::encode(&round).to_vec(),
			::rlp::encode(&H520::from(signature)).to_vec(),
			::rlp::EMPTY_LIST_RLP.to_vec()
		]
	}

	fn insert_and_unlock(tap: &Arc<AccountProvider>, acc: &str) -> Address {
		let addr = tap.insert_account(acc.sha3(), acc).unwrap();
		tap.unlock_account_permanently(addr, acc.into()).unwrap();
		addr
	}

	fn insert_and_register(tap: &Arc<AccountProvider>, engine: &Engine, acc: &str) -> Address {
		let addr = insert_and_unlock(tap, acc);
		engine.set_signer(addr.clone(), acc.into());
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
		let schedule = engine.schedule(&EnvInfo {
			number: 10000000,
			author: 0.into(),
			timestamp: 0,
			difficulty: 0.into(),
			last_hashes: Arc::new(vec![]),
			gas_used: 0.into(),
			gas_limit: 0.into(),
		});

		assert!(schedule.stack_limit > 0);
	}

	#[test]
	fn verification_fails_on_short_seal() {
		let engine = Spec::new_test_tendermint().engine;
		let header = Header::default();

		let verify_result = engine.verify_block_basic(&header, None);

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

		let mut header = Header::default();
		let validator = insert_and_unlock(&tap, "0");
		header.set_author(validator);
		let seal = proposal_seal(&tap, &header, 0);
		header.set_seal(seal);
		// Good proposer.
		assert!(engine.verify_block_unordered(&header.clone(), None).is_ok());

		let validator = insert_and_unlock(&tap, "1");
		header.set_author(validator);
		let seal = proposal_seal(&tap, &header, 0);
		header.set_seal(seal);
		// Bad proposer.
		match engine.verify_block_unordered(&header, None) {
			Err(Error::Engine(EngineError::NotProposer(_))) => {},
			_ => panic!(),
		}

		let random = insert_and_unlock(&tap, "101");
		header.set_author(random);
		let seal = proposal_seal(&tap, &header, 0);
		header.set_seal(seal);
		// Not authority.
		match engine.verify_block_unordered(&header, None) {
			Err(Error::Engine(EngineError::NotAuthorized(_))) => {},
			_ => panic!(),
		};
		engine.stop();
	}

	#[test]
	fn seal_signatures_checking() {
		let (spec, tap) = setup();
		let engine = spec.engine;

		let mut header = Header::default();
		let proposer = insert_and_unlock(&tap, "1");
		header.set_author(proposer);
		let mut seal = proposal_seal(&tap, &header, 0);

		let vote_info = message_info_rlp(0, 0, Step::Precommit, Some(header.bare_hash()));
		let signature1 = tap.sign(proposer, None, vote_info.sha3()).unwrap();

		seal[2] = ::rlp::encode(&vec![H520::from(signature1.clone())]).to_vec();
		header.set_seal(seal.clone());

		// One good signature is not enough.
		match engine.verify_block_unordered(&header, None) {
			Err(Error::Engine(EngineError::BadSealFieldSize(_))) => {},
			_ => panic!(),
		}

		let voter = insert_and_unlock(&tap, "0");
		let signature0 = tap.sign(voter, None, vote_info.sha3()).unwrap();

		seal[2] = ::rlp::encode(&vec![H520::from(signature1.clone()), H520::from(signature0.clone())]).to_vec();
		header.set_seal(seal.clone());

		assert!(engine.verify_block_unordered(&header, None).is_ok());

		let bad_voter = insert_and_unlock(&tap, "101");
		let bad_signature = tap.sign(bad_voter, None, vote_info.sha3()).unwrap();

		seal[2] = ::rlp::encode(&vec![H520::from(signature1), H520::from(bad_signature)]).to_vec();
		header.set_seal(seal);

		// One good and one bad signature.
		match engine.verify_block_unordered(&header, None) {
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
		spec.engine.stop();
	}

	#[test]
	fn can_recognize_proposal() {
		let (spec, tap) = setup();

		let proposer = insert_and_register(&tap, spec.engine.as_ref(), "1");

		let (b, seal) = propose_default(&spec, proposer);
		let sealed = b.lock().seal(spec.engine.as_ref(), seal).unwrap();
		assert!(spec.engine.is_proposal(sealed.header()));
		spec.engine.stop();
	}

	#[test]
	fn relays_messages() {
		let (spec, tap) = setup();
		let engine = spec.engine.clone();
		
		let v0 = insert_and_register(&tap, engine.as_ref(), "0");
		let v1 = insert_and_register(&tap, engine.as_ref(), "1");

		let h = 0;
		let r = 0;

		// Propose
		let (b, _) = propose_default(&spec, v1.clone());
		let proposal = Some(b.header().bare_hash());

		let client = generate_dummy_client(0);
		let notify = Arc::new(TestNotify::default());
		client.add_notify(notify.clone());
		engine.register_client(Arc::downgrade(&client));

		let prevote_current = vote(engine.as_ref(), |mh| tap.sign(v0, None, mh).map(H520::from), h, r, Step::Prevote, proposal);

		let precommit_current = vote(engine.as_ref(), |mh| tap.sign(v0, None, mh).map(H520::from), h, r, Step::Precommit, proposal);

		let prevote_future = vote(engine.as_ref(), |mh| tap.sign(v0, None, mh).map(H520::from), h + 1, r, Step::Prevote, proposal);

		// Relays all valid present and future messages.
		assert!(notify.messages.read().contains(&prevote_current));
		assert!(notify.messages.read().contains(&precommit_current));
		assert!(notify.messages.read().contains(&prevote_future));
		engine.stop();
	}

	#[test]
	fn seal_submission() {
		use ethkey::{Generator, Random};
		use types::transaction::{Transaction, Action};
		use client::BlockChainClient;

		let client = generate_dummy_client_with_spec_and_data(Spec::new_test_tendermint, 0, 0, &[]);
		let engine = client.engine();
		let tap = Arc::new(AccountProvider::transient_provider());

		// Accounts for signing votes.
		let v0 = insert_and_unlock(&tap, "0");
		let v1 = insert_and_unlock(&tap, "1");

		let notify = Arc::new(TestNotify::default());
		engine.register_account_provider(tap.clone());
		client.add_notify(notify.clone());
		engine.register_client(Arc::downgrade(&client));

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

		client.miner().set_engine_signer(v1.clone(), "1".into()).unwrap();

		// Propose
		let proposal = Some(client.miner().pending_block().unwrap().header.bare_hash());
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

		engine.stop();
	}
}
