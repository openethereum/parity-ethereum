mod signer_snapshot;
mod params;

use rlp::{encode_list, encode, Decodable, DecoderError, Encodable, RlpStream, Rlp};
use std::time::Duration;
use itertools::Itertools;

use std::sync::{Weak, Arc};
use std::collections::{BTreeMap, HashMap};
use std::{fmt, error};
use std::str::FromStr;
use hash::keccak;

use self::params::CliqueParams;

use super::epoch::{PendingTransition, EpochVerifier, NoOp};

use account_provider::AccountProvider;
use builtin::Builtin;
use vm::{EnvInfo, Schedule, CreateContractAddress, CallType, ActionValue};
use error::Error;
use header::{Header, BlockNumber, ExtendedHeader};
use snapshot::SnapshotComponents;
use spec::CommonParams;
use transaction::{self, UnverifiedTransaction, SignedTransaction};
use parking_lot::RwLock;
use block::*;
use io::IoService;

use ethkey::{Password, Signature, recover as ec_recover, public_to_address};
use parity_machine::{Machine, LocalizedMachine as Localized, TotalScoredHeader};
use ethereum_types::{H256, U256, Address, Public};
use unexpected::{Mismatch, OutOfBounds};
use bytes::Bytes;
use types::ancestry_action::AncestryAction;
use engines::{Engine, Seal, EngineError, ConstructedVerifier, Headers, PendingTransitionStore};
use super::signer::EngineSigner;
use machine::{Call, AuxiliaryData, EthereumMachine};
use self::signer_snapshot::{CliqueState, SignerAuthorization, NONCE_AUTH_VOTE, NONCE_DROP_VOTE, NULL_AUTHOR};

pub const SIGNER_VANITY_LENGTH: u32 = 32;
// Fixed number of extra-data prefix bytes reserved for signer vanity
//const EXTRA_DATA_POST_LENGTH: u32 = 128;
pub const SIGNER_SIG_LENGTH: u32 = 65; // Fixed number of extra-data suffix bytes reserved for signer seal

use client::{EngineClient, BlockId};

pub struct Clique {
	client: RwLock<Option<Weak<EngineClient>>>,
	state: RwLock<CliqueState>,
	//signers: RwLock<Option<Vec<Address>>>,
	machine: EthereumMachine,
	epoch_length: u64,
	period: u64,
}

/*
 * only sign over non-signature bytes (vanity data).  There shouldn't be a signature here to sign
 * yet.
 */
pub fn sig_hash(header: &Header) -> Result<H256, Error> {
	if header.extra_data().len() >= SIGNER_VANITY_LENGTH as usize {
		let extra_data = header.extra_data().clone();
		let mut reduced_header = header.clone();
		reduced_header.set_extra_data(
			extra_data[..extra_data.len() - SIGNER_SIG_LENGTH as usize].to_vec());

		Ok(reduced_header.hash())
	} else {
		Ok(header.hash())
	}
}

fn recover(header: &Header) -> Result<Public, Error> {
	let data = header.extra_data();
	let mut sig: [u8; 65] = [0; 65];
	sig.copy_from_slice(&data[(data.len() - SIGNER_SIG_LENGTH as usize)..]);

	let msg = sig_hash(header).unwrap();
	let pubkey = ec_recover(&Signature::from(sig), &msg).unwrap();

	Ok(pubkey)
}

const step_time: Duration = Duration::from_millis(100);

impl Clique {

	pub fn new(our_params: CliqueParams, machine: EthereumMachine) -> Result<Arc<Self>, Error> {
		// don't let there be any duplicate signers

		//length of signers must be greater than 1
		//

		trace!(target: "engine", "epoch length: {}, period: {}", our_params.epoch, our_params.period);
		/*
		let snapshot = SignerSnapshot {
		  bn: 0,
		  signers: vec![],
		  epoch_length: our_params.epoch,
		  votes: HashMap::<Address, (bool, Address)>::new(),
		};
		*/

		let engine = Arc::new(
			Clique {
				client: RwLock::new(Option::default()),
				state: RwLock::new(CliqueState::new(our_params.epoch)),
				machine: machine,
				epoch_length: our_params.epoch,
				period: our_params.period,
			});

		return Ok(engine);
	}

//	fn sign_header(&self, header: &Header) -> Result<(Signature, H256), Error> {
//		let digest = sig_hash(header)?;
//		if let Some(sig) = self.snapshot.sign_data(&digest) {
//			return Ok((sig, digest));
//		}
//
//		return Err(From::from("failed to sign header"));
//	}

	//pub fn snapshot(self, bn: u64) -> AuthorizationSnapshot {
	// if we are on a checkpoint block, build a snapshot
	//}
}

impl Engine<EthereumMachine> for Clique {
	fn name(&self) -> &str { "Clique" }

	// nonce + mixHash + extraData
	fn seal_fields(&self, _header: &Header) -> usize { 2 }
	fn machine(&self) -> &EthereumMachine { &self.machine }
	fn maximum_uncle_count(&self, _block: BlockNumber) -> usize { 0 }

	// called only when sealing ?
	fn populate_from_parent(&self, header: &mut Header, parent: &Header) {
	}

//	// only called when we are sealing the block.  TODO rename this to make more sense
//	fn close_block_extra_data(&self, _header: &Header) -> Option<Vec<u8>> {
//		let mut h = _header.clone();
//
//		trace!(target: "engine", "applying sealed block");
//		let mut v: Vec<u8> = vec![0; SIGNER_VANITY_LENGTH as usize + SIGNER_SIG_LENGTH as usize];
//
//		{
//			let signers = self.state.get_signers();
//			trace!(target: "engine", "applied.  found {} signers", signers.len());
//
//			//let mut v: Vec<u8> = vec![0; SIGNER_VANITY_LENGTH as usize+SIGNER_SIG_LENGTH as usize];
//			let mut sig_offset = SIGNER_VANITY_LENGTH as usize;
//
//			if _header.number() % self.epoch_length == 0 {
//				sig_offset += 20 * signers.len();
//
//				for i in 0..signers.len() {
//					v[SIGNER_VANITY_LENGTH as usize + i * 20..SIGNER_VANITY_LENGTH as usize + (i + 1) * 20].clone_from_slice(&signers[i]);
//				}
//			}
//
//			h.set_extra_data(v.clone());
//
//			let (sig, msg) = self.sign_header(&h).expect("should be able to sign header");
//			v[sig_offset..].copy_from_slice(&sig[..]);
//
//			trace!(target: "engine", "header hash: {}", h.hash());
//			trace!(target: "engine", "Sig: {}", sig);
//			trace!(target: "engine", "Message: {:02x}", msg.iter().format(""));
//
//			//trace!(target: "engine", "we are {}", self.signer.read().address().unwrap());
//		}
//
//		return Some(v);
//	}

//	fn set_signer(&self, ap: Arc<AccountProvider>, address: Address, password: Password) {
//		self.snapshot.set_signer(ap, address, password);
//		trace!(target: "engine", "set the signer to {}", address);
//	}

	/// None means that it requires external input (e.g. PoW) to seal a block.
	/// /// Some(true) means the engine is currently prime for seal generation (i.e. node
	///     is the current validator).
	/// /// Some(false) means that the node might seal internally but is not qualified
	///     now.
	///
	fn seals_internally(&self) -> Option<bool> {
		Some(false)
	}

	/// Attempt to seal generate a proposal seal.
	///
	/// This operation is synchronous and may (quite reasonably) not be available, in which case
	/// `Seal::None` will be returned.
	fn generate_seal(&self, block: &ExecutedBlock, _parent: &Header) -> Seal {
        trace!(target: "engine", "tried to generate seal");
		Seal::None
//
//		let mut header = block.header.clone();
//
//		trace!(target: "engine", "attempting to seal...");
//
//		// don't seal the genesis block
//		if header.number() == 0 {
//			trace!(target: "engine", "attempted to seal genesis block");
//			return Seal::None;
//		}
//
//		// if sealing period is 0, refuse to seal
//
//		// let vote_snapshot = self.snapshot.get(bh);
//
//		// if we are not authorized to sign, don't seal
//
//		// if we signed recently, don't seal
//
//		if block.header.timestamp() <= _parent.timestamp() + self.period {
//			trace!(target: "engine", "block too early");
//			return Seal::None;
//		}
//
//		if let SignerAuthorization::Unauthorized = self.snapshot.get_own_authorization() {
//			return Seal::None;
//		}
//
//		// sign the digest of the seal
//		if self.is_signer_proposer(block.header().number()) {
//			trace!(target: "engine", "seal generated for {}", block.header().number());
//			//TODO add our vote here if this is not an epoch transition
//			return Seal::Regular(vec![encode(&vec![0; 32]), encode(&vec![0; 8])]);
//		} else {
//			trace!(target: "engine", "we are not the current for block {}", block.header().number());
//			Seal::None
//		}
	}

	fn on_close_block(&self, block: &mut ExecutedBlock) -> Result<(), Error> {
		/*
		 * TODO:
		if not checkpoint block:
		  if the block was successfully sealed, then grab the signature from the seal data and
		  append it to the block extraData
		  */
		// trace!(target: "engine", "closing block {}...", block.header().number());

		Ok(())
	}

	fn on_new_block(
		&self,
		_block: &mut ExecutedBlock,
		_epoch_begin: bool,
		_ancestry: &mut Iterator<Item=ExtendedHeader>,
	) -> Result<(), Error> {
		//trace!(target: "engine", "new block {}", _block.header().number());

		/*
		if let Some(ref mut snapshot) = *self.snapshot.write() {
		  snapshot.rollback();
		} else {
		  panic!("could not get write access to snapshot");
		}
		*/

		/*
		if let Some(ref mut snapshot) = *self.snapshot.write() {
			snapshot.apply(_block.header());
		}
		*/

		Ok(())
	}

	fn executive_author(&self, header: &Header) -> Address {
		trace!(target: "engine", "called executive_author for block {}", header.number());

//		if self.is_signer_proposer(header.number()) {
//			return self.snapshot.signer_address().unwrap();
//		} else {
			return public_to_address(
				&recover(header).unwrap());
//		}
	}

	fn verify_block_basic(&self, _header: &Header) -> Result<(), Error> {
		// Ignore genisis block.
		if _header.number() == 0 {
			return Ok(());
		  }

		// don't allow blocks from the future
		// Checkpoint blocks need to enforce zero beneficiary
		if _header.number() % self.epoch_length == 0 {
			if _header.author() != &[0; 20].into() {
				return Err(Box::new("Checkpoint blocks need to enforce zero beneficiary").into());
			}
			let nonce = _header.decode_seal::<Vec<&[u8]>>().unwrap()[1];
			if nonce != &NONCE_DROP_VOTE[..] {
				return Err(Box::new("Seal nonce zeros enforced on checkpoints").into());
			}
		} else {
			// TODO
			// - ensure header extraData has length SIGNER_VANITY_LENGTH + SIGNER_SIG_LENGTH
			// - ensure header signature corresponds to the right validator for the turn-ness of the
			// block
		}
		// Nonces must be 0x00..0 or 0xff..f, zeroes enforced on checkpoints
		// Check that the extra-data contains both the vanity and signature
		// Ensure that the extra-data contains a signer list on checkpoint, but none otherwise
		// Ensure that the mix digest is zero as we don't have fork protection currently
		// Ensure that the block doesn't contain any uncles which are meaningless in PoA
		// Ensure that the block's difficulty is meaningful
		// ...

		Ok(())
	}

//	fn on_block_applied(&self, header: &Header) -> Result<(), Error> {
//		self.snapshot.apply(&header);
//		self.snapshot.commit();
//
//		Ok(())
//	}

	fn verify_block_unordered(&self, _header: &Header) -> Result<(), Error> {
		// Verifying the genesis block is not supported
		// Retrieve the snapshot needed to verify this header and cache it
		// Resolve the authorization key and check against signers
		// Ensure that the difficulty corresponds to the turn-ness of the signer
		Ok(())
	}

	fn verify_block_family(&self, header: &Header, parent: &Header) -> Result<(), Error> {
		trace!(target: "engine",
		       "verify_block_family for {}: current state: {:?}.",
		       header.number(), *self.state.read());

		let mut state = self.state.write();

		// see if we have parent state
		if state.state(&parent.hash()).is_none() {
			let client = self.client.read();
			if let Some(c) = client.as_ref().and_then(|w|{ w.upgrade()}) {
				let last_checkpoint_number = (parent.number() / self.epoch_length as u64) * self.epoch_length;
				let mut chain: &mut Vec<Header> = &mut Vec::new();
				chain.push(parent.clone());

				// populate chain to last checkpoint
				let mut last = chain.last().unwrap().clone();

				while last.number() != last_checkpoint_number +1 {
					if let Some(next) = c.block_header(BlockId::Hash(*last.parent_hash())) {
						chain.push(next.decode().unwrap().clone());
						last = chain.last().unwrap().clone();
					} else {
						return Err(From::from("No parent state exist."));
					}
				}

				// Get the last checkpoint header
				if let Some(last_checkpoint_header) = c.block_header(BlockId::Hash(*chain.last().unwrap().parent_hash())) {
					state.apply_checkpoint(&last_checkpoint_header.decode().unwrap())?;
				}
				// Catching up state.
				chain.reverse();

				trace!(target: "engine",
				       "verify_block_family backfilling state. last_checkpoint: {}, chain: {:?}.",
				       last_checkpoint_number, chain);

				for item in chain {
					state.apply(item)?;
				}
			}
		}
		if (header.number() % self.epoch_length == 0) {
			// TODO: we may still need to validate checkpoint state
			state.apply_checkpoint(header);
		} else {
			state.apply(header)?;
		}

		Ok(())
	}

	fn signals_epoch_end(&self, header: &Header, aux: AuxiliaryData)
	                     -> super::EpochChange<EthereumMachine>
	{
		super::EpochChange::No
	}

	fn is_epoch_end(
		&self,
		chain_head: &Header,
		_finalized: &[H256],
		_chain: &Headers<Header>,
		_transition_store: &PendingTransitionStore,
	) -> Option<Vec<u8>> {
		None
	}

	fn epoch_verifier<'a>(&self, _header: &Header, proof: &'a [u8]) -> ConstructedVerifier<'a, EthereumMachine> {
		ConstructedVerifier::Trusted(Box::new(super::epoch::NoOp))
	}

	fn stop(&self) {}

	fn verify_local_seal(&self, header: &Header) -> Result<(), Error> { Ok(()) }

	fn fork_choice(&self, new: &ExtendedHeader, current: &ExtendedHeader) -> super::ForkChoice {
		super::total_difficulty_fork_choice(new, current)
	}

	/*
	 *  Extract signer addresses from header extraData
	 */
	fn genesis_epoch_data(&self, header: &Header, call: &Call) -> Result<Vec<u8>, String> {
		// extract signer list from genesis extradata
		trace!(target: "engine", "genesis_epoch_data received.");

		{
			let mut state = self.state.write();
			state.apply_checkpoint(header).expect("Error processing genesis block");
			trace!(target: "engine", "current state: {:?}", *state);
		}

		Ok(Vec::new())
	}

	fn is_timestamp_valid(&self, header_timestamp: u64, parent_timestamp: u64) -> bool {
		header_timestamp >= parent_timestamp + self.period
	}

	//	/// Check if current signer is the current proposer.
//	fn is_signer_proposer(&self, bn: u64) -> bool {
//		let mut authorized = false;
//
//		let address = match self.snapshot.signer_address() {
//			Some(addr) => { addr }
//			None => { return false; }
//		};
//
//		let signers = self.snapshot.get_signers();
//
//		let authorized = if let Some(pos) = signers.iter().position(|x| self.snapshot.signer_address().unwrap() == *x) {
//			bn % signers.len() as u64 == pos as u64
//		} else {
//			false
//		};
//		return authorized;
//	}

	fn register_client(&self, client: Weak<EngineClient>) {
		trace!(target: "engine", "client regsitered.");
		*self.client.write() = Some(client.clone());
	}

}
