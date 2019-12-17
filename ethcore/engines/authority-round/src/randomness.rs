// Copyright 2015-2019 Parity Technologies (UK) Ltd.
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

//! On-chain randomness generation for authority round
//!
//! This module contains the support code for the on-chain randomness generation used by AuRa. Its
//! core is the finite state machine `RandomnessPhase`, which can be loaded from the blockchain
//! state, then asked to perform potentially necessary transaction afterwards using the `advance()`
//! method.
//!
//! No additional state is kept inside the `RandomnessPhase`, it must be passed in each time.
//!
//! The process of generating random numbers is a simple finite state machine:
//!
//! ```text
//!                                                       +
//!                                                       |
//!                                                       |
//!                                                       |
//! +--------------+                              +-------v-------+
//! |              |                              |               |
//! | BeforeCommit <------------------------------+    Waiting    |
//! |              |          enter commit phase  |               |
//! +------+-------+                              +-------^-------+
//!        |                                              |
//!        |  call                                        |
//!        |  `commitHash()`                              |  call
//!        |                                              |  `revealNumber()`
//!        |                                              |
//! +------v-------+                              +-------+-------+
//! |              |                              |               |
//! |  Committed   +------------------------------>    Reveal     |
//! |              |  enter reveal phase          |               |
//! +--------------+                              +---------------+
//! ```
//!
//! Phase transitions are performed by the smart contract and simply queried by the engine.
//!
//! Randomness generation works as follows:
//! * During the commit phase, all validators locally generate a random number, and commit that number's hash to the
//!   contract.
//! * During the reveal phase, all validators reveal their local random number to the contract. The contract should
//!   verify that it matches the committed hash.
//! * Finally, the XOR of all revealed numbers is used as an on-chain random number.
//!
//! An adversary can only influence that number by either controlling _all_ validators who committed, or, to a lesser
//! extent, by not revealing committed numbers.
//! The length of the commit and reveal phases, as well as any penalties for failure to reveal, are defined by the
//! contract.
//!
//! A typical case of using `RandomnessPhase` is:
//!
//! 1. `RandomnessPhase::load()` the phase from the blockchain data.
//! 2. Call `RandomnessPhase::advance()`.
//!
//! A production implementation of a randomness contract can be found here:
//! https://github.com/poanetwork/posdao-contracts/blob/4fddb108993d4962951717b49222327f3d94275b/contracts/RandomAuRa.sol

use derive_more::Display;
use ethabi::Hash;
use ethabi_contract::use_contract;
use ethereum_types::{Address, H256, U256};
use keccak_hash::keccak;
use log::{debug, error};
use parity_crypto::publickey::{ecies, Error as CryptoError};
use parity_bytes::Bytes;
use rand::Rng;
use engine::signer::EngineSigner;

use crate::util::{BoundContract, CallError};

/// Random number type expected by the contract: This is generated locally, kept secret during the commit phase, and
/// published in the reveal phase.
pub type RandNumber = H256;

use_contract!(aura_random, "../../res/contracts/authority_round_random.json");

/// Validated randomness phase state.
#[derive(Debug)]
pub enum RandomnessPhase {
	// NOTE: Some states include information already gathered during `load` (e.g. `our_address`,
	//       `round`) for efficiency reasons.
	/// Waiting for the next phase.
	///
	/// This state indicates either the successful revelation in this round or having missed the
	/// window to make a commitment, i.e. having failed to commit during the commit phase.
	Waiting,
	/// Indicates a commitment is possible, but still missing.
	BeforeCommit,
	/// Indicates a successful commitment, waiting for the commit phase to end.
	Committed,
	/// Indicates revealing is expected as the next step.
	Reveal { our_address: Address, round: U256 },
}

/// Phase loading error for randomness generation state machine.
///
/// This error usually indicates a bug in either the smart contract, the phase loading function or
/// some state being lost.
///
/// `BadRandNumber` will usually result in punishment by the contract or the other validators.
#[derive(Debug, Display)]
pub enum PhaseError {
	/// The smart contract reported that we already revealed something while still being in the
	/// commit phase.
	#[display(fmt = "Revealed during commit phase")]
	RevealedInCommit,
	/// Failed to load contract information.
	#[display(fmt = "Error loading randomness contract information: {:?}", _0)]
	LoadFailed(CallError),
	/// Failed to load the stored encrypted random number.
	#[display(fmt = "Failed to load random number from the randomness contract")]
	BadRandNumber,
	/// Failed to encrypt random number.
	#[display(fmt = "Failed to encrypt random number: {}", _0)]
	Crypto(CryptoError),
	/// Failed to get the engine signer's public key.
	#[display(fmt = "Failed to get the engine signer's public key")]
	MissingPublicKey,
}

impl From<CryptoError> for PhaseError {
	fn from(err: CryptoError) -> PhaseError {
		PhaseError::Crypto(err)
	}
}

impl RandomnessPhase {
	/// Determine randomness generation state from the contract.
	///
	/// Calls various constant contract functions to determine the precise state that needs to be
	/// handled (that is, the phase and whether or not the current validator still needs to send
	/// commitments or reveal random numbers).
	pub fn load(
		contract: &BoundContract,
		our_address: Address,
	) -> Result<RandomnessPhase, PhaseError> {
		// Determine the current round and which phase we are in.
		let round = contract
			.call_const(aura_random::functions::current_collect_round::call())
			.map_err(PhaseError::LoadFailed)?;
		let is_commit_phase = contract
			.call_const(aura_random::functions::is_commit_phase::call())
			.map_err(PhaseError::LoadFailed)?;

		// Ensure we are not committing or revealing twice.
		let committed = contract
			.call_const(aura_random::functions::is_committed::call(
				round,
				our_address,
			))
			.map_err(PhaseError::LoadFailed)?;
		let revealed: bool = contract
			.call_const(aura_random::functions::sent_reveal::call(
				round,
				our_address,
			))
			.map_err(PhaseError::LoadFailed)?;

		// With all the information known, we can determine the actual state we are in.
		if is_commit_phase {
			if revealed {
				return Err(PhaseError::RevealedInCommit);
			}

			if !committed {
				Ok(RandomnessPhase::BeforeCommit)
			} else {
				Ok(RandomnessPhase::Committed)
			}
		} else {
			if !committed {
				// We apparently entered too late to make a commitment, wait until we get a chance again.
				return Ok(RandomnessPhase::Waiting);
			}

			if !revealed {
				Ok(RandomnessPhase::Reveal { our_address, round })
			} else {
				Ok(RandomnessPhase::Waiting)
			}
		}
	}

	/// Advance the random seed construction process as far as possible.
	///
	/// Returns the encoded contract call necessary to advance the randomness contract's state.
	///
	/// **Warning**: After calling the `advance()` function, wait until the returned transaction has been included in
	/// a block before calling it again; otherwise spurious transactions resulting in punishments might be executed.
	pub fn advance<R: Rng>(
		self,
		contract: &BoundContract,
		rng: &mut R,
		signer: &dyn EngineSigner,
	) -> Result<Option<Bytes>, PhaseError> {
		match self {
			RandomnessPhase::Waiting | RandomnessPhase::Committed => Ok(None),
			RandomnessPhase::BeforeCommit => {
				// Generate a new random number, but don't reveal it yet. Instead, we publish its hash to the
				// randomness contract, together with the number encrypted to ourselves. That way we will later be
				// able to decrypt and reveal it, and other parties are able to verify it against the hash.
				let number: RandNumber = rng.gen();
				let number_hash: Hash = keccak(number.as_bytes());
				let public = signer.public().ok_or(PhaseError::MissingPublicKey)?;
				let cipher = ecies::encrypt(&public, &number_hash.0, number.as_bytes())?;

				debug!(target: "engine", "Randomness contract: committing {}.", number_hash);
				// Return the call data for the transaction that commits the hash and the encrypted number.
				let (data, _decoder) = aura_random::functions::commit_hash::call(number_hash, cipher);
				Ok(Some(data))
			}
			RandomnessPhase::Reveal { round, our_address } => {
				// Load the hash and encrypted number that we stored in the commit phase.
				let call = aura_random::functions::get_commit_and_cipher::call(round, our_address);
				let (committed_hash, cipher) = contract
					.call_const(call)
					.map_err(PhaseError::LoadFailed)?;

				// Decrypt the number and check against the hash.
				let number_bytes = signer.decrypt(&committed_hash.0, &cipher)?;
				let number = if number_bytes.len() == 32 {
					RandNumber::from_slice(&number_bytes)
				} else {
					// This can only happen if there is a bug in the smart contract,
					// or if the entire network goes awry.
					error!(target: "engine", "Decrypted random number has the wrong length.");
					return Err(PhaseError::BadRandNumber);
				};
				let number_hash: Hash = keccak(number.as_bytes());
				if number_hash != committed_hash {
					error!(target: "engine", "Decrypted random number doesn't agree with the hash.");
					return Err(PhaseError::BadRandNumber);
				}

				debug!(target: "engine", "Randomness contract: scheduling tx to reveal our random number {} (round={}, our_address={}).", number_hash, round, our_address);
				// We are now sure that we have the correct secret and can reveal it. So we return the call data for the
				// transaction that stores the revealed random bytes on the contract.
				let (data, _decoder) = aura_random::functions::reveal_number::call(number.as_bytes());
				Ok(Some(data))
			}
		}
	}
}
