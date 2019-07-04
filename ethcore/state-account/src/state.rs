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

//! A mutable state representation suitable to execute transactions.
//! Generic over a `Backend`. Deals with `Account`s.
//! Unconfirmed sub-states are managed with `checkpoint`s which may be canonicalized
//! or rolled back.

use std::{
	cell::{RefCell, RefMut},
	collections::{BTreeMap, BTreeSet, HashMap, HashSet},
	collections::hash_map::Entry,
	fmt,
	sync::Arc,
	error, io::Error as IoError,
};

use ethereum_types::{Address, H256, U256};
use ethtrie::TrieError;
use factories::{Factories, VmFactory};
use hash_db::{AsHashDB, HashDB};
use keccak_hash::{KECCAK_EMPTY, KECCAK_NULL_RLP};
use kvdb::DBValue;
use log::{warn, trace};
use parity_bytes::Bytes;
use trie_db::{Recorder, Trie};
use common_types::{
	basic_account::BasicAccount,
	receipt::{Receipt, TransactionOutcome},
	state_diff::StateDiff,
	transaction::SignedTransaction
};
use ethtrie::{Result as TrieResult, TrieDB};
use keccak_hasher::KeccakHasher;
use pod_account::*;
use rlp::DecoderError;
use vm::EnvInfo;
use derive_more::{Display, From};

use crate::{
	account::Account,
	backend::Backend,
};

/// Used to return information about an `State::apply` operation.
pub struct ApplyOutcome<T, V> {
	/// The receipt for the applied transaction.
	pub receipt: Receipt,
	/// The output of the applied transaction.
	pub output: Bytes,
	/// The trace for the applied transaction, empty if tracing was not produced.
	pub trace: Vec<T>,
	/// The VM trace for the applied transaction, None if tracing was not produced.
	pub vm_trace: Option<V>
}

// TODO: sort out error handling
#[derive(Debug, Display, From)]
pub enum Error {
	/// Trie error.
	Trie(TrieError),
	/// Decoder error.
	Decoder(DecoderError),
	/// Io error.
	Io(IoError), // TODO: maybe not needed?
}

// TODO: needed?
impl error::Error for Error {}

impl<E> From<Box<E>> for Error where Error: From<E> {
	fn from(err: Box<E>) -> Self {
		Error::from(*err)
	}
}

/// Result type for the execution ("application") of a transaction.
pub type ApplyResult<T, V> = Result<ApplyOutcome<T, V>, Error>;

// TODO: needs ExecutionError and Executed
///// Return type of proof validity check.
//#[derive(Debug, Clone)]
//pub enum ProvedExecution {
//	/// Proof wasn't enough to complete execution.
//	BadProof,
//	/// The transaction failed, but not due to a bad proof.
//	Failed(ExecutionError),
//	/// The transaction successfully completed with the given proof.
//	Complete(Box<Executed>),
//}

#[derive(Eq, PartialEq, Clone, Copy, Debug)]
/// Account modification state. Used to check if the account was
/// Modified in between commits and overall.
enum AccountState {
	/// Account was loaded from disk and never modified in this state object.
	CleanFresh,
	/// Account was loaded from the global cache and never modified.
	CleanCached,
	/// Account has been modified and is not committed to the trie yet.
	/// This is set if any of the account data is changed, including
	/// storage and code.
	Dirty,
	/// Account was modified and committed to the trie.
	Committed,
}

#[derive(Debug)]
/// In-memory copy of the account data. Holds the optional account
/// and the modification status.
/// Account entry can contain existing (`Some`) or non-existing
/// account (`None`)
struct AccountEntry {
	/// Account entry. `None` if account known to be non-existent.
	account: Option<Account>,
	/// Unmodified account balance.
	old_balance: Option<U256>,
	/// Entry state.
	state: AccountState,
}

// Account cache item. Contains account data and
// modification state
impl AccountEntry {
	fn is_dirty(&self) -> bool {
		self.state == AccountState::Dirty
	}

	fn exists_and_is_null(&self) -> bool {
		self.account.as_ref().map_or(false, |a| a.is_null())
	}

	/// Clone dirty data into new `AccountEntry`. This includes
	/// basic account data and modified storage keys.
	/// Returns None if clean.
	fn clone_if_dirty(&self) -> Option<AccountEntry> {
		match self.is_dirty() {
			true => Some(self.clone_dirty()),
			false => None,
		}
	}

	/// Clone dirty data into new `AccountEntry`. This includes
	/// basic account data and modified storage keys.
	fn clone_dirty(&self) -> AccountEntry {
		AccountEntry {
			old_balance: self.old_balance,
			account: self.account.as_ref().map(Account::clone_dirty),
			state: self.state,
		}
	}

	// Create a new account entry and mark it as dirty.
	fn new_dirty(account: Option<Account>) -> AccountEntry {
		AccountEntry {
			old_balance: account.as_ref().map(|a| a.balance().clone()),
			account,
			state: AccountState::Dirty,
		}
	}

	// Create a new account entry and mark it as clean.
	fn new_clean(account: Option<Account>) -> AccountEntry {
		AccountEntry {
			old_balance: account.as_ref().map(|a| a.balance().clone()),
			account,
			state: AccountState::CleanFresh,
		}
	}

	// Create a new account entry and mark it as clean and cached.
	fn new_clean_cached(account: Option<Account>) -> AccountEntry {
		AccountEntry {
			old_balance: account.as_ref().map(|a| a.balance().clone()),
			account,
			state: AccountState::CleanCached,
		}
	}

	// Replace data with another entry but preserve storage cache.
	fn overwrite_with(&mut self, other: AccountEntry) {
		self.state = other.state;
		match other.account {
			Some(acc) => {
				if let Some(ref mut ours) = self.account {
					ours.overwrite_with(acc);
				} else {
					self.account = Some(acc);
				}
			},
			None => self.account = None,
		}
	}
}

// TODO: needs Machine
///// Check the given proof of execution.
///// `Err(ExecutionError::Internal)` indicates failure, everything else indicates
///// a successful proof (as the transaction itself may be poorly chosen).
//pub fn check_proof(
//	proof: &[DBValue],
//	root: H256,
//	transaction: &SignedTransaction,
//	machine: &Machine,
//	env_info: &EnvInfo,
//) -> ProvedExecution

// TODO: needs Machine
///// Prove a `virtual` transaction on the given state.
///// Returns `None` when the transacion could not be proved,
///// and a proof otherwise.
//pub fn prove_transaction_virtual<H: AsHashDB<KeccakHasher, DBValue> + Send + Sync>(
//	db: H,
//	root: H256,
//	transaction: &SignedTransaction,
//	machine: &Machine,
//	env_info: &EnvInfo,
//	factories: Factories,
//) -> Option<(Bytes, Vec<DBValue>)> {


/// Representation of the entire state of all accounts in the system.
///
/// `State` can work together with `StateDB` to share account cache.
///
/// Local cache contains changes made locally and changes accumulated
/// locally from previous commits. Global cache reflects the database
/// state and never contains any changes.
///
/// Cache items contains account data, or the flag that account does not exist
/// and modification state (see `AccountState`)
///
/// Account data can be in the following cache states:
/// * In global but not local - something that was queried from the database,
/// but never modified
/// * In local but not global - something that was just added (e.g. new account)
/// * In both with the same value - something that was changed to a new value,
/// but changed back to a previous block in the same block (same State instance)
/// * In both with different values - something that was overwritten with a
/// new value.
///
/// All read-only state queries check local cache/modifications first,
/// then global state cache. If data is not found in any of the caches
/// it is loaded from the DB to the local cache.
///
/// **** IMPORTANT *************************************************************
/// All the modifications to the account data must set the `Dirty` state in the
/// `AccountEntry`. This is done in `require` and `require_or_from`. So just
/// use that.
/// ****************************************************************************
///
/// Upon destruction all the local cache data propagated into the global cache.
/// Propagated items might be rejected if current state is non-canonical.
///
/// State checkpointing.
///
/// A new checkpoint can be created with `checkpoint()`. checkpoints can be
/// created in a hierarchy.
/// When a checkpoint is active all changes are applied directly into
/// `cache` and the original value is copied into an active checkpoint.
/// Reverting a checkpoint with `revert_to_checkpoint` involves copying
/// original values from the latest checkpoint back into `cache`. The code
/// takes care not to overwrite cached storage while doing that.
/// A checkpoint can be discarded with `discard_checkpoint`. All of the original
/// backed-up values are moved into a parent checkpoint (if any).
///
pub struct State<B> {
	db: B,
	root: H256,
	cache: RefCell<HashMap<Address, AccountEntry>>,
	// The original account is preserved in
	checkpoints: RefCell<Vec<HashMap<Address, Option<AccountEntry>>>>,
	account_start_nonce: U256,
	factories: Factories,
}

#[derive(Copy, Clone)]
enum RequireCache {
	None,
	CodeSize,
	Code,
}

/// Mode of dealing with null accounts.
#[derive(PartialEq)]
pub enum CleanupMode<'a> {
	/// Create accounts which would be null.
	ForceCreate,
	/// Don't delete null accounts upon touching, but also don't create them.
	NoEmpty,
	/// Mark all touched accounts.
	TrackTouched(&'a mut HashSet<Address>),
}

/// Provides subset of `State` methods to query state information
pub trait StateInfo {
	/// Get the nonce of account `a`.
	fn nonce(&self, a: &Address) -> TrieResult<U256>;

	/// Get the balance of account `a`.
	fn balance(&self, a: &Address) -> TrieResult<U256>;

	/// Mutate storage of account `address` so that it is `value` for `key`.
	fn storage_at(&self, address: &Address, key: &H256) -> TrieResult<H256>;

	/// Get accounts' code.
	fn code(&self, a: &Address) -> TrieResult<Option<Arc<Bytes>>>;
}

impl<B: Backend> StateInfo for State<B> {
	fn nonce(&self, a: &Address) -> TrieResult<U256> { State::nonce(self, a) }
	fn balance(&self, a: &Address) -> TrieResult<U256> { State::balance(self, a) }
	fn storage_at(&self, address: &Address, key: &H256) -> TrieResult<H256> { State::storage_at(self, address, key) }
	fn code(&self, address: &Address) -> TrieResult<Option<Arc<Bytes>>> { State::code(self, address) }
}

const SEC_TRIE_DB_UNWRAP_STR: &'static str = "A state can only be created with valid root. Creating a SecTrieDB with a valid root will not fail. \
			 Therefore creating a SecTrieDB with this state's root will not fail.";

impl<B: Backend> State<B> {
	/// Creates new state with empty state root
	/// Used for tests.
	pub fn new(mut db: B, account_start_nonce: U256, factories: Factories) -> State<B> {
		let mut root = H256::zero();
		{
			// init trie and reset root to null
			let _ = factories.trie.create(db.as_hash_db_mut(), &mut root);
		}

		State {
			db,
			root,
			cache: RefCell::new(HashMap::new()),
			checkpoints: RefCell::new(Vec::new()),
			account_start_nonce,
			factories,
		}
	}

	/// Creates new state with existing state root
	pub fn from_existing(db: B, root: H256, account_start_nonce: U256, factories: Factories) -> TrieResult<State<B>> {
		if !db.as_hash_db().contains(&root, hash_db::EMPTY_PREFIX) {
			return Err(Box::new(TrieError::InvalidStateRoot(root)));
		}

		let state = State {
			db,
			root,
			cache: RefCell::new(HashMap::new()),
			checkpoints: RefCell::new(Vec::new()),
			account_start_nonce,
			factories,
		};

		Ok(state)
	}

	/// Get a VM factory that can execute on this state.
	pub fn vm_factory(&self) -> VmFactory {
		self.factories.vm.clone()
	}

	/// Create a recoverable checkpoint of this state. Return the checkpoint index.
	pub fn checkpoint(&mut self) -> usize {
		let checkpoints = self.checkpoints.get_mut();
		let index = checkpoints.len();
		checkpoints.push(HashMap::new());
		index
	}

	/// Merge last checkpoint with previous.
	pub fn discard_checkpoint(&mut self) {
		// merge with previous checkpoint
		let last = self.checkpoints.get_mut().pop();
		if let Some(mut checkpoint) = last {
			if let Some(ref mut prev) = self.checkpoints.get_mut().last_mut() {
				if prev.is_empty() {
					**prev = checkpoint;
				} else {
					for (k, v) in checkpoint.drain() {
						prev.entry(k).or_insert(v);
					}
				}
			}
		}
	}

	/// Revert to the last checkpoint and discard it.
	pub fn revert_to_checkpoint(&mut self) {
		if let Some(mut checkpoint) = self.checkpoints.get_mut().pop() {
			for (k, v) in checkpoint.drain() {
				match v {
					Some(v) => {
						match self.cache.get_mut().entry(k) {
							Entry::Occupied(mut e) => {
								// Merge checkpointed changes back into the main account
								// storage preserving the cache.
								e.get_mut().overwrite_with(v);
							},
							Entry::Vacant(e) => {
								e.insert(v);
							}
						}
					},
					None => {
						if let Entry::Occupied(e) = self.cache.get_mut().entry(k) {
							if e.get().is_dirty() {
								e.remove();
							}
						}
					}
				}
			}
		}
	}

	fn insert_cache(&self, address: &Address, account: AccountEntry) {
		// Dirty account which is not in the cache means this is a new account.
		// It goes directly into the checkpoint as there's nothing to rever to.
		//
		// In all other cases account is read as clean first, and after that made
		// dirty in and added to the checkpoint with `note_cache`.
		let is_dirty = account.is_dirty();
		let old_value = self.cache.borrow_mut().insert(*address, account);
		if is_dirty {
			if let Some(ref mut checkpoint) = self.checkpoints.borrow_mut().last_mut() {
				checkpoint.entry(*address).or_insert(old_value);
			}
		}
	}

	fn note_cache(&self, address: &Address) {
		if let Some(ref mut checkpoint) = self.checkpoints.borrow_mut().last_mut() {
			checkpoint.entry(*address)
				.or_insert_with(|| self.cache.borrow().get(address).map(AccountEntry::clone_dirty));
		}
	}

	/// Destroy the current object and return root and database.
	pub fn drop(mut self) -> (H256, B) {
		self.propagate_to_global_cache();
		(self.root, self.db)
	}

	/// Destroy the current object and return single account data.
	pub fn into_account(self, account: &Address) -> TrieResult<(Option<Arc<Bytes>>, HashMap<H256, H256>)> {
		// TODO: deconstruct without cloning.
		let account = self.require(account, true)?;
		Ok((account.code().clone(), account.storage_changes().clone()))
	}

	/// Return reference to root
	pub fn root(&self) -> &H256 {
		&self.root
	}

	/// Create a new contract at address `contract`. If there is already an account at the address
	/// it will have its code reset, ready for `init_code()`.
	pub fn new_contract(&mut self, contract: &Address, balance: U256, nonce_offset: U256) -> TrieResult<()> {
		let original_storage_root = self.original_storage_root(contract)?;
		let (nonce, overflow) = self.account_start_nonce.overflowing_add(nonce_offset);
		if overflow {
			return Err(Box::new(TrieError::DecoderError(H256::from(*contract),
			                                            rlp::DecoderError::Custom("Nonce overflow".into()))));
		}
		self.insert_cache(contract, AccountEntry::new_dirty(Some(Account::new_contract(balance, nonce, original_storage_root))));
		Ok(())
	}

	/// Remove an existing account.
	pub fn kill_account(&mut self, account: &Address) {
		self.insert_cache(account, AccountEntry::new_dirty(None));
	}

	/// Determine whether an account exists.
	pub fn exists(&self, a: &Address) -> TrieResult<bool> {
		// Bloom filter does not contain empty accounts, so it is important here to
		// check if account exists in the database directly before EIP-161 is in effect.
		self.ensure_cached(a, RequireCache::None, false, |a| a.is_some())
	}

	/// Determine whether an account exists and if not empty.
	pub fn exists_and_not_null(&self, a: &Address) -> TrieResult<bool> {
		self.ensure_cached(a, RequireCache::None, false, |a| a.map_or(false, |a| !a.is_null()))
	}

	/// Determine whether an account exists and has code or non-zero nonce.
	pub fn exists_and_has_code_or_nonce(&self, a: &Address) -> TrieResult<bool> {
		self.ensure_cached(a, RequireCache::CodeSize, false,
		                   |a| a.map_or(false, |a| a.code_hash() != KECCAK_EMPTY || *a.nonce() != self.account_start_nonce))
	}

	/// Get the balance of account `a`.
	pub fn balance(&self, a: &Address) -> TrieResult<U256> {
		self.ensure_cached(a, RequireCache::None, true,
		                   |a| a.as_ref().map_or(U256::zero(), |account| *account.balance()))
	}

	/// Get the nonce of account `a`.
	pub fn nonce(&self, a: &Address) -> TrieResult<U256> {
		self.ensure_cached(a, RequireCache::None, true,
		                   |a| a.as_ref().map_or(self.account_start_nonce, |account| *account.nonce()))
	}

	/// Whether the base storage root of an account remains unchanged.
	pub fn is_base_storage_root_unchanged(&self, a: &Address) -> TrieResult<bool> {
		Ok(self.ensure_cached(a, RequireCache::None, true,
		                      |a| a.as_ref().map(|account| account.is_base_storage_root_unchanged()))?
			.unwrap_or(true))
	}

	/// Get the storage root of account `a`.
	pub fn storage_root(&self, a: &Address) -> TrieResult<Option<H256>> {
		self.ensure_cached(a, RequireCache::None, true,
		                   |a| a.as_ref().and_then(|account| account.storage_root()))
	}

	/// Get the original storage root since last commit of account `a`.
	pub fn original_storage_root(&self, a: &Address) -> TrieResult<H256> {
		Ok(self.ensure_cached(a, RequireCache::None, true,
		                      |a| a.as_ref().map(|account| account.original_storage_root()))?
			.unwrap_or(KECCAK_NULL_RLP))
	}

	/// Initialise the code of account `a` so that it is `code`.
	/// NOTE: Account should have been created with `new_contract`.
	pub fn init_code(&mut self, a: &Address, code: Bytes) -> TrieResult<()> {
		self.require_or_from(a, true, || Account::new_contract(0.into(), self.account_start_nonce, KECCAK_NULL_RLP), |_| {})?.init_code(code);
		Ok(())
	}

	/// Reset the code of account `a` so that it is `code`.
	pub fn reset_code(&mut self, a: &Address, code: Bytes) -> TrieResult<()> {
		self.require_or_from(a, true, || Account::new_contract(0.into(), self.account_start_nonce, KECCAK_NULL_RLP), |_| {})?.reset_code(code);
		Ok(())
	}

	// TODO: Needs Machine and TransactionOptions
	/// Execute a given transaction, producing a receipt and an optional trace.
	/// This will change the state accordingly.
//	pub fn apply(&mut self, env_info: &EnvInfo, machine: &Machine, t: &SignedTransaction, tracing: bool) -> ApplyResult<FlatTrace, VMTrace> {
//		if tracing {
//			let options = TransactOptions::with_tracing();
//			self.apply_with_tracing(env_info, machine, t, options.tracer, options.vm_tracer)
//		} else {
//			let options = TransactOptions::with_no_tracing();
//			self.apply_with_tracing(env_info, machine, t, options.tracer, options.vm_tracer)
//		}
//	}

	// TODO: Needs Machine
	/// Execute a given transaction with given tracer and VM tracer producing a receipt and an optional trace.
	/// This will change the state accordingly.
//	pub fn apply_with_tracing<V, T>(
//		&mut self,
//		env_info: &EnvInfo,
//		machine: &Machine,
//		t: &SignedTransaction,
//		tracer: T,
//		vm_tracer: V,
//	) -> ApplyResult<T::Output, V::Output> where
//		T: trace::Tracer,
//		V: trace::VMTracer,
//	{
//		let options = TransactOptions::new(tracer, vm_tracer);
//		let e = self.execute(env_info, machine, t, options, false)?;
//		let params = machine.params();
//
//		let eip658 = env_info.number >= params.eip658_transition;
//		let no_intermediate_commits =
//			eip658 ||
//				(env_info.number >= params.eip98_transition && env_info.number >= params.validate_receipts_transition);
//
//		let outcome = if no_intermediate_commits {
//			if eip658 {
//				TransactionOutcome::StatusCode(if e.exception.is_some() { 0 } else { 1 })
//			} else {
//				TransactionOutcome::Unknown
//			}
//		} else {
//			self.commit()?;
//			TransactionOutcome::StateRoot(self.root().clone())
//		};
//
//		let output = e.output;
//		let receipt = Receipt::new(outcome, e.cumulative_gas_used, e.logs);
//		trace!(target: "state", "Transaction receipt: {:?}", receipt);
//
//		Ok(ApplyOutcome {
//			receipt,
//			output,
//			trace: e.trace,
//			vm_trace: e.vm_trace,
//		})
//	}

	// TODO: Needs Machine and Executed, ExecutionError, Executive
	// Execute a given transaction without committing changes.
	//
	// `virt` signals that we are executing outside of a block set and restrictions like
	// gas limits and gas costs should be lifted.
//	fn execute<T, V>(&mut self, env_info: &EnvInfo, machine: &Machine, t: &SignedTransaction, options: TransactOptions<T, V>, virt: bool)
//	                 -> Result<Executed<T::Output, V::Output>, ExecutionError> where T: trace::Tracer, V: trace::VMTracer,
//	{
//		let schedule = machine.schedule(env_info.number);
//		let mut e = Executive::new(self, env_info, machine, &schedule);
//
//		match virt {
//			true => e.transact_virtual(t, options),
//			false => e.transact(t, options),
//		}
//	}

	fn touch(&mut self, a: &Address) -> TrieResult<()> {
		self.require(a, false)?;
		Ok(())
	}

	/// Commits our cached account changes into the trie.
	pub fn commit(&mut self) -> Result<(), Error> {
		assert!(self.checkpoints.borrow().is_empty());
		// first, commit the sub trees.
		let mut accounts = self.cache.borrow_mut();
		for (address, ref mut a) in accounts.iter_mut().filter(|&(_, ref a)| a.is_dirty()) {
			if let Some(ref mut account) = a.account {
				let addr_hash = account.address_hash(address);
				{
					let mut account_db = self.factories.accountdb.create(self.db.as_hash_db_mut(), addr_hash);
					account.commit_storage(&self.factories.trie, account_db.as_hash_db_mut())?;
					account.commit_code(account_db.as_hash_db_mut());
				}
				if !account.is_empty() {
					self.db.note_non_null_account(address);
				}
			}
		}

		{
			let mut trie = self.factories.trie.from_existing(self.db.as_hash_db_mut(), &mut self.root)?;
			for (address, ref mut a) in accounts.iter_mut().filter(|&(_, ref a)| a.is_dirty()) {
				a.state = AccountState::Committed;
				match a.account {
					Some(ref mut account) => {
						trie.insert(address.as_bytes(), &account.rlp())?;
					},
					None => {
						trie.remove(address.as_bytes())?;
					},
				};
			}
		}

		Ok(())
	}

	/// Propagate local cache into shared canonical state cache.
	fn propagate_to_global_cache(&mut self) {
		let mut addresses = self.cache.borrow_mut();
		trace!("Committing cache {:?} entries", addresses.len());
		for (address, a) in addresses.drain().filter(|&(_, ref a)| a.state == AccountState::Committed || a.state == AccountState::CleanFresh) {
			self.db.add_to_account_cache(address, a.account, a.state == AccountState::Committed);
		}
	}

	/// Clear state cache
	pub fn clear(&mut self) {
		assert!(self.checkpoints.borrow().is_empty());
		self.cache.borrow_mut().clear();
	}

	/// Remove any touched empty or dust accounts.
	pub fn kill_garbage(&mut self, touched: &HashSet<Address>, remove_empty_touched: bool, min_balance: &Option<U256>, kill_contracts: bool) -> TrieResult<()> {
		let to_kill: HashSet<_> = {
			self.cache.borrow().iter().filter_map(|(address, ref entry)|
				if touched.contains(address) && // Check all touched accounts
					((remove_empty_touched && entry.exists_and_is_null()) // Remove all empty touched accounts.
						|| min_balance.map_or(false, |ref balance| entry.account.as_ref().map_or(false, |account|
						(account.is_basic() || kill_contracts) // Remove all basic and optionally contract accounts where balance has been decreased.
							&& account.balance() < balance && entry.old_balance.as_ref().map_or(false, |b| account.balance() < b)))) {

					Some(address.clone())
				} else { None }).collect()
		};
		for address in to_kill {
			self.kill_account(&address);
		}
		Ok(())
	}

	// TODO: Needs PodState
	/// Populate the state from `accounts`.
	/// Used for tests.
//	pub fn populate_from(&mut self, accounts: PodState) {
//		assert!(self.checkpoints.borrow().is_empty());
//		for (add, acc) in accounts.drain().into_iter() {
//			self.cache.borrow_mut().insert(add, AccountEntry::new_dirty(Some(Account::from_pod(acc))));
//		}
//	}
//
//	/// Populate a PodAccount map from this state.
//	fn to_pod_cache(&self) -> PodState {
//		assert!(self.checkpoints.borrow().is_empty());
//		PodState::from(self.cache.borrow().iter().fold(BTreeMap::new(), |mut m, (add, opt)| {
//			if let Some(ref acc) = opt.account {
//				m.insert(*add, acc.to_pod());
//			}
//			m
//		}))
//	}

	// TODO: sort out features
	#[cfg(feature="to-pod-full")]
	/// Populate a PodAccount map from this state.
	/// Warning this is not for real time use.
	/// Use of this method requires FatDB mode to be able
	/// to iterate on accounts.
	pub fn to_pod_full(&self) -> Result<PodState, Error> {

		assert!(self.checkpoints.borrow().is_empty());
		assert!(self.factories.trie.is_fat());

		let mut result = BTreeMap::new();

		let db = &self.db.as_hash_db();
		let trie = self.factories.trie.readonly(db, &self.root)?;

		// put trie in cache
		for item in trie.iter()? {
			if let Ok((addr, _dbval)) = item {
				let address = Address::from_slice(&addr);
				let _ = self.require(&address, true);
			}
		}

		// Resolve missing part
		for (add, opt) in self.cache.borrow().iter() {
			if let Some(ref acc) = opt.account {
				let pod_account = self.account_to_pod_account(acc, add)?;
				result.insert(add.clone(), pod_account);
			}
		}

		Ok(PodState::from(result))
	}

	// TODO: sort out features
	/// Create a PodAccount from an account.
	/// Differs from existing method by including all storage
	/// values of the account to the PodAccount.
	/// This function is only intended for use in small tests or with fresh accounts.
	/// It requires FatDB.
	#[cfg(feature="to-pod-full")]
	fn account_to_pod_account(&self, account: &Account, address: &Address) -> Result<PodAccount, Error> {
		use ethereum_types::BigEndianHash;

		let mut pod_storage = BTreeMap::new();
		let addr_hash = account.address_hash(address);
		let accountdb = self.factories.accountdb.readonly(self.db.as_hash_db(), addr_hash);
		let root = account.base_storage_root();

		let accountdb = &accountdb.as_hash_db();
		let trie = self.factories.trie.readonly(accountdb, &root)?;
		for o_kv in trie.iter()? {
			if let Ok((key, val)) = o_kv {
				pod_storage.insert(
					H256::from_slice(&key[..]),
					BigEndianHash::from_uint(
						&rlp::decode::<U256>(&val[..]).expect("Decoded from trie which was encoded from the same type; qed")
					),
				);
			}
		}

		let mut pod_account = account.to_pod();
		// cached one first
		pod_storage.append(&mut pod_account.storage);
		pod_account.storage = pod_storage;
		Ok(pod_account)
	}

// TODO: Needs PodState
//	/// Populate a PodAccount map from this state, with another state as the account and storage query.
//	fn to_pod_diff<X: Backend>(&mut self, query: &State<X>) -> TrieResult<PodState> {
//		assert!(self.checkpoints.borrow().is_empty());
//
//		// Merge PodAccount::to_pod for cache of self and `query`.
//		let all_addresses = self.cache.borrow().keys().cloned()
//			.chain(query.cache.borrow().keys().cloned())
//			.collect::<BTreeSet<_>>();
//
//		Ok(PodState::from(all_addresses.into_iter().fold(Ok(BTreeMap::new()), |m: TrieResult<_>, address| {
//			let mut m = m?;
//
//			let account = self.ensure_cached(&address, RequireCache::Code, true, |acc| {
//				acc.map(|acc| {
//					// Merge all modified storage keys.
//					let all_keys = {
//						let self_keys = acc.storage_changes().keys().cloned()
//							.collect::<BTreeSet<_>>();
//
//						if let Some(ref query_storage) = query.cache.borrow().get(&address)
//							.and_then(|opt| {
//								Some(opt.account.as_ref()?.storage_changes().keys().cloned()
//									.collect::<BTreeSet<_>>())
//							})
//						{
//							self_keys.union(&query_storage).cloned().collect::<Vec<_>>()
//						} else {
//							self_keys.into_iter().collect::<Vec<_>>()
//						}
//					};
//
//					// Storage must be fetched after ensure_cached to avoid borrow problem.
//					(*acc.balance(), *acc.nonce(), all_keys, acc.code().map(|x| x.to_vec()))
//				})
//			})?;
//
//			if let Some((balance, nonce, storage_keys, code)) = account {
//				let storage = storage_keys.into_iter().fold(Ok(BTreeMap::new()), |s: TrieResult<_>, key| {
//					let mut s = s?;
//
//					s.insert(key, self.storage_at(&address, &key)?);
//					Ok(s)
//				})?;
//
//				m.insert(address, PodAccount {
//					balance, nonce, storage, code
//				});
//			}
//
//			Ok(m)
//		})?))
//	}

	// TODO: Needs PodState
//	/// Returns a `StateDiff` describing the difference from `orig` to `self`.
//	/// Consumes self.
//	pub fn diff_from<X: Backend>(&self, mut orig: State<X>) -> TrieResult<StateDiff> {
//		let pod_state_post = self.to_pod_cache();
//		let pod_state_pre = orig.to_pod_diff(self)?;
//		Ok(pod_state::diff_pod(&pod_state_pre, &pod_state_post))
//	}

	/// Load required account data from the databases. Returns whether the cache succeeds.
	#[must_use]
	fn update_account_cache(require: RequireCache, account: &mut Account, state_db: &B, db: &dyn HashDB<KeccakHasher, DBValue>) -> bool {
		if let RequireCache::None = require {
			return true;
		}

		if account.is_cached() {
			return true;
		}

		// if there's already code in the global cache, always cache it localy
		let hash = account.code_hash();
		match state_db.get_cached_code(&hash) {
			Some(code) => {
				account.cache_given_code(code);
				true
			},
			None => match require {
				RequireCache::None => true,
				RequireCache::Code => {
					if let Some(code) = account.cache_code(db) {
						// propagate code loaded from the database to
						// the global code cache.
						state_db.cache_code(hash, code);
						true
					} else {
						false
					}
				},
				RequireCache::CodeSize => {
					account.cache_code_size(db)
				}
			}
		}
	}

	/// Check caches for required data
	/// First searches for account in the local, then the shared cache.
	/// Populates local cache if nothing found.
	fn ensure_cached<F, U>(&self, a: &Address, require: RequireCache, check_null: bool, f: F) -> TrieResult<U>
		where F: Fn(Option<&Account>) -> U {
		// check local cache first
		if let Some(ref mut maybe_acc) = self.cache.borrow_mut().get_mut(a) {
			if let Some(ref mut account) = maybe_acc.account {
				let accountdb = self.factories.accountdb.readonly(self.db.as_hash_db(), account.address_hash(a));
				if Self::update_account_cache(require, account, &self.db, accountdb.as_hash_db()) {
					return Ok(f(Some(account)));
				} else {
					return Err(Box::new(TrieError::IncompleteDatabase(H256::from(*a))));
				}
			}
			return Ok(f(None));
		}
		// check global cache
		let result = self.db.get_cached(a, |mut acc| {
			if let Some(ref mut account) = acc {
				let accountdb = self.factories.accountdb.readonly(self.db.as_hash_db(), account.address_hash(a));
				if !Self::update_account_cache(require, account, &self.db, accountdb.as_hash_db()) {
					return Err(Box::new(TrieError::IncompleteDatabase(H256::from(*a))));
				}
			}
			Ok(f(acc.map(|a| &*a)))
		});
		match result {
			Some(r) => Ok(r?),
			None => {
				// first check if it is not in database for sure
				if check_null && self.db.is_known_null(a) { return Ok(f(None)); }

				// not found in the global cache, get from the DB and insert into local
				let db = &self.db.as_hash_db();
				let db = self.factories.trie.readonly(db, &self.root)?;
				let from_rlp = |b: &[u8]| Account::from_rlp(b).expect("decoding db value failed");
				let mut maybe_acc = db.get_with(a.as_bytes(), from_rlp)?;
				if let Some(ref mut account) = maybe_acc.as_mut() {
					let accountdb = self.factories.accountdb.readonly(self.db.as_hash_db(), account.address_hash(a));
					if !Self::update_account_cache(require, account, &self.db, accountdb.as_hash_db()) {
						return Err(Box::new(TrieError::IncompleteDatabase(H256::from(*a))));
					}
				}
				let r = f(maybe_acc.as_ref());
				self.insert_cache(a, AccountEntry::new_clean(maybe_acc));
				Ok(r)
			}
		}
	}

	/// Pull account `a` in our cache from the trie DB. `require_code` requires that the code be cached, too.
	fn require<'a>(&'a self, a: &Address, require_code: bool) -> TrieResult<RefMut<'a, Account>> {
		self.require_or_from(a, require_code, || Account::new_basic(0u8.into(), self.account_start_nonce), |_| {})
	}

	/// Pull account `a` in our cache from the trie DB. `require_code` requires that the code be cached, too.
	/// If it doesn't exist, make account equal the evaluation of `default`.
	fn require_or_from<'a, F, G>(&'a self, a: &Address, require_code: bool, default: F, not_default: G) -> TrieResult<RefMut<'a, Account>>
		where F: FnOnce() -> Account, G: FnOnce(&mut Account),
	{
		let contains_key = self.cache.borrow().contains_key(a);
		if !contains_key {
			match self.db.get_cached_account(a) {
				Some(acc) => self.insert_cache(a, AccountEntry::new_clean_cached(acc)),
				None => {
					let maybe_acc = if !self.db.is_known_null(a) {
						let db = &self.db.as_hash_db();
						let db = self.factories.trie.readonly(db, &self.root)?;
						let from_rlp = |b:&[u8]| { Account::from_rlp(b).expect("decoding db value failed") };
						AccountEntry::new_clean(db.get_with(a.as_bytes(), from_rlp)?)
					} else {
						AccountEntry::new_clean(None)
					};
					self.insert_cache(a, maybe_acc);
				}
			}
		}
		self.note_cache(a);

		// at this point the entry is guaranteed to be in the cache.
		let mut account = RefMut::map(self.cache.borrow_mut(), |c| {
			let entry = c.get_mut(a).expect("entry known to exist in the cache; qed");

			match &mut entry.account {
				&mut Some(ref mut acc) => not_default(acc),
				slot => *slot = Some(default()),
			}

			// set the dirty flag after changing account data.
			entry.state = AccountState::Dirty;
			entry.account.as_mut().expect("Required account must always exist; qed")
		});

		if require_code {
			let addr_hash = account.address_hash(a);
			let accountdb = self.factories.accountdb.readonly(self.db.as_hash_db(), addr_hash);

			if !Self::update_account_cache(RequireCache::Code, &mut account, &self.db, accountdb.as_hash_db()) {
				return Err(Box::new(TrieError::IncompleteDatabase(H256::from(*a))))
			}
		}

		Ok(account)
	}

	/// Replace account code and storage. Creates account if it does not exist.
	pub fn patch_account(&self, a: &Address, code: Arc<Bytes>, storage: HashMap<H256, H256>) -> TrieResult<()> {
		Ok(self.require(a, false)?.reset_code_and_storage(code, storage))
	}
}
