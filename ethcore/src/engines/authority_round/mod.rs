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

//! A blockchain engine that supports a non-instant BFT proof-of-authority.

use std::{
    collections::{BTreeMap, HashSet},
    ops::Deref,
    sync::{
        atomic::{AtomicBool, AtomicUsize, Ordering as AtomicOrdering},
        Arc, Weak,
    },
    time::{Duration, UNIX_EPOCH},
};

use self::finality::RollingFinality;
use super::{
    signer::EngineSigner,
    validator_set::{new_validator_set, SimpleList, ValidatorSet},
};
use block::*;
use client::{traits::ForceUpdateSealing, EngineClient};
use engines::{
    block_reward,
    block_reward::{BlockRewardContract, RewardKind},
    ConstructedVerifier, Engine, EngineError, Seal,
};
use error::{BlockError, Error, ErrorKind};
use ethereum_types::{Address, H256, H520, U128, U256};
use ethjson;
use ethkey::{self, Signature};
use io::{IoContext, IoHandler, IoService, TimerToken};
use itertools::{self, Itertools};
use machine::{AuxiliaryData, Call, EthereumMachine};
use parking_lot::{Mutex, RwLock};
use rlp::{encode, Rlp};
use time_utils::CheckedSystemTime;
use types::{
    ancestry_action::AncestryAction,
    header::{ExtendedHeader, Header},
    BlockNumber,
};
use unexpected::{Mismatch, OutOfBounds};

mod finality;

const EXPECTED_SEAL_FIELDS: usize = 2;

/// `AuthorityRound` params.
pub struct AuthorityRoundParams {
    /// Time to wait before next block or authority switching,
    /// in seconds.
    ///
    /// Deliberately typed as u16 as too high of a value leads
    /// to slow block issuance.
    pub step_duration: u16,
    /// Starting step,
    pub start_step: Option<u64>,
    /// Valid validators.
    pub validators: Box<dyn ValidatorSet>,
    /// Chain score validation transition block.
    pub validate_score_transition: u64,
    /// Monotonic step validation transition block.
    pub validate_step_transition: u64,
    /// Immediate transitions.
    pub immediate_transitions: bool,
    /// Block reward in base units.
    pub block_reward: U256,
    /// Block reward contract transition block.
    pub block_reward_contract_transition: u64,
    /// Block reward contract.
    pub block_reward_contract: Option<BlockRewardContract>,
    /// Number of accepted uncles transition block.
    pub maximum_uncle_count_transition: u64,
    /// Number of accepted uncles.
    pub maximum_uncle_count: usize,
}

const U16_MAX: usize = ::std::u16::MAX as usize;

impl From<ethjson::spec::AuthorityRoundParams> for AuthorityRoundParams {
    fn from(p: ethjson::spec::AuthorityRoundParams) -> Self {
        let mut step_duration_usize: usize = p.step_duration.into();
        if step_duration_usize > U16_MAX {
            step_duration_usize = U16_MAX;
            warn!(target: "engine", "step_duration is too high ({}), setting it to {}", step_duration_usize, U16_MAX);
        }
        AuthorityRoundParams {
            step_duration: step_duration_usize as u16,
            validators: new_validator_set(p.validators),
            start_step: p.start_step.map(Into::into),
            validate_score_transition: p.validate_score_transition.map_or(0, Into::into),
            validate_step_transition: p.validate_step_transition.map_or(0, Into::into),
            immediate_transitions: p.immediate_transitions.unwrap_or(false),
            block_reward: p.block_reward.map_or_else(Default::default, Into::into),
            block_reward_contract_transition: p
                .block_reward_contract_transition
                .map_or(0, Into::into),
            block_reward_contract: match (
                p.block_reward_contract_code,
                p.block_reward_contract_address,
            ) {
                (Some(code), _) => Some(BlockRewardContract::new_from_code(Arc::new(code.into()))),
                (_, Some(address)) => Some(BlockRewardContract::new_from_address(address.into())),
                (None, None) => None,
            },
            maximum_uncle_count_transition: p.maximum_uncle_count_transition.map_or(0, Into::into),
            maximum_uncle_count: p.maximum_uncle_count.map_or(0, Into::into),
        }
    }
}

// Helper for managing the step.
#[derive(Debug)]
struct Step {
    calibrate: bool, // whether calibration is enabled.
    inner: AtomicUsize,
    duration: u16,
}

impl Step {
    fn load(&self) -> u64 {
        self.inner.load(AtomicOrdering::SeqCst) as u64
    }
    fn duration_remaining(&self) -> Duration {
        let now = unix_now();
        let expected_seconds = self
            .load()
            .checked_add(1)
            .and_then(|ctr| ctr.checked_mul(self.duration as u64))
            .map(Duration::from_secs);

        match expected_seconds {
            Some(step_end) if step_end > now => step_end - now,
            Some(_) => Duration::from_secs(0),
            None => {
                let ctr = self.load();
                error!(target: "engine", "Step counter is too high: {}, aborting", ctr);
                panic!("step counter is too high: {}", ctr)
            }
        }
    }

    fn increment(&self) {
        use std::usize;
        // fetch_add won't panic on overflow but will rather wrap
        // around, leading to zero as the step counter, which might
        // lead to unexpected situations, so it's better to shut down.
        if self.inner.fetch_add(1, AtomicOrdering::SeqCst) == usize::MAX {
            error!(target: "engine", "Step counter is too high: {}, aborting", usize::MAX);
            panic!("step counter is too high: {}", usize::MAX);
        }
    }

    fn calibrate(&self) {
        if self.calibrate {
            let new_step = unix_now().as_secs() / (self.duration as u64);
            self.inner.store(new_step as usize, AtomicOrdering::SeqCst);
        }
    }

    fn check_future(&self, given: u64) -> Result<(), Option<OutOfBounds<u64>>> {
        const REJECTED_STEP_DRIFT: u64 = 4;

        // Verify if the step is correct.
        if given <= self.load() {
            return Ok(());
        }

        // Make absolutely sure that the given step is incorrect.
        self.calibrate();
        let current = self.load();

        // reject blocks too far in the future
        if given > current + REJECTED_STEP_DRIFT {
            Err(None)
        // wait a bit for blocks in near future
        } else if given > current {
            let d = self.duration as u64;
            Err(Some(OutOfBounds {
                min: None,
                max: Some(d * current),
                found: d * given,
            }))
        } else {
            Ok(())
        }
    }
}

// Chain scoring: total weight is sqrt(U256::max_value())*height - step
fn calculate_score(parent_step: u64, current_step: u64) -> U256 {
    U256::from(U128::max_value()) + U256::from(parent_step) - U256::from(current_step)
}

struct EpochManager {
    epoch_transition_hash: H256,
    epoch_transition_number: BlockNumber,
    finality_checker: RollingFinality,
    force: bool,
}

impl EpochManager {
    fn blank() -> Self {
        EpochManager {
            epoch_transition_hash: H256::default(),
            epoch_transition_number: 0,
            finality_checker: RollingFinality::blank(Vec::new()),
            force: true,
        }
    }

    // zoom to epoch for given header. returns true if succeeded, false otherwise.
    fn zoom_to(
        &mut self,
        client: &dyn EngineClient,
        machine: &EthereumMachine,
        validators: &dyn ValidatorSet,
        header: &Header,
    ) -> bool {
        let last_was_parent = self.finality_checker.subchain_head() == Some(*header.parent_hash());

        // early exit for current target == chain head, but only if the epochs are
        // the same.
        if last_was_parent && !self.force {
            return true;
        }

        self.force = false;
        debug!(target: "engine", "Zooming to epoch for block {}", header.hash());

        // epoch_transition_for can be an expensive call, but in the absence of
        // forks it will only need to be called for the block directly after
        // epoch transition, in which case it will be O(1) and require a single
        // DB lookup.
        let last_transition = match client.epoch_transition_for(*header.parent_hash()) {
            Some(t) => t,
            None => {
                // this really should never happen unless the block passed
                // hasn't got a parent in the database.
                debug!(target: "engine", "No genesis transition found.");
                return false;
            }
        };

        // extract other epoch set if it's not the same as the last.
        if last_transition.block_hash != self.epoch_transition_hash {
            let (signal_number, set_proof, _) = destructure_proofs(&last_transition.proof)
                .expect("proof produced by this engine; therefore it is valid; qed");

            trace!(target: "engine", "extracting epoch set for epoch ({}, {}) signalled at #{}",
				last_transition.block_number, last_transition.block_hash, signal_number);

            let first = signal_number == 0;
            let epoch_set = validators
                .epoch_set(
                    first,
                    machine,
                    signal_number, // use signal number so multi-set first calculation is correct.
                    set_proof,
                )
                .ok()
                .map(|(list, _)| list.into_inner())
                .expect("proof produced by this engine; therefore it is valid; qed");

            self.finality_checker = RollingFinality::blank(epoch_set);
        }

        self.epoch_transition_hash = last_transition.block_hash;
        self.epoch_transition_number = last_transition.block_number;

        true
    }

    // note new epoch hash. this will force the next block to re-load
    // the epoch set
    // TODO: optimize and don't require re-loading after epoch change.
    fn note_new_epoch(&mut self) {
        self.force = true;
    }

    /// Get validator set. Zoom to the correct epoch first.
    fn validators(&self) -> &SimpleList {
        self.finality_checker.validators()
    }
}

struct PermissionedStep {
    inner: Step,
    can_propose: AtomicBool,
}

/// Engine using `AuthorityRound` proof-of-authority BFT consensus.
pub struct AuthorityRound {
    transition_service: IoService<()>,
    step: Arc<PermissionedStep>,
    client: Arc<RwLock<Option<Weak<dyn EngineClient>>>>,
    signer: RwLock<Option<Box<dyn EngineSigner>>>,
    validators: Box<dyn ValidatorSet>,
    validate_score_transition: u64,
    validate_step_transition: u64,
    epoch_manager: Mutex<EpochManager>,
    immediate_transitions: bool,
    block_reward: U256,
    block_reward_contract_transition: u64,
    block_reward_contract: Option<BlockRewardContract>,
    maximum_uncle_count_transition: u64,
    maximum_uncle_count: usize,
    machine: EthereumMachine,
}

// header-chain validator.
struct EpochVerifier {
    step: Arc<PermissionedStep>,
    subchain_validators: SimpleList,
}

impl super::EpochVerifier<EthereumMachine> for EpochVerifier {
    fn verify_light(&self, header: &Header) -> Result<(), Error> {
        // Validate the timestamp
        verify_timestamp(&self.step.inner, header_step(header)?)?;
        // always check the seal since it's fast.
        // nothing heavier to do.
        verify_external(header, &self.subchain_validators)
    }

    fn check_finality_proof(&self, proof: &[u8]) -> Option<Vec<H256>> {
        let mut finality_checker =
            RollingFinality::blank(self.subchain_validators.clone().into_inner());
        let mut finalized = Vec::new();

        let headers: Vec<Header> = Rlp::new(proof).as_list().ok()?;

        {
            let mut push_header = |parent_header: &Header, header: Option<&Header>| {
                // ensure all headers have correct number of seal fields so we can `verify_external` without panic.
                if parent_header.seal().len() != EXPECTED_SEAL_FIELDS {
                    return None;
                }
                if header
                    .iter()
                    .any(|h| h.seal().len() != EXPECTED_SEAL_FIELDS)
                {
                    return None;
                }

                // `verify_external` checks that signature is correct and author == signer.
                verify_external(parent_header, &self.subchain_validators).ok()?;

                let newly_finalized = finality_checker
                    .push_hash(parent_header.hash(), *parent_header.author())
                    .ok()?;
                finalized.extend(newly_finalized);

                Some(())
            };

            for window in headers.windows(2) {
                push_header(&window[0], Some(&window[1]))?;
            }

            if let Some(last) = headers.last() {
                push_header(last, None)?;
            }
        }

        if finalized.is_empty() {
            None
        } else {
            Some(finalized)
        }
    }
}

fn header_seal_hash(header: &Header) -> H256 {
    header.bare_hash()
}

fn header_step(header: &Header) -> Result<u64, ::rlp::DecoderError> {
    Rlp::new(&header.seal().get(0).unwrap_or_else(||
		panic!("was either checked with verify_block_basic or is genesis; has {} fields; qed (Make sure the spec
				file has a correct genesis seal)", EXPECTED_SEAL_FIELDS)
	))
	.as_val()
}

fn header_signature(header: &Header) -> Result<Signature, ::rlp::DecoderError> {
    Rlp::new(&header.seal().get(1).unwrap_or_else(|| {
        panic!(
            "was checked with verify_block_basic; has {} fields; qed",
            EXPECTED_SEAL_FIELDS
        )
    }))
    .as_val::<H520>()
    .map(Into::into)
}

fn step_proposer(validators: &dyn ValidatorSet, bh: &H256, step: u64) -> Address {
    let proposer = validators.get(bh, step as usize);
    trace!(target: "engine", "Fetched proposer for step {}: {}", step, proposer);
    proposer
}

fn is_step_proposer(
    validators: &dyn ValidatorSet,
    bh: &H256,
    step: u64,
    address: &Address,
) -> bool {
    step_proposer(validators, bh, step) == *address
}

fn verify_timestamp(step: &Step, header_step: u64) -> Result<(), BlockError> {
    match step.check_future(header_step) {
        Err(None) => {
            trace!(target: "engine", "verify_timestamp: block from the future");
            Err(BlockError::InvalidSeal.into())
        }
        Err(Some(oob)) => {
            // NOTE This error might be returned only in early stage of verification (Stage 1).
            // Returning it further won't recover the sync process.
            trace!(target: "engine", "verify_timestamp: block too early");

            let found = CheckedSystemTime::checked_add(UNIX_EPOCH, Duration::from_secs(oob.found))
                .ok_or(BlockError::TimestampOverflow)?;
            let max = oob
                .max
                .and_then(|m| CheckedSystemTime::checked_add(UNIX_EPOCH, Duration::from_secs(m)));
            let min = oob
                .min
                .and_then(|m| CheckedSystemTime::checked_add(UNIX_EPOCH, Duration::from_secs(m)));

            let new_oob = OutOfBounds { min, max, found };

            Err(BlockError::TemporarilyInvalid(new_oob).into())
        }
        Ok(_) => Ok(()),
    }
}

fn verify_external(header: &Header, validators: &dyn ValidatorSet) -> Result<(), Error> {
    let header_step = header_step(header)?;

    let proposer_signature = header_signature(header)?;
    let correct_proposer = validators.get(header.parent_hash(), header_step as usize);
    let is_invalid_proposer = *header.author() != correct_proposer || {
        let header_seal_hash = header_seal_hash(header);
        !ethkey::verify_address(&correct_proposer, &proposer_signature, &header_seal_hash)?
    };

    if is_invalid_proposer {
        trace!(target: "engine", "verify_block_external: bad proposer for step: {}", header_step);
        Err(EngineError::NotProposer(Mismatch {
            expected: correct_proposer,
            found: *header.author(),
        }))?
    } else {
        Ok(())
    }
}

fn combine_proofs(signal_number: BlockNumber, set_proof: &[u8], finality_proof: &[u8]) -> Vec<u8> {
    let mut stream = ::rlp::RlpStream::new_list(3);
    stream
        .append(&signal_number)
        .append(&set_proof)
        .append(&finality_proof);
    stream.out()
}

fn destructure_proofs(combined: &[u8]) -> Result<(BlockNumber, &[u8], &[u8]), Error> {
    let rlp = Rlp::new(combined);
    Ok((rlp.at(0)?.as_val()?, rlp.at(1)?.data()?, rlp.at(2)?.data()?))
}

trait AsMillis {
    fn as_millis(&self) -> u64;
}

impl AsMillis for Duration {
    fn as_millis(&self) -> u64 {
        self.as_secs() * 1_000 + (self.subsec_nanos() / 1_000_000) as u64
    }
}

// A type for storing owned or borrowed data that has a common type.
// Useful for returning either a borrow or owned data from a function.
enum CowLike<'a, A: 'a + ?Sized, B> {
    Borrowed(&'a A),
    Owned(B),
}

impl<'a, A: ?Sized, B> Deref for CowLike<'a, A, B>
where
    B: AsRef<A>,
{
    type Target = A;
    fn deref(&self) -> &A {
        match self {
            CowLike::Borrowed(b) => b,
            CowLike::Owned(o) => o.as_ref(),
        }
    }
}

impl AuthorityRound {
    /// Create a new instance of AuthorityRound engine.
    pub fn new(
        our_params: AuthorityRoundParams,
        machine: EthereumMachine,
    ) -> Result<Arc<Self>, Error> {
        if our_params.step_duration == 0 {
            error!(target: "engine", "Authority Round step duration can't be zero, aborting");
            panic!("authority_round: step duration can't be zero")
        }
        let should_timeout = our_params.start_step.is_none();
        let initial_step = our_params
            .start_step
            .unwrap_or_else(|| (unix_now().as_secs() / (our_params.step_duration as u64)));
        let engine = Arc::new(AuthorityRound {
            transition_service: IoService::<()>::start()?,
            step: Arc::new(PermissionedStep {
                inner: Step {
                    inner: AtomicUsize::new(initial_step as usize),
                    calibrate: our_params.start_step.is_none(),
                    duration: our_params.step_duration,
                },
                can_propose: AtomicBool::new(true),
            }),
            client: Arc::new(RwLock::new(None)),
            signer: RwLock::new(None),
            validators: our_params.validators,
            validate_score_transition: our_params.validate_score_transition,
            validate_step_transition: our_params.validate_step_transition,
            epoch_manager: Mutex::new(EpochManager::blank()),
            immediate_transitions: our_params.immediate_transitions,
            block_reward: our_params.block_reward,
            block_reward_contract_transition: our_params.block_reward_contract_transition,
            block_reward_contract: our_params.block_reward_contract,
            maximum_uncle_count_transition: our_params.maximum_uncle_count_transition,
            maximum_uncle_count: our_params.maximum_uncle_count,
            machine: machine,
        });

        // Do not initialize timeouts for tests.
        if should_timeout {
            let handler = TransitionHandler {
                step: engine.step.clone(),
                client: engine.client.clone(),
            };
            engine
                .transition_service
                .register_handler(Arc::new(handler))?;
        }
        Ok(engine)
    }

    // fetch correct validator set for epoch at header, taking into account
    // finality of previous transitions.
    fn epoch_set<'a>(
        &'a self,
        header: &Header,
    ) -> Result<(CowLike<dyn ValidatorSet, SimpleList>, BlockNumber), Error> {
        Ok(if self.immediate_transitions {
            (CowLike::Borrowed(&*self.validators), header.number())
        } else {
            let mut epoch_manager = self.epoch_manager.lock();
            let client = match self.client.read().as_ref().and_then(|weak| weak.upgrade()) {
                Some(client) => client,
                None => {
                    debug!(target: "engine", "Unable to verify sig: missing client ref.");
                    return Err(EngineError::RequiresClient.into());
                }
            };

            if !epoch_manager.zoom_to(&*client, &self.machine, &*self.validators, header) {
                debug!(target: "engine", "Unable to zoom to epoch.");
                return Err(EngineError::RequiresClient.into());
            }

            (
                CowLike::Owned(epoch_manager.validators().clone()),
                epoch_manager.epoch_transition_number,
            )
        })
    }

    fn report_skipped(
        &self,
        header: &Header,
        current_step: u64,
        parent_step: u64,
        validators: &dyn ValidatorSet,
        set_number: u64,
    ) {
        // we're building on top of the genesis block so don't report any skipped steps
        if header.number() == 1 {
            return;
        }

        if let (true, Some(me)) = (
            current_step > parent_step + 1,
            self.signer.read().as_ref().map(|s| s.address()),
        ) {
            debug!(target: "engine", "Author {} built block with step gap. current step: {}, parent step: {}",
				   header.author(), current_step, parent_step);
            let mut reported = HashSet::new();
            for step in parent_step + 1..current_step {
                let skipped_primary = step_proposer(validators, header.parent_hash(), step);
                // Do not report this signer.
                if skipped_primary != me {
                    // Stop reporting once validators start repeating.
                    if !reported.insert(skipped_primary) {
                        break;
                    }
                    self.validators
                        .report_benign(&skipped_primary, set_number, header.number());
                }
            }
        }
    }

    // Returns the hashes of all ancestor blocks that are finalized by the given `chain_head`.
    fn build_finality(
        &self,
        chain_head: &Header,
        ancestry: &mut dyn Iterator<Item = Header>,
    ) -> Vec<H256> {
        if self.immediate_transitions {
            return Vec::new();
        }

        let client = match self.client.read().as_ref().and_then(|weak| weak.upgrade()) {
            Some(client) => client,
            None => {
                warn!(target: "engine", "Unable to apply ancestry actions: missing client ref.");
                return Vec::new();
            }
        };

        let mut epoch_manager = self.epoch_manager.lock();
        if !epoch_manager.zoom_to(&*client, &self.machine, &*self.validators, chain_head) {
            return Vec::new();
        }

        if epoch_manager.finality_checker.subchain_head() != Some(*chain_head.parent_hash()) {
            // build new finality checker from unfinalized ancestry of chain head, not including chain head itself yet.
            trace!(target: "finality", "Building finality up to parent of {} ({})",
				   chain_head.hash(), chain_head.parent_hash());

            let epoch_transition_hash = epoch_manager.epoch_transition_hash;
            let ancestry_iter = ancestry
                .map(|header| {
                    let res = (header.hash(), *header.author());
                    trace!(target: "finality", "Ancestry iteration: yielding {:?}", res);

                    Some(res)
                })
                .while_some()
                .take_while(|&(h, _)| h != epoch_transition_hash);

            if let Err(e) = epoch_manager
                .finality_checker
                .build_ancestry_subchain(ancestry_iter)
            {
                debug!(target: "engine", "inconsistent validator set within epoch: {:?}", e);
                return Vec::new();
            }
        }

        let finalized = epoch_manager
            .finality_checker
            .push_hash(chain_head.hash(), *chain_head.author());
        finalized.unwrap_or_default()
    }
}

fn unix_now() -> Duration {
    UNIX_EPOCH
        .elapsed()
        .expect("Valid time has to be set in your system.")
}

struct TransitionHandler {
    step: Arc<PermissionedStep>,
    client: Arc<RwLock<Option<Weak<dyn EngineClient>>>>,
}

const ENGINE_TIMEOUT_TOKEN: TimerToken = 23;

impl IoHandler<()> for TransitionHandler {
    fn initialize(&self, io: &IoContext<()>) {
        let remaining = AsMillis::as_millis(&self.step.inner.duration_remaining());
        io.register_timer_once(ENGINE_TIMEOUT_TOKEN, Duration::from_millis(remaining))
            .unwrap_or_else(
                |e| warn!(target: "engine", "Failed to start consensus step timer: {}.", e),
            )
    }

    fn timeout(&self, io: &IoContext<()>, timer: TimerToken) {
        if timer == ENGINE_TIMEOUT_TOKEN {
            // NOTE we might be lagging by couple of steps in case the timeout
            // has not been called fast enough.
            // Make sure to advance up to the actual step.
            while AsMillis::as_millis(&self.step.inner.duration_remaining()) == 0 {
                self.step.inner.increment();
                self.step.can_propose.store(true, AtomicOrdering::SeqCst);
                if let Some(ref weak) = *self.client.read() {
                    if let Some(c) = weak.upgrade() {
                        c.update_sealing(ForceUpdateSealing::No);
                    }
                }
            }

            let next_run_at = AsMillis::as_millis(&self.step.inner.duration_remaining()) >> 2;
            io.register_timer_once(ENGINE_TIMEOUT_TOKEN, Duration::from_millis(next_run_at))
                .unwrap_or_else(
                    |e| warn!(target: "engine", "Failed to restart consensus step timer: {}.", e),
                )
        }
    }
}

impl Engine<EthereumMachine> for AuthorityRound {
    fn name(&self) -> &str {
        "AuthorityRound"
    }

    fn machine(&self) -> &EthereumMachine {
        &self.machine
    }

    /// Two fields - consensus step and the corresponding proposer signature
    fn seal_fields(&self) -> usize {
        EXPECTED_SEAL_FIELDS
    }

    fn step(&self) {
        self.step.inner.increment();
        self.step.can_propose.store(true, AtomicOrdering::SeqCst);
        if let Some(ref weak) = *self.client.read() {
            if let Some(c) = weak.upgrade() {
                c.update_sealing(ForceUpdateSealing::No);
            }
        }
    }

    /// Additional engine-specific information for the user/developer concerning `header`.
    fn extra_info(&self, header: &Header) -> BTreeMap<String, String> {
        if header.seal().len() < EXPECTED_SEAL_FIELDS {
            return BTreeMap::default();
        }

        let step = header_step(header)
            .as_ref()
            .map(ToString::to_string)
            .unwrap_or_default();
        let signature = header_signature(header)
            .as_ref()
            .map(ToString::to_string)
            .unwrap_or_default();

        let info = map![
            "step".into() => step,
            "signature".into() => signature
        ];

        info
    }

    fn maximum_uncle_count(&self, block: BlockNumber) -> usize {
        if block >= self.maximum_uncle_count_transition {
            self.maximum_uncle_count
        } else {
            // fallback to default value
            2
        }
    }

    fn populate_from_parent(&self, header: &mut Header, parent: &Header) {
        let parent_step = header_step(parent).expect("Header has been verified; qed");
        let current_step = self.step.inner.load();

        let score = calculate_score(parent_step, current_step);
        header.set_difficulty(score);
    }

    fn seals_internally(&self) -> Option<bool> {
        // TODO: accept a `&Call` here so we can query the validator set.
        Some(self.signer.read().is_some())
    }

    /// Attempt to seal the block internally.
    ///
    /// This operation is synchronous and may (quite reasonably) not be available, in which case
    /// `Seal::None` will be returned.
    fn generate_seal(&self, block: &ExecutedBlock, parent: &Header) -> Seal {
        // first check to avoid generating signature most of the time
        // (but there's still a race to the `compare_and_swap`)
        if !self.step.can_propose.load(AtomicOrdering::SeqCst) {
            trace!(target: "engine", "Aborting seal generation. Can't propose.");
            return Seal::None;
        }

        let header = &block.header;
        let parent_step = header_step(parent).expect("Header has been verified; qed");

        let step = self.step.inner.load();

        let expected_diff = calculate_score(parent_step, step.into());

        if header.difficulty() != &expected_diff {
            debug!(target: "engine", "Aborting seal generation. The step has changed in the meantime. {:?} != {:?}",
				   header.difficulty(), expected_diff);
            return Seal::None;
        }

        if parent_step > step.into() {
            warn!(target: "engine", "Aborting seal generation for invalid step: {} > {}", parent_step, step);
            return Seal::None;
        }

        let (validators, set_number) = match self.epoch_set(header) {
            Err(err) => {
                warn!(target: "engine", "Unable to generate seal: {}", err);
                return Seal::None;
            }
            Ok(ok) => ok,
        };

        if is_step_proposer(&*validators, header.parent_hash(), step, header.author()) {
            // this is guarded against by `can_propose` unless the block was signed
            // on the same step (implies same key) and on a different node.
            if parent_step == step {
                warn!("Attempted to seal block on the same step as parent. Is this authority sealing with more than one node?");
                return Seal::None;
            }

            if let Ok(signature) = self.sign(header_seal_hash(header)) {
                trace!(target: "engine", "generate_seal: Issuing a block for step {}.", step);

                // only issue the seal if we were the first to reach the compare_and_swap.
                if self
                    .step
                    .can_propose
                    .compare_and_swap(true, false, AtomicOrdering::SeqCst)
                {
                    // report any skipped primaries between the parent block and the block we're sealing
                    self.report_skipped(header, step, parent_step, &*validators, set_number);

                    let fields = vec![encode(&step), encode(&(&H520::from(signature) as &[u8]))];

                    return Seal::Regular(fields);
                }
            } else {
                warn!(target: "engine", "generate_seal: FAIL: Accounts secret key unavailable.");
            }
        } else {
            trace!(target: "engine", "generate_seal: {} not a proposer for step {}.",
				header.author(), step);
        }

        Seal::None
    }

    fn verify_local_seal(&self, _header: &Header) -> Result<(), Error> {
        Ok(())
    }

    fn on_new_block(
        &self,
        block: &mut ExecutedBlock,
        epoch_begin: bool,
        _ancestry: &mut dyn Iterator<Item = ExtendedHeader>,
    ) -> Result<(), Error> {
        // with immediate transitions, we don't use the epoch mechanism anyway.
        // the genesis is always considered an epoch, but we ignore it intentionally.
        if self.immediate_transitions || !epoch_begin {
            return Ok(());
        }

        // genesis is never a new block, but might as well check.
        let header = block.header.clone();
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
    fn on_close_block(&self, block: &mut ExecutedBlock) -> Result<(), Error> {
        let author = *block.header.author();
        let reward_kind = RewardKind::Author;

        let rewards: Vec<_> = match self.block_reward_contract {
            Some(ref c) if block.header.number() >= self.block_reward_contract_transition => {
                let mut call = super::default_system_or_code_call(&self.machine, block);

                let rewards = c.reward(&vec![(author, reward_kind)], &mut call)?;
                rewards
                    .into_iter()
                    .map(|(author, amount)| (author, RewardKind::External, amount))
                    .collect()
            }
            _ => vec![(author, reward_kind, self.block_reward)],
        };

        block_reward::apply_block_rewards(&rewards, block, &self.machine)
    }

    /// Check the number of seal fields.
    fn verify_block_basic(&self, header: &Header) -> Result<(), Error> {
        if header.number() >= self.validate_score_transition
            && *header.difficulty() >= U256::from(U128::max_value())
        {
            return Err(From::from(BlockError::DifficultyOutOfBounds(OutOfBounds {
                min: None,
                max: Some(U256::from(U128::max_value())),
                found: *header.difficulty(),
            })));
        }

        match verify_timestamp(&self.step.inner, header_step(header)?) {
            Err(BlockError::InvalidSeal) => {
                // This check runs in Phase 1 where there is no guarantee that the parent block is
                // already imported, therefore the call to `epoch_set` may fail. In that case we
                // won't report the misbehavior but this is not a concern because:
                // - Only authorities can report and it's expected that they'll be up-to-date and
                //   importing, therefore the parent header will most likely be available
                // - Even if you are an authority that is syncing the chain, the contract will most
                //   likely ignore old reports
                // - This specific check is only relevant if you're importing (since it checks
                //   against wall clock)
                if let Ok((_, set_number)) = self.epoch_set(header) {
                    self.validators
                        .report_benign(header.author(), set_number, header.number());
                }

                Err(BlockError::InvalidSeal.into())
            }
            Err(e) => Err(e.into()),
            Ok(()) => Ok(()),
        }
    }

    /// Do the step and gas limit validation.
    fn verify_block_family(&self, header: &Header, parent: &Header) -> Result<(), Error> {
        let step = header_step(header)?;
        let parent_step = header_step(parent)?;

        let (validators, set_number) = self.epoch_set(header)?;

        // Ensure header is from the step after parent.
        if step == parent_step
            || (header.number() >= self.validate_step_transition && step <= parent_step)
        {
            trace!(target: "engine", "Multiple blocks proposed for step {}.", parent_step);

            self.validators.report_malicious(
                header.author(),
                set_number,
                header.number(),
                Default::default(),
            );
            Err(EngineError::DoubleVote(*header.author()))?;
        }

        self.report_skipped(header, step, parent_step, &*validators, set_number);

        if header.number() >= self.validate_score_transition {
            let expected_difficulty = calculate_score(parent_step.into(), step.into());
            if header.difficulty() != &expected_difficulty {
                return Err(From::from(BlockError::InvalidDifficulty(Mismatch {
                    expected: expected_difficulty,
                    found: header.difficulty().clone(),
                })));
            }
        }

        Ok(())
    }

    // Check the validators.
    fn verify_block_external(&self, header: &Header) -> Result<(), Error> {
        let (validators, set_number) = self.epoch_set(header)?;

        // verify signature against fixed list, but reports should go to the
        // contract itself.
        let res = verify_external(header, &*validators);
        if let Err(Error(ErrorKind::Engine(EngineError::NotProposer(_)), _)) = res {
            self.validators
                .report_benign(header.author(), set_number, header.number());
        }
        res
    }

    fn genesis_epoch_data(&self, header: &Header, call: &Call) -> Result<Vec<u8>, String> {
        self.validators
            .genesis_epoch_data(header, call)
            .map(|set_proof| combine_proofs(0, &set_proof, &[]))
    }

    fn signals_epoch_end(
        &self,
        header: &Header,
        aux: AuxiliaryData,
    ) -> super::EpochChange<EthereumMachine> {
        if self.immediate_transitions {
            return super::EpochChange::No;
        }

        let first = header.number() == 0;
        self.validators.signals_epoch_end(first, header, aux)
    }

    fn is_epoch_end_light(
        &self,
        chain_head: &Header,
        chain: &super::Headers<Header>,
        transition_store: &super::PendingTransitionStore,
    ) -> Option<Vec<u8>> {
        // epochs only matter if we want to support light clients.
        if self.immediate_transitions {
            return None;
        }

        let epoch_transition_hash = {
            let client = match self.client.read().as_ref().and_then(|weak| weak.upgrade()) {
                Some(client) => client,
                None => {
                    warn!(target: "engine", "Unable to check for epoch end: missing client ref.");
                    return None;
                }
            };

            let mut epoch_manager = self.epoch_manager.lock();
            if !epoch_manager.zoom_to(&*client, &self.machine, &*self.validators, chain_head) {
                return None;
            }

            epoch_manager.epoch_transition_hash
        };

        let mut hash = *chain_head.parent_hash();

        let mut ancestry = itertools::repeat_call(move || {
            chain(hash).and_then(|header| {
                if header.number() == 0 {
                    return None;
                }
                hash = *header.parent_hash();
                Some(header)
            })
        })
        .while_some()
        .take_while(|header| header.hash() != epoch_transition_hash);

        let finalized = self.build_finality(chain_head, &mut ancestry);

        self.is_epoch_end(chain_head, &finalized, chain, transition_store)
    }

    fn is_epoch_end(
        &self,
        chain_head: &Header,
        finalized: &[H256],
        chain: &super::Headers<Header>,
        transition_store: &super::PendingTransitionStore,
    ) -> Option<Vec<u8>> {
        // epochs only matter if we want to support light clients.
        if self.immediate_transitions {
            return None;
        }

        let first = chain_head.number() == 0;

        // Apply transitions that don't require finality and should be enacted immediately (e.g from chain spec)
        if let Some(change) = self.validators.is_epoch_end(first, chain_head) {
            info!(target: "engine", "Immediately applying validator set change signalled at block {}", chain_head.number());
            self.epoch_manager.lock().note_new_epoch();
            let change = combine_proofs(chain_head.number(), &change, &[]);
            return Some(change);
        }

        // check transition store for pending transitions against recently finalized blocks
        for finalized_hash in finalized {
            if let Some(pending) = transition_store(*finalized_hash) {
                // walk the chain backwards from current head until finalized_hash
                // to construct transition proof. author == ec_recover(sig) known
                // since the blocks are in the DB.
                let mut hash = chain_head.hash();
                let mut finality_proof: Vec<_> = itertools::repeat_call(move || {
                    chain(hash).and_then(|header| {
                        hash = *header.parent_hash();
                        if header.number() == 0 {
                            None
                        } else {
                            Some(header)
                        }
                    })
                })
                .while_some()
                .take_while(|h| h.hash() != *finalized_hash)
                .collect();

                let finalized_header = if *finalized_hash == chain_head.hash() {
                    // chain closure only stores ancestry, but the chain head is also unfinalized.
                    chain_head.clone()
                } else {
                    chain(*finalized_hash).expect(
                        "header is finalized; finalized headers must exist in the chain; qed",
                    )
                };

                let signal_number = finalized_header.number();
                info!(target: "engine", "Applying validator set change signalled at block {}", signal_number);

                finality_proof.push(finalized_header);
                finality_proof.reverse();

                let finality_proof = ::rlp::encode_list(&finality_proof);

                self.epoch_manager.lock().note_new_epoch();

                // We turn off can_propose here because upon validator set change there can
                // be two valid proposers for a single step: one from the old set and
                // one from the new.
                //
                // This way, upon encountering an epoch change, the proposer from the
                // new set will be forced to wait until the next step to avoid sealing a
                // block that breaks the invariant that the parent's step < the block's step.
                self.step.can_propose.store(false, AtomicOrdering::SeqCst);
                return Some(combine_proofs(
                    signal_number,
                    &pending.proof,
                    &*finality_proof,
                ));
            }
        }

        None
    }

    fn epoch_verifier<'a>(
        &self,
        _header: &Header,
        proof: &'a [u8],
    ) -> ConstructedVerifier<'a, EthereumMachine> {
        let (signal_number, set_proof, finality_proof) = match destructure_proofs(proof) {
            Ok(x) => x,
            Err(e) => return ConstructedVerifier::Err(e),
        };

        let first = signal_number == 0;
        match self
            .validators
            .epoch_set(first, &self.machine, signal_number, set_proof)
        {
            Ok((list, finalize)) => {
                let verifier = Box::new(EpochVerifier {
                    step: self.step.clone(),
                    subchain_validators: list,
                });

                match finalize {
                    Some(finalize) => {
                        ConstructedVerifier::Unconfirmed(verifier, finality_proof, finalize)
                    }
                    None => ConstructedVerifier::Trusted(verifier),
                }
            }
            Err(e) => ConstructedVerifier::Err(e),
        }
    }

    fn register_client(&self, client: Weak<dyn EngineClient>) {
        *self.client.write() = Some(client.clone());
        self.validators.register_client(client);
    }

    fn set_signer(&self, signer: Box<dyn EngineSigner>) {
        *self.signer.write() = Some(signer);
    }

    fn sign(&self, hash: H256) -> Result<Signature, Error> {
        Ok(self
            .signer
            .read()
            .as_ref()
            .ok_or(ethkey::Error::InvalidAddress)?
            .sign(hash)?)
    }

    fn snapshot_components(&self) -> Option<Box<dyn crate::snapshot::SnapshotComponents>> {
        if self.immediate_transitions {
            None
        } else {
            Some(Box::new(::snapshot::PoaSnapshot))
        }
    }

    fn fork_choice(&self, new: &ExtendedHeader, current: &ExtendedHeader) -> super::ForkChoice {
        super::total_difficulty_fork_choice(new, current)
    }

    fn ancestry_actions(
        &self,
        header: &Header,
        ancestry: &mut dyn Iterator<Item = ExtendedHeader>,
    ) -> Vec<AncestryAction> {
        let finalized = self.build_finality(
            header,
            &mut ancestry.take_while(|e| !e.is_finalized).map(|e| e.header),
        );

        if !finalized.is_empty() {
            debug!(target: "finality", "Finalizing blocks: {:?}", finalized);
        }

        finalized
            .into_iter()
            .map(AncestryAction::MarkFinalized)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::{calculate_score, AuthorityRound, AuthorityRoundParams};
    use accounts::AccountProvider;
    use block::*;
    use engines::{validator_set::TestSet, Engine, Seal};
    use ethereum_types::{H520, U256};
    use hash::keccak;
    use rlp::encode;
    use spec::Spec;
    use std::sync::{
        atomic::{AtomicUsize, Ordering as AtomicOrdering},
        Arc,
    };
    use test_helpers::get_temp_state_db;
    use types::header::Header;

    fn aura<F>(f: F) -> Arc<AuthorityRound>
    where
        F: FnOnce(&mut AuthorityRoundParams),
    {
        let mut params = AuthorityRoundParams {
            step_duration: 1,
            start_step: Some(1),
            validators: Box::new(TestSet::default()),
            validate_score_transition: 0,
            validate_step_transition: 0,
            immediate_transitions: true,
            maximum_uncle_count_transition: 0,
            maximum_uncle_count: 0,
            block_reward: Default::default(),
            block_reward_contract_transition: 0,
            block_reward_contract: Default::default(),
        };

        // mutate aura params
        f(&mut params);

        // create engine
        let mut c_params = ::spec::CommonParams::default();
        c_params.gas_limit_bound_divisor = 5.into();
        let machine = ::machine::EthereumMachine::regular(c_params, Default::default());
        AuthorityRound::new(params, machine).unwrap()
    }

    #[test]
    fn has_valid_metadata() {
        let engine = Spec::new_test_round().engine;
        assert!(!engine.name().is_empty());
    }

    #[test]
    fn can_return_schedule() {
        let engine = Spec::new_test_round().engine;
        let schedule = engine.schedule(10000000);

        assert!(schedule.stack_limit > 0);
    }

    #[test]
    fn can_do_signature_verification_fail() {
        let engine = Spec::new_test_round().engine;
        let mut header: Header = Header::default();
        header.set_seal(vec![encode(&H520::default())]);

        let verify_result = engine.verify_block_external(&header);
        assert!(verify_result.is_err());
    }

    #[test]
    fn generates_seal_and_does_not_double_propose() {
        let tap = Arc::new(AccountProvider::transient_provider());
        let addr1 = tap.insert_account(keccak("1").into(), &"1".into()).unwrap();
        let addr2 = tap.insert_account(keccak("2").into(), &"2".into()).unwrap();

        let spec = Spec::new_test_round();
        let engine = &*spec.engine;
        let genesis_header = spec.genesis_header();
        let db1 = spec
            .ensure_db_good(get_temp_state_db(), &Default::default())
            .unwrap();
        let db2 = spec
            .ensure_db_good(get_temp_state_db(), &Default::default())
            .unwrap();
        let last_hashes = Arc::new(vec![genesis_header.hash()]);
        let b1 = OpenBlock::new(
            engine,
            Default::default(),
            false,
            db1,
            &genesis_header,
            last_hashes.clone(),
            addr1,
            (3141562.into(), 31415620.into()),
            vec![],
            false,
            None,
        )
        .unwrap();
        let b1 = b1.close_and_lock().unwrap();
        let b2 = OpenBlock::new(
            engine,
            Default::default(),
            false,
            db2,
            &genesis_header,
            last_hashes,
            addr2,
            (3141562.into(), 31415620.into()),
            vec![],
            false,
            None,
        )
        .unwrap();
        let b2 = b2.close_and_lock().unwrap();

        engine.set_signer(Box::new((tap.clone(), addr1, "1".into())));
        if let Seal::Regular(seal) = engine.generate_seal(&b1, &genesis_header) {
            assert!(b1.clone().try_seal(engine, seal).is_ok());
            // Second proposal is forbidden.
            assert!(engine.generate_seal(&b1, &genesis_header) == Seal::None);
        }

        engine.set_signer(Box::new((tap, addr2, "2".into())));
        if let Seal::Regular(seal) = engine.generate_seal(&b2, &genesis_header) {
            assert!(b2.clone().try_seal(engine, seal).is_ok());
            // Second proposal is forbidden.
            assert!(engine.generate_seal(&b2, &genesis_header) == Seal::None);
        }
    }

    #[test]
    fn checks_difficulty_in_generate_seal() {
        let tap = Arc::new(AccountProvider::transient_provider());
        let addr1 = tap.insert_account(keccak("1").into(), &"1".into()).unwrap();
        let addr2 = tap.insert_account(keccak("0").into(), &"0".into()).unwrap();

        let spec = Spec::new_test_round();
        let engine = &*spec.engine;

        let genesis_header = spec.genesis_header();
        let db1 = spec
            .ensure_db_good(get_temp_state_db(), &Default::default())
            .unwrap();
        let db2 = spec
            .ensure_db_good(get_temp_state_db(), &Default::default())
            .unwrap();
        let last_hashes = Arc::new(vec![genesis_header.hash()]);

        let b1 = OpenBlock::new(
            engine,
            Default::default(),
            false,
            db1,
            &genesis_header,
            last_hashes.clone(),
            addr1,
            (3141562.into(), 31415620.into()),
            vec![],
            false,
            None,
        )
        .unwrap();
        let b1 = b1.close_and_lock().unwrap();
        let b2 = OpenBlock::new(
            engine,
            Default::default(),
            false,
            db2,
            &genesis_header,
            last_hashes,
            addr2,
            (3141562.into(), 31415620.into()),
            vec![],
            false,
            None,
        )
        .unwrap();
        let b2 = b2.close_and_lock().unwrap();

        engine.set_signer(Box::new((tap.clone(), addr1, "1".into())));
        match engine.generate_seal(&b1, &genesis_header) {
            Seal::None => panic!("wrong seal"),
            Seal::Regular(_) => {
                engine.step();

                engine.set_signer(Box::new((tap.clone(), addr2, "0".into())));
                match engine.generate_seal(&b2, &genesis_header) {
                    Seal::Regular(_) => panic!("sealed despite wrong difficulty"),
                    Seal::None => {}
                }
            }
        }
    }

    #[test]
    fn proposer_switching() {
        let tap = AccountProvider::transient_provider();
        let addr = tap.insert_account(keccak("0").into(), &"0".into()).unwrap();
        let mut parent_header: Header = Header::default();
        parent_header.set_seal(vec![encode(&0usize)]);
        parent_header.set_gas_limit("222222".parse::<U256>().unwrap());
        let mut header: Header = Header::default();
        header.set_number(1);
        header.set_gas_limit("222222".parse::<U256>().unwrap());
        header.set_author(addr);

        let engine = Spec::new_test_round().engine;

        // Two validators.
        // Spec starts with step 2.
        header.set_difficulty(calculate_score(0, 2));
        let signature = tap
            .sign(addr, Some("0".into()), header.bare_hash())
            .unwrap();
        header.set_seal(vec![encode(&2usize), encode(&(&*signature as &[u8]))]);
        assert!(engine.verify_block_family(&header, &parent_header).is_ok());
        assert!(engine.verify_block_external(&header).is_err());
        header.set_difficulty(calculate_score(0, 1));
        let signature = tap
            .sign(addr, Some("0".into()), header.bare_hash())
            .unwrap();
        header.set_seal(vec![encode(&1usize), encode(&(&*signature as &[u8]))]);
        assert!(engine.verify_block_family(&header, &parent_header).is_ok());
        assert!(engine.verify_block_external(&header).is_ok());
    }

    #[test]
    fn rejects_future_block() {
        let tap = AccountProvider::transient_provider();
        let addr = tap.insert_account(keccak("0").into(), &"0".into()).unwrap();

        let mut parent_header: Header = Header::default();
        parent_header.set_seal(vec![encode(&0usize)]);
        parent_header.set_gas_limit("222222".parse::<U256>().unwrap());
        let mut header: Header = Header::default();
        header.set_number(1);
        header.set_gas_limit("222222".parse::<U256>().unwrap());
        header.set_author(addr);

        let engine = Spec::new_test_round().engine;

        // Two validators.
        // Spec starts with step 2.
        header.set_difficulty(calculate_score(0, 1));
        let signature = tap
            .sign(addr, Some("0".into()), header.bare_hash())
            .unwrap();
        header.set_seal(vec![encode(&1usize), encode(&(&*signature as &[u8]))]);
        assert!(engine.verify_block_family(&header, &parent_header).is_ok());
        assert!(engine.verify_block_external(&header).is_ok());
        header.set_seal(vec![encode(&5usize), encode(&(&*signature as &[u8]))]);
        assert!(engine.verify_block_basic(&header).is_err());
    }

    #[test]
    fn rejects_step_backwards() {
        let tap = AccountProvider::transient_provider();
        let addr = tap.insert_account(keccak("0").into(), &"0".into()).unwrap();

        let mut parent_header: Header = Header::default();
        parent_header.set_seal(vec![encode(&4usize)]);
        parent_header.set_gas_limit("222222".parse::<U256>().unwrap());
        let mut header: Header = Header::default();
        header.set_number(1);
        header.set_gas_limit("222222".parse::<U256>().unwrap());
        header.set_author(addr);

        let engine = Spec::new_test_round().engine;

        let signature = tap
            .sign(addr, Some("0".into()), header.bare_hash())
            .unwrap();
        // Two validators.
        // Spec starts with step 2.
        header.set_seal(vec![encode(&5usize), encode(&(&*signature as &[u8]))]);
        header.set_difficulty(calculate_score(4, 5));
        assert!(engine.verify_block_family(&header, &parent_header).is_ok());
        header.set_seal(vec![encode(&3usize), encode(&(&*signature as &[u8]))]);
        header.set_difficulty(calculate_score(4, 3));
        assert!(engine.verify_block_family(&header, &parent_header).is_err());
    }

    #[test]
    fn reports_skipped() {
        let last_benign = Arc::new(AtomicUsize::new(0));
        let aura = aura(|p| {
            p.validators = Box::new(TestSet::new(Default::default(), last_benign.clone()));
        });

        let mut parent_header: Header = Header::default();
        parent_header.set_seal(vec![encode(&1usize)]);
        parent_header.set_gas_limit("222222".parse::<U256>().unwrap());
        let mut header: Header = Header::default();
        header.set_difficulty(calculate_score(1, 3));
        header.set_gas_limit("222222".parse::<U256>().unwrap());
        header.set_seal(vec![encode(&3usize)]);

        // Do not report when signer not present.
        assert!(aura.verify_block_family(&header, &parent_header).is_ok());
        assert_eq!(last_benign.load(AtomicOrdering::SeqCst), 0);

        aura.set_signer(Box::new((
            Arc::new(AccountProvider::transient_provider()),
            Default::default(),
            "".into(),
        )));

        // Do not report on steps skipped between genesis and first block.
        header.set_number(1);
        assert!(aura.verify_block_family(&header, &parent_header).is_ok());
        assert_eq!(last_benign.load(AtomicOrdering::SeqCst), 0);

        // Report on skipped steps otherwise.
        header.set_number(2);
        assert!(aura.verify_block_family(&header, &parent_header).is_ok());
        assert_eq!(last_benign.load(AtomicOrdering::SeqCst), 2);
    }

    #[test]
    fn test_uncles_transition() {
        let aura = aura(|params| {
            params.maximum_uncle_count_transition = 1;
        });

        assert_eq!(aura.maximum_uncle_count(0), 2);
        assert_eq!(aura.maximum_uncle_count(1), 0);
        assert_eq!(aura.maximum_uncle_count(100), 0);
    }

    #[test]
    #[should_panic(expected = "counter is too high")]
    fn test_counter_increment_too_high() {
        use super::Step;
        let step = Step {
            calibrate: false,
            inner: AtomicUsize::new(::std::usize::MAX),
            duration: 1,
        };
        step.increment();
    }

    #[test]
    #[should_panic(expected = "counter is too high")]
    fn test_counter_duration_remaining_too_high() {
        use super::Step;
        let step = Step {
            calibrate: false,
            inner: AtomicUsize::new(::std::usize::MAX),
            duration: 1,
        };
        step.duration_remaining();
    }

    #[test]
    #[should_panic(expected = "authority_round: step duration can't be zero")]
    fn test_step_duration_zero() {
        aura(|params| {
            params.step_duration = 0;
        });
    }
}
