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
use basic_types::Seal;
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
use self::message::{ConsensusMessage, message_info_rlp, message_full_rlp};
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
		} else {
			warn!(target: "poa", "broadcast_message: Message could not be generated.");
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
			None
		}
	}

	fn to_step(&self, step: Step) {
		*self.step.write() = step;
		match step {
			Step::Propose => {
				*self.proposal.write() = None;
				self.update_sealing()
			},
			Step::Prevote => {
				let block_hash = match *self.lock_change.read() {
					Some(ref m) if self.should_unlock(m.round) => self.proposal.read().clone(),
					Some(ref m) => m.block_hash,
					None => None,
				};
				self.broadcast_message(block_hash)
			},
			Step::Precommit => {
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

	fn nonce_proposer(&self, proposer_nonce: usize) -> &Address {
		let ref p = self.our_params;
		p.authorities.get(proposer_nonce % p.authority_n).expect("There are authority_n authorities; taking number modulo authority_n gives number in authority_n range; qed")
	}

	fn is_nonce_proposer(&self, proposer_nonce: usize, address: &Address) -> bool {
		self.nonce_proposer(proposer_nonce) == address
	}

	fn is_authority(&self, address: &Address) -> bool {
		self.our_params.authorities.contains(address)
	}

	fn threshold(&self) -> usize {
		self.our_params.authority_n * 2/3
	}

	/// Round proposer switching.
	fn is_proposer(&self, address: &Address) -> bool {
		self.is_nonce_proposer(self.proposer_nonce.load(AtomicOrdering::SeqCst), address)
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
		self.votes.count_step_votes(self.height.load(AtomicOrdering::SeqCst), self.round.load(AtomicOrdering::SeqCst), *self.step.read()) > self.threshold()	
	}

	fn has_enough_future_step_votes(&self, message: &ConsensusMessage) -> bool {
		message.round > self.round.load(AtomicOrdering::SeqCst)
			&& self.votes.count_step_votes(message.height, message.round, message.step) > self.threshold()	
	}

	fn has_enough_aligned_votes(&self, message: &ConsensusMessage) -> bool {
		self.votes.aligned_votes(&message).len() > self.threshold()
	}
}

/// Block hash including the consensus round, gets signed and included in the seal.
fn block_hash(header: &Header) -> H256 {
	header.rlp(Seal::WithSome(1)).sha3()
}

fn proposer_signature(header: &Header) -> Result<H520, ::rlp::DecoderError> {
	UntrustedRlp::new(header.seal()[1].as_slice()).as_val()
}

fn consensus_round(header: &Header) -> Result<Round, ::rlp::DecoderError> {
	UntrustedRlp::new(header.seal()[0].as_slice()).as_val()
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
		map![
			"signature".into() => proposer_signature(header).as_ref().map(ToString::to_string).unwrap_or("".into()),
			"height".into() => header.number().to_string(),
			"round".into() => consensus_round(header).as_ref().map(ToString::to_string).unwrap_or("".into()),
			"block_hash".into() => block_hash(header).to_string()
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

	/// Set the correct round in the seal.
	fn on_close_block(&self, block: &mut ExecutedBlock) {
		let round = self.round.load(AtomicOrdering::SeqCst);
		block.fields_mut().header.set_seal(vec![::rlp::encode(&round).to_vec(), Vec::new(), Vec::new()]);
	}

	/// Round proposer switching.
	fn is_sealer(&self, address: &Address) -> Option<bool> {
		Some(self.is_proposer(address))
	}

	/// Attempt to seal the block internally using all available signatures.
	fn generate_seal(&self, block: &ExecutedBlock) -> Option<Vec<Bytes>> {
		if let Some(ref ap) = *self.account_provider.lock() {
			let header = block.header();
			let author = header.author();
			let vote_info = message_info_rlp(header.number() as Height, self.round.load(AtomicOrdering::SeqCst), Step::Propose, Some(block_hash(header)));
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
			try!(Err(BlockError::InvalidSeal));
		}

		// Check if the message is known.
		if self.votes.vote(message.clone(), sender).is_none() {
			let is_newer_than_lock = match *self.lock_change.read() {
				Some(ref lock) => &message > lock,
				None => true,
			};
			if is_newer_than_lock
				&& message.step == Step::Prevote
				&& self.has_enough_aligned_votes(&message) {
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
		let signature = try!(proposer_signature(header));
		let proposal = ConsensusMessage {
			signature: signature,
			height: header.number() as Height,
			round: try!(consensus_round(header)),
			step: Step::Propose,
			block_hash: Some(block_hash(header))
		};
		let proposer = public_to_address(&try!(recover(&signature.into(), &::rlp::encode(&proposal))));
		if !self.is_proposer(&proposer) {
			try!(Err(BlockError::InvalidSeal))
		}
		self.votes.vote(proposal, proposer);
		let votes_rlp = UntrustedRlp::new(&header.seal()[2]);
		for rlp in votes_rlp.iter() {
			let sig: H520 = try!(rlp.as_val());
			let address = public_to_address(&try!(recover(&sig.into(), &block_hash(header))));
			if !self.our_params.authorities.contains(&address) {
				try!(Err(BlockError::InvalidSeal))
			}
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
	use std::thread::sleep;
	use std::time::{Duration};
	use util::*;
	use rlp::{UntrustedRlp, RlpStream, Stream, View};
	use block::*;
	use error::{Error, BlockError};
	use header::Header;
	use env_info::EnvInfo;
	use tests::helpers::*;
	use account_provider::AccountProvider;
	use spec::Spec;
	use engines::{Engine, EngineError};
	use super::Tendermint;
	use super::params::TendermintParams;

	fn propose_default(engine: &Arc<Engine>, round: u8, proposer: Address) -> Result<Bytes, Error> {
		let mut s = RlpStream::new_list(3);
		let header = Header::default();
		s.append(&round).append(&0u8).append(&header.bare_hash());
		let drain = s.out();
		let propose_rlp = UntrustedRlp::new(&drain);

		engine.handle_message(proposer, H520::default(), propose_rlp)
	}

	fn vote_default(engine: &Arc<Engine>, round: u8, voter: Address) -> Result<Bytes, Error> {
		let mut s = RlpStream::new_list(3);
		let header = Header::default();
		s.append(&round).append(&1u8).append(&header.bare_hash());
		let drain = s.out();
		let vote_rlp = UntrustedRlp::new(&drain);

		engine.handle_message(voter, H520::default(), vote_rlp)
	}

	fn good_seal(header: &Header) -> Vec<Bytes> {
		let tap = AccountProvider::transient_provider();

		let mut seal = Vec::new();

		let v0 = tap.insert_account("0".sha3(), "0").unwrap();
		let sig0 = tap.sign(v0, Some("0".into()), header.bare_hash()).unwrap();
		seal.push(::rlp::encode(&(&*sig0 as &[u8])).to_vec());

		let v1 = tap.insert_account("1".sha3(), "1").unwrap();
		let sig1 = tap.sign(v1, Some("1".into()), header.bare_hash()).unwrap();
		seal.push(::rlp::encode(&(&*sig1 as &[u8])).to_vec());
		seal
	}

	fn default_block() -> Vec<u8> {
		vec![160, 39, 191, 179, 126, 80, 124, 233, 13, 161, 65, 48, 114, 4, 177, 198, 186, 36, 25, 67, 128, 97, 53, 144, 172, 80, 202, 75, 29, 113, 152, 255, 101]
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
		let header: Header = Header::default();

		let verify_result = engine.verify_block_basic(&header, None);

		match verify_result {
			Err(Error::Block(BlockError::InvalidSealArity(_))) => {},
			Err(_) => { panic!("should be block seal-arity mismatch error (got {:?})", verify_result); },
			_ => { panic!("Should be error, got Ok"); },
		}
	}

	#[test]
	fn verification_fails_on_wrong_signatures() {
		let engine = Spec::new_test_tendermint().engine;
		let mut header = Header::default();
		let tap = AccountProvider::transient_provider();

		let mut seal = Vec::new();

		let v1 = tap.insert_account("0".sha3(), "0").unwrap();
		let sig1 = tap.sign(v1, Some("0".into()), header.bare_hash()).unwrap();
		seal.push(::rlp::encode(&(&*sig1 as &[u8])).to_vec());

		header.set_seal(seal.clone());

		// Not enough signatures.
		assert!(engine.verify_block_basic(&header, None).is_err());

		let v2 = tap.insert_account("101".sha3(), "101").unwrap();
		let sig2 = tap.sign(v2, Some("101".into()), header.bare_hash()).unwrap();
		seal.push(::rlp::encode(&(&*sig2 as &[u8])).to_vec());

		header.set_seal(seal);

		// Enough signatures.
		assert!(engine.verify_block_basic(&header, None).is_ok());

		let verify_result = engine.verify_block_unordered(&header, None);

		// But wrong signatures.
		match verify_result {
			Err(Error::Block(BlockError::InvalidSeal)) => (),
			Err(_) => panic!("should be block seal-arity mismatch error (got {:?})", verify_result),
			_ => panic!("Should be error, got Ok"),
		}
	}

	#[test]
	fn seal_with_enough_signatures_is_ok() {
		let engine = Spec::new_test_tendermint().engine;
		let mut header = Header::default();

		let seal = good_seal(&header);
		header.set_seal(seal);

		// Enough signatures.
		assert!(engine.verify_block_basic(&header, None).is_ok());

		// And they are ok.
		assert!(engine.verify_block_unordered(&header, None).is_ok());
	}

	#[test]
	fn can_generate_seal() {
		let spec = Spec::new_test_tendermint();
		let ref engine = *spec.engine;
		let tender = Tendermint::new(engine.params().clone(), TendermintParams::default(), BTreeMap::new());

		let genesis_header = spec.genesis_header();
		let mut db_result = get_temp_state_db();
		let mut db = db_result.take();
		spec.ensure_db_good(&mut db).unwrap();
		let last_hashes = Arc::new(vec![genesis_header.hash()]);
		let b = OpenBlock::new(engine, Default::default(), false, db, &genesis_header, last_hashes, Address::default(), (3141562.into(), 31415620.into()), vec![]).unwrap();
		let b = b.close_and_lock();

		tender.to_commit(b.hash(), good_seal(&b.header()));

		let seal = tender.generate_seal(b.block(), None).unwrap();
		assert!(b.try_seal(engine, seal).is_ok());
	}

	#[test]
	fn propose_step() {
		let engine = Spec::new_test_tendermint().engine;
		let tap = AccountProvider::transient_provider();
		let r = 0;

		let not_authority_addr = tap.insert_account("101".sha3(), "101").unwrap();
		assert!(propose_default(&engine, r, not_authority_addr).is_err());

		let not_proposer_addr = tap.insert_account("0".sha3(), "0").unwrap();
		assert!(propose_default(&engine, r, not_proposer_addr).is_err());

		let proposer_addr = tap.insert_account("1".sha3(), "1").unwrap();
		assert_eq!(default_block(), propose_default(&engine, r, proposer_addr).unwrap());

		assert!(propose_default(&engine, r, proposer_addr).is_err());
		assert!(propose_default(&engine, r, not_proposer_addr).is_err());
	}

	#[test]
	fn proposer_switching() {
		let engine = Spec::new_test_tendermint().engine;
		let tap = AccountProvider::transient_provider();

		// Currently not a proposer.
		let not_proposer_addr = tap.insert_account("0".sha3(), "0").unwrap();
		assert!(propose_default(&engine, 0, not_proposer_addr).is_err());

		sleep(Duration::from_millis(TendermintParams::default().timeouts.propose as u64));

		// Becomes proposer after timeout.
		assert_eq!(default_block(), propose_default(&engine, 0, not_proposer_addr).unwrap());
	}

	#[test]
	fn prevote_step() {
		let engine = Spec::new_test_tendermint().engine;
		let tap = AccountProvider::transient_provider();
		let r = 0;

		let v0 = tap.insert_account("0".sha3(), "0").unwrap();
		let v1 = tap.insert_account("1".sha3(), "1").unwrap();

		// Propose.
		assert!(propose_default(&engine, r, v1.clone()).is_ok());

		// Prevote.
		assert_eq!(default_block(), vote_default(&engine, r, v0.clone()).unwrap());

		assert!(vote_default(&engine, r, v0).is_err());
		assert!(vote_default(&engine, r, v1).is_err());
	}

	#[test]
	fn precommit_step() {
		let engine = Spec::new_test_tendermint().engine;
		let tap = AccountProvider::transient_provider();
		let r = 0;

		let v0 = tap.insert_account("0".sha3(), "0").unwrap();
		let v1 = tap.insert_account("1".sha3(), "1").unwrap();

		// Propose.
		assert!(propose_default(&engine, r, v1.clone()).is_ok());

		// Prevote.
		assert_eq!(default_block(), vote_default(&engine, r, v0.clone()).unwrap());

		assert!(vote_default(&engine, r, v0).is_err());
		assert!(vote_default(&engine, r, v1).is_err());
	}

	#[test]
	fn timeout_switching() {
		let tender = {
			let engine = Spec::new_test_tendermint().engine;
			Tendermint::new(engine.params().clone(), TendermintParams::default(), BTreeMap::new())
		};

		println!("Waiting for timeout");
		sleep(Duration::from_secs(10));
	}

	#[test]
	fn increments_round() {
		let spec = Spec::new_test_tendermint();
		let ref engine = *spec.engine;
		let def_params = TendermintParams::default();
		let tender = Tendermint::new(engine.params().clone(), def_params.clone(), BTreeMap::new());
		let header = Header::default();

		tender.to_commit(header.bare_hash(), good_seal(&header));

		sleep(Duration::from_millis(def_params.timeouts.commit as u64));

		match propose_default(&(tender as Arc<Engine>), 0, Address::default()) {
			Err(Error::Engine(EngineError::WrongRound)) => {},
			_ => panic!("Should be EngineError::WrongRound"),
		}
	}
}
