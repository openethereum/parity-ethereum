//! On-chain randomness generation for authority round
//!
//! This module contains the support code for the on-chain randomness generation used by AuRa. Its
//! core is the finite state machine `RandomnessPhase`, which can be loaded from the blockchain
//! state, then asked to perform potentially necessary transaction afterwards using the `advance()`
//! method.
//!
//! No additional state is kept inside the `RandomnessPhase`, it must be passed in each time.

use ethabi::Hash;
use ethereum_types::{Address, U256};
use hash::keccak;
use rand::Rng;

use super::util::{BoundContract, CallError};

/// Secret type expected by the contract.
// Note: Conversion from `U256` back into `[u8; 32]` is cumbersome (missing implementations), for
//       this reason we store the raw buffers.
pub type Secret = [u8; 32];

use_contract!(aura_random, "res/contracts/authority_round_random.json");

/// Validated randomness phase state.
///
/// The process of generating random numbers is a simple finite state machine:
///
/// ```text
///                                                       +
///                                                       |
///                                                       |
///                                                       |
/// +--------------+                              +-------v-------+
/// |              |                              |               |
/// | BeforeCommit <------------------------------+    Waiting    |
/// |              |          enter commit phase  |               |
/// +------+-------+                              +-------^-------+
///        |                                              |
///        |  call                                        |
///        |  `commitHash()`                              |  call
///        |                                              |  `revealSecret`
///        |                                              |
/// +------v-------+                              +-------+-------+
/// |              |                              |               |
/// |  Committed   +------------------------------>    Reveal     |
/// |              |  enter reveal phase          |               |
/// +--------------+                              +---------------+
/// ```
///
///
/// Phase transitions are performed by the smart contract and simply queried by the engine.
///
/// A typical case of using `RandomnessPhase` is:
///
/// 1. `RandomnessPhase::load()` the phase from the blockchain data.
/// 2. Load the stored secret.
/// 3. Call `RandomnessPhase::advance()` with the stored secret.
/// 4. Update the stored secret with the return value.
#[derive(Debug)]
pub enum RandomnessPhase {
	// NOTE: Some states include information already gathered during `load` (e.g. `our_address`,
	//       `round`) for efficiency reasons.
	/// Waiting for the next phase.
	///
	/// This state indicates either the successful revelation in this round or having missed the
	/// window to make a commitment.
	Waiting,
	/// Indicates a commitment is possible, but still missing.
	BeforeCommit { our_address: Address, round: U256 },
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
/// The `LostSecret` and `StaleSecret` will usually result in punishment by the contract or the
/// other validators.
#[derive(Debug)]
pub enum PhaseError {
	/// The smart contract reported a phase as both commitment and reveal phase.
	PhaseConflict,
	/// The smart contract reported that we already revealed something while still being in the
	/// commit phase.
	RevealedInCommit,
	/// Calling a contract to determine the phase failed.
	LoadFailed(CallError),
	/// Failed to schedule a transaction to call a contract.
	TransactionFailed(CallError),
	/// When trying to reveal the secret, no secret was found.
	LostSecret,
	/// When trying to reveal the secret, the stored secret had the wrong round number.
	UnexpectedRound(U256),
	/// A secret was stored, but it did not match the committed hash.
	StaleSecret,
}

impl RandomnessPhase {
	/// Determine randomness generation state from the contract.
	///
	/// Calls various constant contract functions to determine the precise state that needs to be
	/// handled (that is, the phase and whether or not the current validator still needs to send
	/// commitments or reveal secrets).
	pub fn load(
		contract: &BoundContract,
		our_address: Address,
	) -> Result<RandomnessPhase, PhaseError> {
		// Determine the current round and which phase we are in.
		let round = contract
			.call_const(aura_random::functions::current_collect_round::call())
			.map_err(PhaseError::LoadFailed)?;
		let is_reveal_phase = contract
			.call_const(aura_random::functions::is_reveal_phase::call())
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
		if is_reveal_phase && is_commit_phase {
			return Err(PhaseError::PhaseConflict);
		}

		if is_commit_phase {
			if revealed {
				return Err(PhaseError::RevealedInCommit);
			}

			if !committed {
				Ok(RandomnessPhase::BeforeCommit { our_address, round })
			} else {
				Ok(RandomnessPhase::Committed)
			}
		} else {
			if !committed {
				// We apparently entered too late to make a committment, wait until we get a chance
				// again.
				return Ok(RandomnessPhase::Waiting);
			}

			if !revealed {
				Ok(RandomnessPhase::Reveal { our_address, round })
			} else {
				Ok(RandomnessPhase::Waiting)
			}
		}
	}

	/// Advance the randomness state, if necessary.
	///
	/// Creates the transaction necessary to advance the randomness contract's state and schedules
	/// them to run on the `client` inside `contract`. The `stored_secret` must be managed by the
	/// the caller, passed in each time `advance` is called and replaced with the returned value
	/// each time the function returns successfully.
	///
	/// **Warning**: The `advance()` function should be called only once per block state; otherwise
	///              spurious transactions resulting in punishments might be executed.
	pub fn advance<R: Rng>(
		self,
		contract: &BoundContract,
		stored_secret: Option<(U256, Secret)>,
		rng: &mut R,
	) -> Result<Option<(U256, Secret)>, PhaseError> {
		match self {
			RandomnessPhase::Waiting | RandomnessPhase::Committed => Ok(stored_secret),
			RandomnessPhase::BeforeCommit { round, .. } => {
				// We generate a new secret to submit each time, this function will only be called
				// once per round of randomness generation.

				let secret: Secret = match stored_secret {
					Some((r, secret)) if r == round => secret,
					_ => {
						let mut buf = [0u8; 32];
						rng.fill_bytes(&mut buf);
						buf.into()
					}
				};
				let secret_hash: Hash = keccak(secret.as_ref());

				// Schedule the transaction that commits the hash.
				let data = aura_random::functions::commit_hash::call(secret_hash);
				contract.schedule_service_transaction(data).map_err(PhaseError::TransactionFailed)?;

				// Store the newly generated secret.
				Ok(Some((round, secret)))
			}
			RandomnessPhase::Reveal { round, our_address } => {
				let (r, secret) = stored_secret.ok_or(PhaseError::LostSecret)?;
				if r != round {
					return Err(PhaseError::UnexpectedRound(r));
				}

				// We hash our secret again to check against the already committed hash:
				let secret_hash: Hash = keccak(secret.as_ref());
				let committed_hash: Hash = contract
					.call_const(aura_random::functions::get_commit::call(round, our_address))
					.map_err(PhaseError::LoadFailed)?;

				if secret_hash != committed_hash {
					return Err(PhaseError::StaleSecret);
				}

				// We are now sure that we have the correct secret and can reveal it.
				let data = aura_random::functions::reveal_secret::call(secret);
				contract.schedule_service_transaction(data).map_err(PhaseError::TransactionFailed)?;

				// We still pass back the secret -- if anything fails later down the line, we can
				// resume by simply creating another transaction.
				Ok(Some((round, secret)))
			}
		}
	}
}
