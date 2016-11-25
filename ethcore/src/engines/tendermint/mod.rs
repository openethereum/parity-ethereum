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

//! Tendermint BFT consensus engine with round robin proof-of-authority.

mod message;
mod transition;
mod params;
mod vote_collector;

use std::sync::atomic::{AtomicUsize, Ordering as AtomicOrdering};
use util::*;
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
use engines::{Engine, EngineError};
use blockchain::extras::BlockDetails;
use views::HeaderView;
use evm::Schedule;
use io::{IoService, IoChannel};
use service::ClientIoMessage;
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

pub type Height = usize;
pub type Round = usize;
pub type BlockHash = H256;

type Signatures = Vec<Bytes>;

/// Engine using `Tendermint` consensus algorithm, suitable for EVM chain.
pub struct Tendermint {
	params: CommonParams,
	our_params: TendermintParams,
	builtins: BTreeMap<Address, Builtin>,
	step_service: IoService<Step>,
	/// Address to be used as authority.
	authority: RwLock<Address>,
	/// Blockchain height.
	height: AtomicUsize,
	/// Consensus round.
	round: AtomicUsize,
	/// Consensus step.
	step: RwLock<Step>,
	/// Used to swith proposer.
	proposer_nonce: AtomicUsize,
	/// Vote accumulator.
	votes: VoteCollector,
	/// Channel for updating the sealing.
	message_channel: Mutex<Option<IoChannel<ClientIoMessage>>>,
	/// Used to sign messages and proposals.
	account_provider: Mutex<Option<Arc<AccountProvider>>>,
	/// Message for the last PoLC.
	lock_change: RwLock<Option<ConsensusMessage>>,
	/// Last lock round.
	last_lock: AtomicUsize,
	/// Bare hash of the proposed block, used for seal submission.
	proposal: RwLock<Option<H256>>
}

impl Tendermint {
	/// Create a new instance of Tendermint engine
	pub fn new(params: CommonParams, our_params: TendermintParams, builtins: BTreeMap<Address, Builtin>) -> Result<Arc<Self>, Error> {
		let engine = Arc::new(
			Tendermint {
				params: params,
				our_params: our_params,
				builtins: builtins,
				step_service: try!(IoService::<Step>::start()),
				authority: RwLock::new(Address::default()),
				height: AtomicUsize::new(0),
				round: AtomicUsize::new(0),
				step: RwLock::new(Step::Propose),
				proposer_nonce: AtomicUsize::new(0),
				votes: VoteCollector::new(),
				message_channel: Mutex::new(None),
				account_provider: Mutex::new(None),
				lock_change: RwLock::new(None),
				last_lock: AtomicUsize::new(0),
				proposal: RwLock::new(None)
			});
		let handler = TransitionHandler { engine: Arc::downgrade(&engine) };
		try!(engine.step_service.register_handler(Arc::new(handler)));
		Ok(engine)
	}

	fn update_sealing(&self) {
		if let Some(ref channel) = *self.message_channel.lock() {
			match channel.send(ClientIoMessage::UpdateSealing) {
				Ok(_) => trace!(target: "poa", "timeout: UpdateSealing message sent."),
				Err(err) => warn!(target: "poa", "timeout: Could not send a sealing message {}.", err),
			}
		}
	}

	fn submit_seal(&self, block_hash: H256, seal: Vec<Bytes>) {
		if let Some(ref channel) = *self.message_channel.lock() {
			match channel.send(ClientIoMessage::SubmitSeal(block_hash, seal)) {
				Ok(_) => trace!(target: "poa", "timeout: SubmitSeal message sent."),
				Err(err) => warn!(target: "poa", "timeout: Could not send a sealing message {}.", err),
			}
		}
	}

	fn broadcast_message(&self, block_hash: Option<BlockHash>) {
		if let Some(message) = self.generate_message(block_hash) {
			if let Some(ref channel) = *self.message_channel.lock() {
				match channel.send(ClientIoMessage::BroadcastMessage(message)) {
					Ok(_) => trace!(target: "poa", "timeout: BroadcastMessage message sent."),
					Err(err) => warn!(target: "poa", "timeout: Could not send a sealing message {}.", err),
				}
			}
		}
	}

	fn generate_message(&self, block_hash: Option<BlockHash>) -> Option<Bytes> {
		if let Some(ref ap) = *self.account_provider.lock() {
			message_full_rlp(
				|mh| ap.sign(*self.authority.read(), None, mh).ok().map(H520::from),
				self.height.load(AtomicOrdering::SeqCst),
				self.round.load(AtomicOrdering::SeqCst),
				*self.step.read(),
				block_hash
			)
		} else {
			warn!(target: "poa", "generate_message: No AccountProvider available.");
			None
		}
	}

	fn to_step(&self, step: Step) {
		*self.step.write() = step;
		match step {
			Step::Propose => {
				trace!(target: "poa", "to_step: Transitioning to Propose.");
				*self.proposal.write() = None;
				self.update_sealing()
			},
			Step::Prevote => {
				trace!(target: "poa", "to_step: Transitioning to Prevote.");
				let block_hash = match *self.lock_change.read() {
					Some(ref m) if self.should_unlock(m.round) => self.proposal.read().clone(),
					Some(ref m) => m.block_hash,
					None => None,
				};
				self.broadcast_message(block_hash)
			},
			Step::Precommit => {
				trace!(target: "poa", "to_step: Transitioning to Precommit.");
				let block_hash = match *self.lock_change.read() {
					Some(ref m) if self.is_round(m) => {
						self.last_lock.store(m.round, AtomicOrdering::SeqCst);
						m.block_hash
					},
					_ => None,
				};
				self.broadcast_message(block_hash);
			},
			Step::Commit => {
				trace!(target: "poa", "to_step: Transitioning to Commit.");
				// Commit the block using a complete signature set.
				let round = self.round.load(AtomicOrdering::SeqCst);
				if let Some(seal) = self.votes.seal_signatures(self.height.load(AtomicOrdering::SeqCst), round, *self.proposal.read()) {
					let seal = vec![
						::rlp::encode(&round).to_vec(),
						::rlp::encode(&seal.proposal).to_vec(),
						::rlp::encode(&seal.votes).to_vec()
					];
					if let Some(block_hash) = *self.proposal.read() {
						self.submit_seal(block_hash, seal);
					}
				}
				*self.lock_change.write() = None;
			},
		}
	}

	fn is_authority(&self, address: &Address) -> bool {
		self.our_params.authorities.contains(address)
	}

	fn is_above_threshold(&self, n: usize) -> bool {
		n > self.our_params.authority_n * 2/3
	}

	/// Round proposer switching.
	fn is_proposer(&self, address: &Address) -> Result<(), BlockError> {
		let ref p = self.our_params;
		let proposer_nonce = self.proposer_nonce.load(AtomicOrdering::SeqCst);
		let proposer = p.authorities.get(proposer_nonce % p.authority_n).expect("There are authority_n authorities; taking number modulo authority_n gives number in authority_n range; qed");
		if proposer == address {
			Ok(())
		} else {
			Err(BlockError::NotProposer(Mismatch { expected: proposer.clone(), found: address.clone() }))
		}
	}

	fn is_height(&self, message: &ConsensusMessage) -> bool {
		message.is_height(self.height.load(AtomicOrdering::SeqCst)) 
	}

	fn is_round(&self, message: &ConsensusMessage) -> bool {
		message.is_round(self.height.load(AtomicOrdering::SeqCst), self.round.load(AtomicOrdering::SeqCst)) 
	}

	fn increment_round(&self, n: Round) {
		self.proposer_nonce.fetch_add(n, AtomicOrdering::SeqCst);
		self.round.fetch_add(n, AtomicOrdering::SeqCst);
	}

	fn reset_round(&self) {
		self.last_lock.store(0, AtomicOrdering::SeqCst);
		self.proposer_nonce.fetch_add(1, AtomicOrdering::SeqCst);
		self.height.fetch_add(1, AtomicOrdering::SeqCst);
		self.round.store(0, AtomicOrdering::SeqCst);
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
		let aligned_votes = self.votes.aligned_votes(&message).len();
		self.is_above_threshold(aligned_votes)
	}
}

impl Engine for Tendermint {
	fn name(&self) -> &str { "Tendermint" }
	fn version(&self) -> SemanticVersion { SemanticVersion::new(1, 0, 0) }
	/// (consensus round, proposal signature, authority signatures)
	fn seal_fields(&self) -> usize { 3 }

	fn params(&self) -> &CommonParams { &self.params }
	fn builtins(&self) -> &BTreeMap<Address, Builtin> { &self.builtins }

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
		Schedule::new_homestead()
	}

	fn populate_from_parent(&self, header: &mut Header, parent: &Header, gas_floor_target: U256, _gas_ceil_target: U256) {
		header.set_difficulty(parent.difficulty().clone());
		header.set_gas_limit({
			let gas_limit = parent.gas_limit().clone();
			let bound_divisor = self.our_params.gas_limit_bound_divisor;
			if gas_limit < gas_floor_target {
				min(gas_floor_target, gas_limit + gas_limit / bound_divisor - 1.into())
			} else {
				max(gas_floor_target, gas_limit - gas_limit / bound_divisor + 1.into())
			}
		});
	}

	/// Get the address to be used as authority.
	fn on_new_block(&self, block: &mut ExecutedBlock) {
		*self.authority.write()	= *block.header().author()
	}

	/// Round proposer switching.
	fn is_sealer(&self, address: &Address) -> Option<bool> {
		Some(self.is_proposer(address).is_ok())
	}

	/// Attempt to seal the block internally using all available signatures.
	fn generate_seal(&self, block: &ExecutedBlock) -> Option<Vec<Bytes>> {
		if let Some(ref ap) = *self.account_provider.lock() {
			let header = block.header();
			let author = header.author();
			let vote_info = message_info_rlp(header.number() as Height, self.round.load(AtomicOrdering::SeqCst), Step::Propose, Some(header.bare_hash()));
			if let Ok(signature) = ap.sign(*author, None, vote_info.sha3()) {
				*self.proposal.write() = Some(header.bare_hash());
				Some(vec![
					::rlp::encode(&self.round.load(AtomicOrdering::SeqCst)).to_vec(),
					::rlp::encode(&H520::from(signature)).to_vec(),
					Vec::new()
				])
			} else {
				warn!(target: "poa", "generate_seal: FAIL: accounts secret key unavailable");
				None
			}
		} else {
			warn!(target: "poa", "generate_seal: FAIL: accounts not provided");
			None
		}
	}

	fn handle_message(&self, rlp: UntrustedRlp) -> Result<(), Error> {
		let message: ConsensusMessage = try!(rlp.as_val());
		let sender = public_to_address(&try!(recover(&message.signature.into(), &try!(rlp.at(1)).as_raw().sha3())));
		// TODO: Do not admit old messages.
		if !self.is_authority(&sender) {
			try!(Err(BlockError::NotAuthorized(sender)));
		}

		// Check if the message is known.
		if self.votes.vote(message.clone(), sender).is_none() {
			trace!(target: "poa", "handle_message: Processing new authorized message: {:?}", &message);
			let is_newer_than_lock = match *self.lock_change.read() {
				Some(ref lock) => &message > lock,
				None => true,
			};
			if is_newer_than_lock
				&& message.step == Step::Prevote
				&& self.has_enough_aligned_votes(&message) {
				trace!(target: "poa", "handle_message: Lock change.");
				*self.lock_change.write()	= Some(message.clone());
			}
			// Check if it can affect the step transition.
			if self.is_height(&message) {
				let next_step = match *self.step.read() {
					Step::Precommit if self.has_enough_aligned_votes(&message) => {
						if message.block_hash.is_none() {
							self.increment_round(1);
							Some(Step::Propose)
						} else {
							Some(Step::Commit)
						}
					},
					Step::Precommit if self.has_enough_future_step_votes(&message) => {
						self.increment_round(message.round - self.round.load(AtomicOrdering::SeqCst));
						Some(Step::Precommit)
					},
					Step::Prevote if self.has_enough_aligned_votes(&message) => Some(Step::Precommit),
					Step::Prevote if self.has_enough_future_step_votes(&message) => {
						self.increment_round(message.round - self.round.load(AtomicOrdering::SeqCst));
						Some(Step::Prevote)
					},
					_ => None,
				};

				if let Some(step) = next_step {
					if let Err(io_err) = self.step_service.send_message(step) {
						warn!(target: "poa", "Could not proceed to next step {}.", io_err)
					}
				}
			}
		}
		Ok(())
	}

	fn verify_block_basic(&self, header: &Header, _block: Option<&[u8]>) -> Result<(), Error> {
		let seal_length = header.seal().len();
		if seal_length == self.seal_fields() {
			Ok(())
		} else {
			Err(From::from(BlockError::InvalidSealArity(
				Mismatch { expected: self.seal_fields(), found: seal_length }
			)))
		}
	}

	/// Also transitions to Prevote if verifying Proposal.
	fn verify_block_unordered(&self, header: &Header, _block: Option<&[u8]>) -> Result<(), Error> {
		let proposal = try!(ConsensusMessage::new_proposal(header));	
		let proposer = try!(proposal.verify());
		try!(self.is_proposer(&proposer));
		self.votes.vote(proposal, proposer);
		let block_info_hash = try!(message_info_rlp_from_header(header)).sha3();

		// TODO: use addresses recovered during precommit vote
		let mut signature_count = 0;
		for rlp in UntrustedRlp::new(&header.seal()[2]).iter() {
			let signature: H520 = try!(rlp.as_val());
			let address = public_to_address(&try!(recover(&signature.into(), &block_info_hash)));
			if !self.our_params.authorities.contains(&address) {
				try!(Err(BlockError::NotAuthorized(address)))
			}

			signature_count += 1;
		}
		if signature_count > self.our_params.authority_n {
			try!(Err(BlockError::InvalidSealArity(Mismatch { expected: self.our_params.authority_n, found: signature_count })))
		}
		Ok(())
	}

	fn verify_block_family(&self, header: &Header, parent: &Header, _block: Option<&[u8]>) -> Result<(), Error> {
		// we should not calculate difficulty for genesis blocks
		if header.number() == 0 {
			return Err(From::from(BlockError::RidiculousNumber(OutOfBounds { min: Some(1), max: None, found: header.number() })));
		}

		// Check difficulty is correct given the two timestamps.
		if header.difficulty() != parent.difficulty() {
			return Err(From::from(BlockError::InvalidDifficulty(Mismatch { expected: *parent.difficulty(), found: *header.difficulty() })))
		}
		let gas_limit_divisor = self.our_params.gas_limit_bound_divisor;
		let min_gas = parent.gas_limit().clone() - parent.gas_limit().clone() / gas_limit_divisor;
		let max_gas = parent.gas_limit().clone() + parent.gas_limit().clone() / gas_limit_divisor;
		if header.gas_limit() <= &min_gas || header.gas_limit() >= &max_gas {
			return Err(From::from(BlockError::InvalidGasLimit(OutOfBounds { min: Some(min_gas), max: Some(max_gas), found: header.gas_limit().clone() })));
		}
		Ok(())
	}

	fn verify_transaction_basic(&self, t: &SignedTransaction, _header: &Header) -> Result<(), Error> {
		try!(t.check_low_s());
		Ok(())
	}

	fn verify_transaction(&self, t: &SignedTransaction, _header: &Header) -> Result<(), Error> {
		t.sender().map(|_|()) // Perform EC recovery and cache sender
	}

	fn is_new_best_block(&self, _best_total_difficulty: U256, best_header: HeaderView, _parent_details: &BlockDetails, new_header: &HeaderView) -> bool {
		let new_signatures = new_header.seal().get(2).expect("Tendermint seal should have three elements.").len();
		let best_signatures = best_header.seal().get(2).expect("Tendermint seal should have three elements.").len();
		new_signatures > best_signatures
	}

	fn register_message_channel(&self, message_channel: IoChannel<ClientIoMessage>) {
		*self.message_channel.lock() = Some(message_channel);
	}

	fn register_account_provider(&self, account_provider: Arc<AccountProvider>) {
		*self.account_provider.lock() = Some(account_provider);
	}
}

#[cfg(test)]
mod tests {
	use util::*;
	use rlp::{UntrustedRlp, View};
	use io::{IoContext, IoHandler};
	use block::*;
	use state_db::StateDB;
	use error::{Error, BlockError};
	use header::Header;
	use env_info::EnvInfo;
	use tests::helpers::*;
	use account_provider::AccountProvider;
	use io::IoService;
	use service::ClientIoMessage;
	use spec::Spec;
	use engines::{Engine, EngineError};
	use super::*;
	use super::params::TendermintParams;
	use super::message::*;

	/// Accounts inserted with "1" and "2" are authorities. First proposer is "0".
	fn setup() -> (Spec, Arc<AccountProvider>) {
		let tap = Arc::new(AccountProvider::transient_provider());
		let spec = Spec::new_test_tendermint();
		spec.engine.register_account_provider(tap.clone());
		(spec, tap)
	}

	fn propose_default(spec: &Spec, proposer: Address) -> (LockedBlock, Vec<Bytes>) {
		let mut db_result = get_temp_state_db();
		let mut db = db_result.take();
		spec.ensure_db_good(&mut db).unwrap();
		let genesis_header = spec.genesis_header();
		let last_hashes = Arc::new(vec![genesis_header.hash()]);
		let b = OpenBlock::new(spec.engine.as_ref(), Default::default(), false, db.boxed_clone(), &genesis_header, last_hashes, proposer, (3141562.into(), 31415620.into()), vec![]).unwrap();
		let b = b.close_and_lock();
		let seal = spec.engine.generate_seal(b.block()).unwrap();
		(b, seal)
	}

	fn vote<F>(engine: &Arc<Engine>, signer: F, height: usize, round: usize, step: Step, block_hash: Option<H256>) where F: FnOnce(H256) -> Option<H520> {
		let m = message_full_rlp(signer, height, round, step, block_hash).unwrap();
		engine.handle_message(UntrustedRlp::new(&m)).unwrap();
	}

	fn proposal_seal(tap: &Arc<AccountProvider>, header: &Header, round: Round) -> Vec<Bytes> {
		let author = header.author();
		println!("author: {:?}", author);
		let vote_info = message_info_rlp(header.number() as Height, round, Step::Propose, Some(header.bare_hash()));
		let signature = tap.sign(*author, None, vote_info.sha3()).unwrap();
		vec![
			::rlp::encode(&round).to_vec(),
			::rlp::encode(&H520::from(signature)).to_vec(),
			Vec::new()
		]
	}

	fn insert_and_unlock(tap: &Arc<AccountProvider>, acc: &str) -> Address {
		let addr = tap.insert_account(acc.sha3(), acc).unwrap();
		tap.unlock_account_permanently(addr, acc.into()).unwrap();
		addr
	}

	struct TestIo(ClientIoMessage);

	impl IoHandler<ClientIoMessage> for TestIo {
		fn message(&self, _io: &IoContext<ClientIoMessage>, net_message: &ClientIoMessage) {
			let TestIo(ref expected) = *self;
			if net_message == expected {
				panic!()
			}
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
		println!("validator: {:?}", &validator);
		header.set_author(validator);
		let seal = proposal_seal(&tap, &header, 0);
		header.set_seal(seal);
		// Good proposer.
		println!("{:?}", engine.verify_block_unordered(&header, None));
		assert!(engine.verify_block_unordered(&header, None).is_ok());

		let mut header = Header::default();
		let random = insert_and_unlock(&tap, "101");
		header.set_author(random);
		let seal = proposal_seal(&tap, &header, 0);
		header.set_seal(seal);
		// Bad proposer.
		match engine.verify_block_unordered(&header, None) {
			Err(Error::Block(BlockError::NotProposer(_))) => {},
			_ => panic!(),
		}
	}

	#[test]
	fn seal_signatures_checking() {
		let (spec, tap) = setup();
		let engine = spec.engine;

		let mut header = Header::default();
		let proposer = insert_and_unlock(&tap, "0");
		header.set_author(proposer);
		let mut seal = proposal_seal(&tap, &header, 0);

		let voter = insert_and_unlock(&tap, "1");
		println!("voter: {:?}", &voter);
		let vote_info = message_info_rlp(0, 0, Step::Precommit, Some(header.bare_hash()));
		let signature = tap.sign(voter, None, vote_info.sha3()).unwrap();

		seal[2] = ::rlp::encode(&vec![H520::from(signature.clone())]).to_vec();

		header.set_seal(seal.clone());

		println!("{:?}", engine.verify_block_unordered(&header, None));
		// One good signature.
		assert!(engine.verify_block_unordered(&header, None).is_ok());

		let bad_voter = insert_and_unlock(&tap, "101");
		let bad_signature = tap.sign(bad_voter, None, vote_info.sha3()).unwrap();
		seal[2] = ::rlp::encode(&vec![H520::from(signature), H520::from(bad_signature)]).to_vec();

		header.set_seal(seal);

		// One good and one bad signature.
		match engine.verify_block_unordered(&header, None) {
			Err(Error::Block(BlockError::NotAuthorized(_))) => {},
			_ => panic!(),
		}
	}

	#[test]
	fn can_generate_seal() {
		let (spec, tap) = setup();
		
		let proposer = insert_and_unlock(&tap, "0");

		let (b, seal) = propose_default(&spec, proposer);
		assert!(b.try_seal(spec.engine.as_ref(), seal).is_ok());
	}

	#[test]
	#[should_panic]
	fn prevote_step() {
		::env_logger::init().unwrap();
		let (spec, tap) = setup();
		let engine = spec.engine.clone();
		let mut db_result = get_temp_state_db();
		let mut db = db_result.take();
		spec.ensure_db_good(&mut db).unwrap();
		
		let v0 = insert_and_unlock(&tap, "0");
		let v1 = insert_and_unlock(&tap, "1");

		// Propose
		let (b, seal) = propose_default(&spec, v0.clone());
		let proposal = Some(b.header().bare_hash());

		// Register IoHandler that panics on correct message.
		let io_service = IoService::<ClientIoMessage>::start().unwrap();
		io_service.register_handler(Arc::new(TestIo(ClientIoMessage::SubmitSeal(Default::default(), seal)))).unwrap();
		engine.register_message_channel(io_service.channel());

		let h = 1;
		let r = 0;

		// Prevote.
		vote(&engine, |mh| tap.sign(v1, None, mh).ok().map(H520::from), h, r, Step::Prevote, proposal);

		vote(&engine, |mh| tap.sign(v0, None, mh).ok().map(H520::from), h, r, Step::Prevote, proposal);
		vote(&engine, |mh| tap.sign(v1, None, mh).ok().map(H520::from), h, r, Step::Prevote, proposal);
	}

	#[test]
	fn precommit_step() {
		let (spec, tap) = setup();
		let engine = spec.engine.clone();

		let not_authority_addr = insert_and_unlock(&tap, "101");
		let proposer_addr = insert_and_unlock(&tap, "0");
		propose_default(&spec, proposer_addr);

		let not_proposer_addr = insert_and_unlock(&tap, "1");

	}

	#[test]
	fn timeout_switching() {
		let tender = {
			let engine = Spec::new_test_tendermint().engine;
			Tendermint::new(engine.params().clone(), TendermintParams::default(), BTreeMap::new())
		};

		println!("Waiting for timeout");
	}
}
