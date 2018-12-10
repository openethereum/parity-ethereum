use std::sync::Weak;
use client::{EngineClient, BlockId};
use ethkey::{public_to_address, Signature};
use ethereum_types::{Address, H256, U256};
use std::collections::{HashMap, VecDeque};
use engines::clique::{SIGNER_SIG_LENGTH, SIGNER_VANITY_LENGTH, recover};
use error::Error;
use header::{Header, ExtendedHeader};
use super::super::signer::EngineSigner;
use parking_lot::RwLock;
use std::sync::Arc;
use account_provider::AccountProvider;
use ethkey::Password;
use std::borrow::BorrowMut;

pub const NONCE_DROP_VOTE: &[u8; 8] = &[0x00; 8];
pub const NONCE_AUTH_VOTE: &[u8; 8] = &[0xff; 8];
pub const NULL_AUTHOR: [u8; 20] = [0; 20];
pub const DIFF_INTURN: u8 = 2;
pub const DIFF_NOT_INTURN: u8 = 1;

pub enum SignerAuthorization {
	InTurn,
	OutOfTurn,
	Unauthorized,
}

#[derive(Debug)]
pub struct CliqueBlock {
	is_checkpoint_block: bool,
	creator: Address,
	header: Header,
}

pub struct CliqueState {
	epoch_length: u64,
	states_by_hash: HashMap<H256, SnapshotState>,
}

#[derive(Clone, Debug)]
pub struct SnapshotState {
	pub votes: Vec<(Address, bool, Address)>,
	pub signers: Vec<Address>,
}

impl CliqueState {
	pub fn new(epoch_length: u64) -> Self {
		CliqueState {
			epoch_length: epoch_length,
			states_by_hash: HashMap::new(),
		}
	}

	/// Get an valid state
	pub fn state(&self, hash: &H256) -> Option<SnapshotState> {
		return self.states_by_hash.get(hash).cloned();
	}

	/// Apply an new header
	pub fn apply(&mut self, header: &Header) -> Result<(), Error> {
		let db = self.states_by_hash.borrow_mut();

		// make sure current hash is not in the db
		match db.get(header.parent_hash()).cloned() {
			Some(ref mut new_state) => {
				process_header(&header, new_state, self.epoch_length)?;
				db.insert(header.hash(), new_state.clone());
				Ok(())

			}
			None => {
				Err(From::from(
					format!("Parent block (hash: {}) for Block {}, hash {} is not found!",
					        header.parent_hash(),
					        header.number(), header.hash() )))
			}
		}
	}

	pub fn apply_checkpoint(&mut self, header: &Header) -> Result<(), Error> {
		let db = self.states_by_hash.borrow_mut();
		let state = &mut SnapshotState {
			votes: Vec::new(),
			signers: Vec::new(),
		};
		process_genesis_header(header, state)?;
		db.insert(header.hash(), state.clone());

		Ok(())
	}
}

impl std::fmt::Debug for CliqueState {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(f, "CliqueState {{ epoch: {:?}, states: {:?} }}", self.epoch_length, self.states_by_hash)
	}
}

fn extract_signers(header: &Header) -> Result<Vec<Address>, Error> {
	let min_extra_data_size = (SIGNER_VANITY_LENGTH as usize) + (SIGNER_SIG_LENGTH as usize);

	assert!(header.extra_data().len() >= min_extra_data_size, "need minimum genesis extra data size {}.  found {}.", min_extra_data_size, header.extra_data().len());

	// extract only the portion of extra_data which includes the signer list
	let signers_raw = &header.extra_data()[(SIGNER_VANITY_LENGTH as usize)..header.extra_data().len() - (SIGNER_SIG_LENGTH as usize)];

	assert_eq!(signers_raw.len() % 20, 0, "bad signer list length {}", signers_raw.len());

	let num_signers = signers_raw.len() / 20;
	let mut signers_list: Vec<Address> = vec![];

	for i in 0..num_signers {
		let mut signer = Address::default();
		signer.copy_from_slice(&signers_raw[i * 20..(i + 1) * 20]);
		signers_list.push(signer);
	}
	// NOTE: base on geth implmentation , signers list area always sorted to ascending order.
	signers_list.sort();

	trace!(target: "engine", "extracted signers {:?}", &signers_list);
	Ok(signers_list)
}

impl SnapshotState {
	pub fn get_signer_authorization(&self, currentBlockNumber: u64, author: Address) -> SignerAuthorization {
		// TODO: Implement recent signer check list.
		if let Some(pos) = self.signers.iter().position(|x| author == *x) {
			if currentBlockNumber % self.signers.len() as u64 == pos as u64 {
				return SignerAuthorization::InTurn;
			} else {
				return SignerAuthorization::OutOfTurn;
			}
		}
		return SignerAuthorization::Unauthorized;
	}
}

fn process_genesis_header(header: &Header, state: &mut SnapshotState) -> Result<(), Error> {
	assert_eq!(header.number(), 0, "header is not for gensis block.");

	state.signers = extract_signers(header)?;
	state.votes.clear();

	Ok(())
}

pub fn process_header(header: &Header, state: &mut SnapshotState, epoch_length: u64) -> Result<(), Error> {
	// Check signature & dificulty
	let creator = public_to_address(&recover(header).unwrap()).clone();

	match state.get_signer_authorization(header.number(),creator) {
		SignerAuthorization::InTurn => {
			if *header.difficulty() != U256::from(DIFF_INTURN) {
				return Err(From::from("difficulty must be set to DIFF_INTURN"));
			}
		}
		SignerAuthorization::OutOfTurn => {
			if *header.difficulty() != U256::from(DIFF_NOT_INTURN) {
				return Err(From::from("difficulty must be set to DIFF_NOT_INTURN"));
			}
		}
		SignerAuthorization::Unauthorized => {
			return Err(From::from(
				format!("unauthorized to sign at this time: current state: {:?}, creator: {}", state, creator )
			));
		}
	}

	// If this is checkpoint blocks
	if header.number() % epoch_length == 0 {
		state.signers = extract_signers(header)?;
		state.votes.clear();
		return Ok(());
	}

	// non checkpoint block and no votes,  we just ignore.
	if header.author()[0..20] == NULL_AUTHOR {
		return Ok(());
	}

	//TODO: votes that reach a majority consensus should have effects applied immediately to the signer list
	let nonce = header.decode_seal::<Vec<&[u8]>>().unwrap()[1];
	let mut author = header.author().clone();
	if nonce == NONCE_DROP_VOTE {
		state.votes.push((creator, false, author));
	} else if nonce == NONCE_AUTH_VOTE {
		state.votes.push((creator, true, author));
	} else {
		return Err(From::from("beneficiary specificed but nonce was not AUTH or DROP"));
	}

	return Ok(());
}

