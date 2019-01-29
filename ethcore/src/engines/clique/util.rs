use std::collections::HashMap;
use std::collections::VecDeque;
use std::fmt;
use std::mem;
use std::sync::{Arc, Weak};
use std::time::Duration;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

use ethereum_types::{Address, H160, H256, Public, U256};
use hash::KECCAK_EMPTY_LIST_RLP;
use lru_cache::LruCache;
use parking_lot::RwLock;
use parking_lot::RwLockUpgradableReadGuard;
use rand::thread_rng;
use rlp::encode;

use account_provider::AccountProvider;
use block::*;
use client::{BlockId, EngineClient};
use engines::{ConstructedVerifier, Engine, Headers, PendingTransitionStore, Seal};
use error::Error;
use ethkey::{Password, public_to_address, recover as ec_recover, Signature};
use io::IoService;
use machine::{AuxiliaryData, Call, EthereumMachine};
use types::BlockNumber;
use types::header::{ExtendedHeader, Header};
use engines::clique::{SIGNER_SIG_LENGTH, SIGNER_VANITY_LENGTH};

/// How many recovered signature to cache in the memory.
pub const CREATOR_CACHE_NUM: usize = 4096;
lazy_static! {
	/// key: header hash
	/// value: creator address
	static ref CREATOR_BY_HASH: RwLock<LruCache<H256, Address>> = RwLock::new(LruCache::new(CREATOR_CACHE_NUM));
}

/// Recover block creator from signature
pub fn recover_creator(header: &Header) -> Result<Address, Error> {
	// Initialization
	let mut cache = CREATOR_BY_HASH.write();

	if let Some(creator) = cache.get_mut(&header.hash()) {
		return Ok(*creator);
	}

	let data = header.extra_data();
	let mut sig_data = data[data.len() - SIGNER_SIG_LENGTH..].to_vec();
	sig_data.resize(SIGNER_SIG_LENGTH, 0);

	let mut sig = [0; SIGNER_SIG_LENGTH];
	sig.copy_from_slice(&sig_data[..]);

	let reduced_header = &mut header.clone();
	reduced_header.set_extra_data(data[..data.len() - SIGNER_SIG_LENGTH].to_vec());

	let msg = reduced_header.hash();
	let pubkey = ec_recover(&Signature::from(sig), &msg)?;
	let creator = public_to_address(&pubkey);

	cache.insert(header.hash(), creator.clone());
	Ok(creator)
}


/// Extract signer list from extra_data.
///
/// Layout of extra_data:
/// ----
/// VANITY: 32 bytes
/// Signers: N * 32 bytes as hex encoded (20 characters)
/// Signature: 65 bytes
/// --
pub fn extract_signers(header: &Header) -> Result<Vec<Address>, Error> {
	let data = header.extra_data();

	if data.len() <= SIGNER_VANITY_LENGTH + SIGNER_SIG_LENGTH {
		return Err(Box::new("Invalid extra_data size.").into());
	}

	// extract only the portion of extra_data which includes the signer list
	let signers_raw = &data[(SIGNER_VANITY_LENGTH)..data.len() - (SIGNER_SIG_LENGTH)];

	if signers_raw.len() % 20 != 0 {
		return Err(Box::new("bad signer list.").into());
	}

	let num_signers = signers_raw.len() / 20;
	let mut signers_list: Vec<Address> = Vec::with_capacity(num_signers);

	for i in 0..num_signers {
		let mut signer = Address::default();
		signer.copy_from_slice(&signers_raw[i * 20..(i + 1) * 20]);
		signers_list.push(signer);
	}

	// NOTE: signers list must be sorted by ascending order.
	signers_list.sort();

	Ok(signers_list)
}
