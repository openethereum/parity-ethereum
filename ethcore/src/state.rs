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

use common::*;
use engine::Engine;
use executive::{Executive, TransactOptions};
use account_db::*;
use trace::Trace;
#[cfg(test)]
#[cfg(feature = "json-tests")]
use pod_account::*;
#[cfg(test)]
#[cfg(feature = "json-tests")]
use pod_state::PodState;
//use state_diff::*;	// TODO: uncomment once to_pod() works correctly.

/// Used to return information about an `State::apply` operation.
pub struct ApplyOutcome {
	/// The receipt for the applied transaction.
	pub receipt: Receipt,
	/// The trace for the applied transaction, if None if tracing is disabled.
	pub trace: Option<Trace>,
}

/// Result type for the execution ("application") of a transaction.
pub type ApplyResult = Result<ApplyOutcome, Error>;

/// Representation of the entire state of all accounts in the system.
pub struct State {
	db: Box<JournalDB>,
	root: H256,
	cache: RefCell<HashMap<Address, Option<Account>>>,
	snapshots: RefCell<Vec<HashMap<Address, Option<Option<Account>>>>>,
	account_start_nonce: U256,
}

impl State {
	/// Creates new state with empty state root
	#[cfg(test)]
	pub fn new(mut db: Box<JournalDB>, account_start_nonce: U256) -> State {
		let mut root = H256::new();
		{
			// init trie and reset root too null
			let _ = SecTrieDBMut::new(db.as_hashdb_mut(), &mut root);
		}

		State {
			db: db,
			root: root,
			cache: RefCell::new(HashMap::new()),
			snapshots: RefCell::new(Vec::new()),
			account_start_nonce: account_start_nonce,
		}
	}

	/// Creates new state with existing state root
	pub fn from_existing(db: Box<JournalDB>, root: H256, account_start_nonce: U256) -> State {
		{
			// trie should panic! if root does not exist
			let _ = SecTrieDB::new(db.as_hashdb(), &root);
		}

		State {
			db: db,
			root: root,
			cache: RefCell::new(HashMap::new()),
			snapshots: RefCell::new(Vec::new()),
			account_start_nonce: account_start_nonce,
		}
	}

	/// Create a recoverable snaphot of this state
	pub fn snapshot(&mut self) {
		self.snapshots.borrow_mut().push(HashMap::new());
	}

	/// Merge last snapshot with previous
	pub fn clear_snapshot(&mut self) {
		// merge with previous snapshot
		let last = self.snapshots.borrow_mut().pop();
		if let Some(mut snapshot) = last {
			if let Some(ref mut prev) = self.snapshots.borrow_mut().last_mut() {
				for (k, v) in snapshot.drain() {
					prev.entry(k).or_insert(v);
				}
			}
		}
	}

	/// Revert to snapshot
	pub fn revert_snapshot(&mut self) {
		if let Some(mut snapshot) = self.snapshots.borrow_mut().pop() {
			for (k, v) in snapshot.drain() {
				match v {
					Some(v) => {
						self.cache.borrow_mut().insert(k, v);
					},
					None => {
						self.cache.borrow_mut().remove(&k);
					}
				}
			}
		}
	}

	fn insert_cache(&self, address: &Address, account: Option<Account>) {
		if let Some(ref mut snapshot) = self.snapshots.borrow_mut().last_mut() {
			if !snapshot.contains_key(&address) {
				snapshot.insert(address.clone(), self.cache.borrow_mut().insert(address.clone(), account));
				return;
			}
		}
		self.cache.borrow_mut().insert(address.clone(), account);
	}

	fn note_cache(&self, address: &Address) {
		if let Some(ref mut snapshot) = self.snapshots.borrow_mut().last_mut() {
			if !snapshot.contains_key(&address) {
				snapshot.insert(address.clone(), self.cache.borrow().get(address).cloned());
			}
		}
	}

	/// Destroy the current object and return root and database.
	pub fn drop(self) -> (H256, Box<JournalDB>) {
		(self.root, self.db)
	}

	/// Return reference to root
	pub fn root(&self) -> &H256 {
		&self.root
	}

	/// Create a new contract at address `contract`. If there is already an account at the address
	/// it will have its code reset, ready for `init_code()`.
	pub fn new_contract(&mut self, contract: &Address, balance: U256) {
		self.insert_cache(&contract, Some(Account::new_contract(balance, self.account_start_nonce)));
	}

	/// Remove an existing account.
	pub fn kill_account(&mut self, account: &Address) {
		self.insert_cache(account, None);
	}

	/// Determine whether an account exists.
	pub fn exists(&self, a: &Address) -> bool {
		self.cache.borrow().get(&a).unwrap_or(&None).is_some() || SecTrieDB::new(self.db.as_hashdb(), &self.root).contains(&a)
	}

	/// Get the balance of account `a`.
	pub fn balance(&self, a: &Address) -> U256 {
		self.get(a, false).as_ref().map_or(U256::zero(), |account| *account.balance())
	}

	/// Get the nonce of account `a`.
	pub fn nonce(&self, a: &Address) -> U256 {
		self.get(a, false).as_ref().map_or(U256::zero(), |account| *account.nonce())
	}

	/// Mutate storage of account `address` so that it is `value` for `key`.
	pub fn storage_at(&self, address: &Address, key: &H256) -> H256 {
		self.get(address, false).as_ref().map_or(H256::new(), |a|a.storage_at(&AccountDB::new(self.db.as_hashdb(), address), key))
	}

	/// Mutate storage of account `a` so that it is `value` for `key`.
	pub fn code(&self, a: &Address) -> Option<Bytes> {
		self.get(a, true).as_ref().map_or(None, |a|a.code().map(|x|x.to_vec()))
	}

	/// Add `incr` to the balance of account `a`.
	pub fn add_balance(&mut self, a: &Address, incr: &U256) {
		let old = self.balance(a);
		self.require(a, false).add_balance(incr);
		trace!("state: add_balance({}, {}): {} -> {}\n", a, incr, old, self.balance(a));
	}

	/// Subtract `decr` from the balance of account `a`.
	pub fn sub_balance(&mut self, a: &Address, decr: &U256) {
		let old = self.balance(a);
		self.require(a, false).sub_balance(decr);
		trace!("state: sub_balance({}, {}): {} -> {}\n", a, decr, old, self.balance(a));
	}

	/// Subtracts `by` from the balance of `from` and adds it to that of `to`.
	pub fn transfer_balance(&mut self, from: &Address, to: &Address, by: &U256) {
		self.sub_balance(from, by);
		self.add_balance(to, by);
	}

	/// Increment the nonce of account `a` by 1.
	pub fn inc_nonce(&mut self, a: &Address) {
		self.require(a, false).inc_nonce()
	}

	/// Mutate storage of account `a` so that it is `value` for `key`.
	pub fn set_storage(&mut self, a: &Address, key: H256, value: H256) {
		self.require(a, false).set_storage(key, value)
	}

	/// Initialise the code of account `a` so that it is `value` for `key`.
	/// NOTE: Account should have been created with `new_contract`.
	pub fn init_code(&mut self, a: &Address, code: Bytes) {
		self.require_or_from(a, true, || Account::new_contract(x!(0), self.account_start_nonce), |_|{}).init_code(code);
	}

	/// Execute a given transaction.
	/// This will change the state accordingly.
	pub fn apply(&mut self, env_info: &EnvInfo, engine: &Engine, t: &SignedTransaction, tracing: bool) -> ApplyResult {
//		let old = self.to_pod();

		let options = TransactOptions { tracing: tracing, check_nonce: true };
		let e = try!(Executive::new(self, env_info, engine).transact(t, options));

		// TODO uncomment once to_pod() works correctly.
//		trace!("Applied transaction. Diff:\n{}\n", StateDiff::diff_pod(&old, &self.to_pod()));
		self.commit();
		let receipt = Receipt::new(self.root().clone(), e.cumulative_gas_used, e.logs);
//		trace!("Transaction receipt: {:?}", receipt);
		Ok(ApplyOutcome{receipt: receipt, trace: e.trace})
	}

	/// Commit accounts to SecTrieDBMut. This is similar to cpp-ethereum's dev::eth::commit.
	/// `accounts` is mutable because we may need to commit the code or storage and record that.
	#[cfg_attr(feature="dev", allow(match_ref_pats))]
	pub fn commit_into(db: &mut HashDB, root: &mut H256, accounts: &mut HashMap<Address, Option<Account>>) {
		// first, commit the sub trees.
		// TODO: is this necessary or can we dispense with the `ref mut a` for just `a`?
		for (address, ref mut a) in accounts.iter_mut() {
			match a {
				&mut&mut Some(ref mut account) => {
					let mut account_db = AccountDBMut::new(db, address);
					account.commit_storage(&mut account_db);
					account.commit_code(&mut account_db);
				}
				&mut&mut None => {}
			}
		}

		{
			let mut trie = SecTrieDBMut::from_existing(db, root);
			for (address, ref a) in accounts.iter() {
				match **a {
					Some(ref account) => trie.insert(address, &account.rlp()),
					None => trie.remove(address),
				}
			}
		}
	}

	/// Commits our cached account changes into the trie.
	pub fn commit(&mut self) {
		assert!(self.snapshots.borrow().is_empty());
		Self::commit_into(self.db.as_hashdb_mut(), &mut self.root, self.cache.borrow_mut().deref_mut());
	}

	#[cfg(test)]
	#[cfg(feature = "json-tests")]
	/// Populate the state from `accounts`.
	pub fn populate_from(&mut self, accounts: PodState) {
		assert!(self.snapshots.borrow().is_empty());
		for (add, acc) in accounts.drain().into_iter() {
			self.cache.borrow_mut().insert(add, Some(Account::from_pod(acc)));
		}
	}

	#[cfg(test)]
	#[cfg(feature = "json-tests")]
	/// Populate a PodAccount map from this state.
	pub fn to_pod(&self) -> PodState {
		assert!(self.snapshots.borrow().is_empty());
		// TODO: handle database rather than just the cache.
		PodState::from(self.cache.borrow().iter().fold(BTreeMap::new(), |mut m, (add, opt)| {
			if let Some(ref acc) = *opt {
				m.insert(add.clone(), PodAccount::from_account(acc));
			}
			m
		}))
	}

	/// Pull account `a` in our cache from the trie DB and return it.
	/// `require_code` requires that the code be cached, too.
	fn get<'a>(&'a self, a: &Address, require_code: bool) -> &'a Option<Account> {
		let have_key = self.cache.borrow().contains_key(a);
		if !have_key {
			self.insert_cache(a, SecTrieDB::new(self.db.as_hashdb(), &self.root).get(&a).map(Account::from_rlp))
		}
		if require_code {
			if let Some(ref mut account) = self.cache.borrow_mut().get_mut(a).unwrap().as_mut() {
				account.cache_code(&AccountDB::new(self.db.as_hashdb(), a));
			}
		}
		unsafe { ::std::mem::transmute(self.cache.borrow().get(a).unwrap()) }
	}

	/// Pull account `a` in our cache from the trie DB. `require_code` requires that the code be cached, too.
	fn require<'a>(&'a self, a: &Address, require_code: bool) -> &'a mut Account {
		self.require_or_from(a, require_code, || Account::new_basic(U256::from(0u8), self.account_start_nonce), |_|{})
	}

	/// Pull account `a` in our cache from the trie DB. `require_code` requires that the code be cached, too.
	/// If it doesn't exist, make account equal the evaluation of `default`.
	fn require_or_from<'a, F: FnOnce() -> Account, G: FnOnce(&mut Account)>(&self, a: &Address, require_code: bool, default: F, not_default: G) -> &'a mut Account {
		let have_key = self.cache.borrow().contains_key(a);
		if !have_key {
			self.insert_cache(a, SecTrieDB::new(self.db.as_hashdb(), &self.root).get(&a).map(Account::from_rlp))
		} else {
			self.note_cache(a);
		}
		let preexists = self.cache.borrow().get(a).unwrap().is_none();
		if preexists {
			self.cache.borrow_mut().insert(a.clone(), Some(default()));
		} else {
			not_default(self.cache.borrow_mut().get_mut(a).unwrap().as_mut().unwrap());
		}

		unsafe { ::std::mem::transmute(self.cache.borrow_mut().get_mut(a).unwrap().as_mut().map(|account| {
			if require_code {
				account.cache_code(&AccountDB::new(self.db.as_hashdb(), a));
			}
			account
		}).unwrap()) }
	}
}

impl fmt::Debug for State {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "{:?}", self.cache.borrow())
	}
}

impl Clone for State {
	fn clone(&self) -> State {
		State {
			db: self.db.boxed_clone(),
			root: self.root.clone(),
			cache: RefCell::new(self.cache.borrow().clone()),
			snapshots: RefCell::new(self.snapshots.borrow().clone()),
			account_start_nonce: self.account_start_nonce.clone(),
		}
	}
}

#[cfg(test)]
mod tests {

use super::*;
use util::common::*;
use util::trie::*;
use util::rlp::*;
use account::*;
use tests::helpers::*;
use devtools::*;
use evm::factory::*;
use env_info::*;
use spec::*;
use transaction::*;
use util::log::init_log;
use trace::trace;
use trace::trace::{Trace};

#[test]
fn should_apply_create_transaction() {
	init_log();

	let temp = RandomTempPath::new();
	let mut state = get_temp_state_in(temp.as_path());

	let mut info = EnvInfo::default();
	info.gas_limit = x!(1_000_000);
	let engine = TestEngine::new(5, Factory::default());

	let t = Transaction {
		nonce: x!(0),
		gas_price: x!(0),
		gas: x!(100_000),
		action: Action::Create,
		value: x!(100),
		data: FromHex::from_hex("601080600c6000396000f3006000355415600957005b60203560003555").unwrap(),
	}.sign(&"".sha3());

	state.add_balance(t.sender().as_ref().unwrap(), &x!(100));
	let result = state.apply(&info, &engine, &t, true).unwrap();
	let expected_trace = Some(Trace {
		depth: 0,
		action: trace::Action::Create(trace::Create {
			from: x!("9cce34f7ab185c7aba1b7c8140d620b4bda941d6"),
			value: x!(100),
			gas: x!(77412),
			init: vec![96, 16, 128, 96, 12, 96, 0, 57, 96, 0, 243, 0, 96, 0, 53, 84, 21, 96, 9, 87, 0, 91, 96, 32, 53, 96, 0, 53, 85],
		}),
		result: trace::Res::Create(trace::CreateResult {
			gas_used: U256::from(3224),
			address: Address::from_str("8988167e088c87cd314df6d3c2b83da5acb93ace").unwrap(),
			code: vec![96, 0, 53, 84, 21, 96, 9, 87, 0, 91, 96, 32, 53, 96, 0, 53]
		}),
		subs: vec![]
	});

	assert_eq!(result.trace, expected_trace);
}

#[test]
fn should_work_when_cloned() {
	init_log();

	let a = Address::zero();

	let temp = RandomTempPath::new();
	let mut state = {
		let mut state = get_temp_state_in(temp.as_path());
		assert_eq!(state.exists(&a), false);
		state.inc_nonce(&a);
		state.commit();
		state.clone()
	};

	state.inc_nonce(&a);
	state.commit();
}

#[test]
fn should_trace_failed_create_transaction() {
	init_log();

	let temp = RandomTempPath::new();
	let mut state = get_temp_state_in(temp.as_path());

	let mut info = EnvInfo::default();
	info.gas_limit = x!(1_000_000);
	let engine = TestEngine::new(5, Factory::default());

	let t = Transaction {
		nonce: x!(0),
		gas_price: x!(0),
		gas: x!(100_000),
		action: Action::Create,
		value: x!(100),
		data: FromHex::from_hex("5b600056").unwrap(),
	}.sign(&"".sha3());

	state.add_balance(t.sender().as_ref().unwrap(), &x!(100));
	let result = state.apply(&info, &engine, &t, true).unwrap();
	let expected_trace = Some(Trace {
		depth: 0,
		action: trace::Action::Create(trace::Create {
			from: x!("9cce34f7ab185c7aba1b7c8140d620b4bda941d6"),
			value: x!(100),
			gas: x!(78792),
			init: vec![91, 96, 0, 86],
		}),
		result: trace::Res::FailedCreate,
		subs: vec![]
	});

	assert_eq!(result.trace, expected_trace);
}

#[test]
fn should_trace_call_transaction() {
	init_log();

	let temp = RandomTempPath::new();
	let mut state = get_temp_state_in(temp.as_path());

	let mut info = EnvInfo::default();
	info.gas_limit = x!(1_000_000);
	let engine = TestEngine::new(5, Factory::default());

	let t = Transaction {
		nonce: x!(0),
		gas_price: x!(0),
		gas: x!(100_000),
		action: Action::Call(x!(0xa)),
		value: x!(100),
		data: vec![],
	}.sign(&"".sha3());

	state.init_code(&x!(0xa), FromHex::from_hex("6000").unwrap());
	state.add_balance(t.sender().as_ref().unwrap(), &x!(100));
	let result = state.apply(&info, &engine, &t, true).unwrap();
	let expected_trace = Some(Trace {
		depth: 0,
		action: trace::Action::Call(trace::Call {
			from: x!("9cce34f7ab185c7aba1b7c8140d620b4bda941d6"),
			to: x!(0xa),
			value: x!(100),
			gas: x!(79000),
			input: vec![],
		}),
		result: trace::Res::Call(trace::CallResult {
			gas_used: U256::from(3),
			output: vec![]
		}),
		subs: vec![]
	});

	assert_eq!(result.trace, expected_trace);
}

#[test]
fn should_trace_basic_call_transaction() {
	init_log();

	let temp = RandomTempPath::new();
	let mut state = get_temp_state_in(temp.as_path());

	let mut info = EnvInfo::default();
	info.gas_limit = x!(1_000_000);
	let engine = TestEngine::new(5, Factory::default());

	let t = Transaction {
		nonce: x!(0),
		gas_price: x!(0),
		gas: x!(100_000),
		action: Action::Call(x!(0xa)),
		value: x!(100),
		data: vec![],
	}.sign(&"".sha3());

	state.add_balance(t.sender().as_ref().unwrap(), &x!(100));
	let result = state.apply(&info, &engine, &t, true).unwrap();
	let expected_trace = Some(Trace {
		depth: 0,
		action: trace::Action::Call(trace::Call {
			from: x!("9cce34f7ab185c7aba1b7c8140d620b4bda941d6"),
			to: x!(0xa),
			value: x!(100),
			gas: x!(79000),
			input: vec![],
		}),
		result: trace::Res::Call(trace::CallResult {
			gas_used: U256::from(0),
			output: vec![]
		}),
		subs: vec![]
	});

	assert_eq!(result.trace, expected_trace);
}

#[test]
fn should_trace_call_transaction_to_builtin() {
	init_log();

	let temp = RandomTempPath::new();
	let mut state = get_temp_state_in(temp.as_path());

	let mut info = EnvInfo::default();
	info.gas_limit = x!(1_000_000);
	let engine = Spec::new_test().engine;

	let t = Transaction {
		nonce: x!(0),
		gas_price: x!(0),
		gas: x!(100_000),
		action: Action::Call(x!(0x1)),
		value: x!(0),
		data: vec![],
	}.sign(&"".sha3());

	let result = state.apply(&info, engine.deref(), &t, true).unwrap();

	assert_eq!(result.trace, Some(Trace {
		depth: 0,
		action: trace::Action::Call(trace::Call {
			from: x!("9cce34f7ab185c7aba1b7c8140d620b4bda941d6"),
			to: x!("0000000000000000000000000000000000000001"),
			value: x!(0),
			gas: x!(79_000),
			input: vec![],
		}),
		result: trace::Res::Call(trace::CallResult {
			gas_used: U256::from(3000),
			output: vec![]
		}),
		subs: vec![]
	}));
}

#[test]
fn should_not_trace_subcall_transaction_to_builtin() {
	init_log();

	let temp = RandomTempPath::new();
	let mut state = get_temp_state_in(temp.as_path());

	let mut info = EnvInfo::default();
	info.gas_limit = x!(1_000_000);
	let engine = Spec::new_test().engine;

	let t = Transaction {
		nonce: x!(0),
		gas_price: x!(0),
		gas: x!(100_000),
		action: Action::Call(x!(0xa)),
		value: x!(0),
		data: vec![],
	}.sign(&"".sha3());

	state.init_code(&x!(0xa), FromHex::from_hex("600060006000600060006001610be0f1").unwrap());
	let result = state.apply(&info, engine.deref(), &t, true).unwrap();

	let expected_trace = Some(Trace {
		depth: 0,
		action: trace::Action::Call(trace::Call {
			from: x!("9cce34f7ab185c7aba1b7c8140d620b4bda941d6"),
			to: x!(0xa),
			value: x!(0),
			gas: x!(79000),
			input: vec![],
		}),
		result: trace::Res::Call(trace::CallResult {
			gas_used: U256::from(28_061),
			output: vec![]
		}),
		subs: vec![]
	});
	assert_eq!(result.trace, expected_trace);
}

#[test]
fn should_not_trace_callcode() {
	init_log();

	let temp = RandomTempPath::new();
	let mut state = get_temp_state_in(temp.as_path());

	let mut info = EnvInfo::default();
	info.gas_limit = x!(1_000_000);
	let engine = Spec::new_test().engine;

	let t = Transaction {
		nonce: x!(0),
		gas_price: x!(0),
		gas: x!(100_000),
		action: Action::Call(x!(0xa)),
		value: x!(0),
		data: vec![],
	}.sign(&"".sha3());

	state.init_code(&x!(0xa), FromHex::from_hex("60006000600060006000600b611000f2").unwrap());
	state.init_code(&x!(0xb), FromHex::from_hex("6000").unwrap());
	let result = state.apply(&info, engine.deref(), &t, true).unwrap();

	let expected_trace = Some(Trace {
		depth: 0,
		action: trace::Action::Call(trace::Call {
			from: x!("9cce34f7ab185c7aba1b7c8140d620b4bda941d6"),
			to: x!(0xa),
			value: x!(0),
			gas: x!(79000),
			input: vec![],
		}),
		result: trace::Res::Call(trace::CallResult {
			gas_used: U256::from(64),
			output: vec![]
		}),
		subs: vec![]
	});
	assert_eq!(result.trace, expected_trace);
}

#[test]
fn should_not_trace_delegatecall() {
	init_log();

	let temp = RandomTempPath::new();
	let mut state = get_temp_state_in(temp.as_path());

	let mut info = EnvInfo::default();
	info.gas_limit = x!(1_000_000);
	info.number = 0x789b0;
	let engine = Spec::new_test().engine;

	println!("schedule.have_delegate_call: {:?}", engine.schedule(&info).have_delegate_call);

	let t = Transaction {
		nonce: x!(0),
		gas_price: x!(0),
		gas: x!(100_000),
		action: Action::Call(x!(0xa)),
		value: x!(0),
		data: vec![],
	}.sign(&"".sha3());

	state.init_code(&x!(0xa), FromHex::from_hex("6000600060006000600b618000f4").unwrap());
	state.init_code(&x!(0xb), FromHex::from_hex("6000").unwrap());
	let result = state.apply(&info, engine.deref(), &t, true).unwrap();

	let expected_trace = Some(Trace {
		depth: 0,
		action: trace::Action::Call(trace::Call {
			from: x!("9cce34f7ab185c7aba1b7c8140d620b4bda941d6"),
			to: x!(0xa),
			value: x!(0),
			gas: x!(79000),
			input: vec![],
		}),
		result: trace::Res::Call(trace::CallResult {
			gas_used: U256::from(61),
			output: vec![]
		}),
		subs: vec![]
	});
	assert_eq!(result.trace, expected_trace);
}

#[test]
fn should_trace_failed_call_transaction() {
	init_log();

	let temp = RandomTempPath::new();
	let mut state = get_temp_state_in(temp.as_path());

	let mut info = EnvInfo::default();
	info.gas_limit = x!(1_000_000);
	let engine = TestEngine::new(5, Factory::default());

	let t = Transaction {
		nonce: x!(0),
		gas_price: x!(0),
		gas: x!(100_000),
		action: Action::Call(x!(0xa)),
		value: x!(100),
		data: vec![],
	}.sign(&"".sha3());

	state.init_code(&x!(0xa), FromHex::from_hex("5b600056").unwrap());
	state.add_balance(t.sender().as_ref().unwrap(), &x!(100));
	let result = state.apply(&info, &engine, &t, true).unwrap();
	let expected_trace = Some(Trace {
		depth: 0,
		action: trace::Action::Call(trace::Call {
			from: x!("9cce34f7ab185c7aba1b7c8140d620b4bda941d6"),
			to: x!(0xa),
			value: x!(100),
			gas: x!(79000),
			input: vec![],
		}),
		result: trace::Res::FailedCall,
		subs: vec![]
	});

	println!("trace: {:?}", result.trace);

	assert_eq!(result.trace, expected_trace);
}

#[test]
fn should_trace_call_with_subcall_transaction() {
	init_log();

	let temp = RandomTempPath::new();
	let mut state = get_temp_state_in(temp.as_path());

	let mut info = EnvInfo::default();
	info.gas_limit = x!(1_000_000);
	let engine = TestEngine::new(5, Factory::default());

	let t = Transaction {
		nonce: x!(0),
		gas_price: x!(0),
		gas: x!(100_000),
		action: Action::Call(x!(0xa)),
		value: x!(100),
		data: vec![],
	}.sign(&"".sha3());

	state.init_code(&x!(0xa), FromHex::from_hex("60006000600060006000600b602b5a03f1").unwrap());
	state.init_code(&x!(0xb), FromHex::from_hex("6000").unwrap());
	state.add_balance(t.sender().as_ref().unwrap(), &x!(100));
	let result = state.apply(&info, &engine, &t, true).unwrap();
	let expected_trace = Some(Trace {
		depth: 0,
		action: trace::Action::Call(trace::Call {
			from: x!("9cce34f7ab185c7aba1b7c8140d620b4bda941d6"),
			to: x!(0xa),
			value: x!(100),
			gas: x!(79000),
			input: vec![],
		}),
		result: trace::Res::Call(trace::CallResult {
			gas_used: U256::from(69),
			output: vec![]
		}),
		subs: vec![Trace {
			depth: 1,
			action: trace::Action::Call(trace::Call {
				from: x!(0xa),
				to: x!(0xb),
				value: x!(0),
				gas: x!(78934),
				input: vec![],
			}),
			result: trace::Res::Call(trace::CallResult {
				gas_used: U256::from(3),
				output: vec![]
			}),
			subs: vec![]
		}]
	});

	assert_eq!(result.trace, expected_trace);
}

#[test]
fn should_trace_call_with_basic_subcall_transaction() {
	init_log();

	let temp = RandomTempPath::new();
	let mut state = get_temp_state_in(temp.as_path());

	let mut info = EnvInfo::default();
	info.gas_limit = x!(1_000_000);
	let engine = TestEngine::new(5, Factory::default());

	let t = Transaction {
		nonce: x!(0),
		gas_price: x!(0),
		gas: x!(100_000),
		action: Action::Call(x!(0xa)),
		value: x!(100),
		data: vec![],
	}.sign(&"".sha3());

	state.init_code(&x!(0xa), FromHex::from_hex("60006000600060006045600b6000f1").unwrap());
	state.add_balance(t.sender().as_ref().unwrap(), &x!(100));
	let result = state.apply(&info, &engine, &t, true).unwrap();
	let expected_trace = Some(Trace {
		depth: 0,
		action: trace::Action::Call(trace::Call {
			from: x!("9cce34f7ab185c7aba1b7c8140d620b4bda941d6"),
			to: x!(0xa),
			value: x!(100),
			gas: x!(79000),
			input: vec![],
		}),
		result: trace::Res::Call(trace::CallResult {
			gas_used: U256::from(31761),
			output: vec![]
		}),
		subs: vec![Trace {
			depth: 1,
			action: trace::Action::Call(trace::Call {
				from: x!(0xa),
				to: x!(0xb),
				value: x!(69),
				gas: x!(2300),
				input: vec![],
			}),
			result: trace::Res::Call(trace::CallResult::default()),
			subs: vec![]
		}]
	});

	assert_eq!(result.trace, expected_trace);
}

#[test]
fn should_not_trace_call_with_invalid_basic_subcall_transaction() {
	init_log();

	let temp = RandomTempPath::new();
	let mut state = get_temp_state_in(temp.as_path());

	let mut info = EnvInfo::default();
	info.gas_limit = x!(1_000_000);
	let engine = TestEngine::new(5, Factory::default());

	let t = Transaction {
		nonce: x!(0),
		gas_price: x!(0),
		gas: x!(100_000),
		action: Action::Call(x!(0xa)),
		value: x!(100),
		data: vec![],
	}.sign(&"".sha3());

	state.init_code(&x!(0xa), FromHex::from_hex("600060006000600060ff600b6000f1").unwrap());	// not enough funds.
	state.add_balance(t.sender().as_ref().unwrap(), &x!(100));
	let result = state.apply(&info, &engine, &t, true).unwrap();
	let expected_trace = Some(Trace {
		depth: 0,
		action: trace::Action::Call(trace::Call {
			from: x!("9cce34f7ab185c7aba1b7c8140d620b4bda941d6"),
			to: x!(0xa),
			value: x!(100),
			gas: x!(79000),
			input: vec![],
		}),
		result: trace::Res::Call(trace::CallResult {
			gas_used: U256::from(31761),
			output: vec![]
		}),
		subs: vec![]
	});

	assert_eq!(result.trace, expected_trace);
}

#[test]
fn should_trace_failed_subcall_transaction() {
	init_log();

	let temp = RandomTempPath::new();
	let mut state = get_temp_state_in(temp.as_path());

	let mut info = EnvInfo::default();
	info.gas_limit = x!(1_000_000);
	let engine = TestEngine::new(5, Factory::default());

	let t = Transaction {
		nonce: x!(0),
		gas_price: x!(0),
		gas: x!(100_000),
		action: Action::Call(x!(0xa)),
		value: x!(100),
		data: vec![],//600480600b6000396000f35b600056
	}.sign(&"".sha3());

	state.init_code(&x!(0xa), FromHex::from_hex("60006000600060006000600b602b5a03f1").unwrap());
	state.init_code(&x!(0xb), FromHex::from_hex("5b600056").unwrap());
	state.add_balance(t.sender().as_ref().unwrap(), &x!(100));
	let result = state.apply(&info, &engine, &t, true).unwrap();
	let expected_trace = Some(Trace {
		depth: 0,
		action: trace::Action::Call(trace::Call {
			from: x!("9cce34f7ab185c7aba1b7c8140d620b4bda941d6"),
			to: x!(0xa),
			value: x!(100),
			gas: x!(79000),
			input: vec![],
		}),
		result: trace::Res::Call(trace::CallResult {
			gas_used: U256::from(79_000),
			output: vec![]
		}),
		subs: vec![Trace {
			depth: 1,
			action: trace::Action::Call(trace::Call {
				from: x!(0xa),
				to: x!(0xb),
				value: x!(0),
				gas: x!(78934),
				input: vec![],
			}),
			result: trace::Res::FailedCall,
			subs: vec![]
		}]
	});

	assert_eq!(result.trace, expected_trace);
}

#[test]
fn should_trace_call_with_subcall_with_subcall_transaction() {
	init_log();

	let temp = RandomTempPath::new();
	let mut state = get_temp_state_in(temp.as_path());

	let mut info = EnvInfo::default();
	info.gas_limit = x!(1_000_000);
	let engine = TestEngine::new(5, Factory::default());

	let t = Transaction {
		nonce: x!(0),
		gas_price: x!(0),
		gas: x!(100_000),
		action: Action::Call(x!(0xa)),
		value: x!(100),
		data: vec![],
	}.sign(&"".sha3());

	state.init_code(&x!(0xa), FromHex::from_hex("60006000600060006000600b602b5a03f1").unwrap());
	state.init_code(&x!(0xb), FromHex::from_hex("60006000600060006000600c602b5a03f1").unwrap());
	state.init_code(&x!(0xc), FromHex::from_hex("6000").unwrap());
	state.add_balance(t.sender().as_ref().unwrap(), &x!(100));
	let result = state.apply(&info, &engine, &t, true).unwrap();
	let expected_trace = Some(Trace {
		depth: 0,
		action: trace::Action::Call(trace::Call {
			from: x!("9cce34f7ab185c7aba1b7c8140d620b4bda941d6"),
			to: x!(0xa),
			value: x!(100),
			gas: x!(79000),
			input: vec![],
		}),
		result: trace::Res::Call(trace::CallResult {
			gas_used: U256::from(135),
			output: vec![]
		}),
		subs: vec![Trace {
			depth: 1,
			action: trace::Action::Call(trace::Call {
				from: x!(0xa),
				to: x!(0xb),
				value: x!(0),
				gas: x!(78934),
				input: vec![],
			}),
			result: trace::Res::Call(trace::CallResult {
				gas_used: U256::from(69),
				output: vec![]
			}),
			subs: vec![Trace {
				depth: 2,
				action: trace::Action::Call(trace::Call {
					from: x!(0xb),
					to: x!(0xc),
					value: x!(0),
					gas: x!(78868),
					input: vec![],
				}),
				result: trace::Res::Call(trace::CallResult {
					gas_used: U256::from(3),
					output: vec![]
				}),
				subs: vec![]
			}]
		}]
	});

	assert_eq!(result.trace, expected_trace);
}

#[test]
fn should_trace_failed_subcall_with_subcall_transaction() {
	init_log();

	let temp = RandomTempPath::new();
	let mut state = get_temp_state_in(temp.as_path());

	let mut info = EnvInfo::default();
	info.gas_limit = x!(1_000_000);
	let engine = TestEngine::new(5, Factory::default());

	let t = Transaction {
		nonce: x!(0),
		gas_price: x!(0),
		gas: x!(100_000),
		action: Action::Call(x!(0xa)),
		value: x!(100),
		data: vec![],//600480600b6000396000f35b600056
	}.sign(&"".sha3());

	state.init_code(&x!(0xa), FromHex::from_hex("60006000600060006000600b602b5a03f1").unwrap());
	state.init_code(&x!(0xb), FromHex::from_hex("60006000600060006000600c602b5a03f1505b601256").unwrap());
	state.init_code(&x!(0xc), FromHex::from_hex("6000").unwrap());
	state.add_balance(t.sender().as_ref().unwrap(), &x!(100));
	let result = state.apply(&info, &engine, &t, true).unwrap();
	let expected_trace = Some(Trace {
		depth: 0,
		action: trace::Action::Call(trace::Call {
			from: x!("9cce34f7ab185c7aba1b7c8140d620b4bda941d6"),
			to: x!(0xa),
			value: x!(100),
			gas: x!(79000),
			input: vec![],
		}),
		result: trace::Res::Call(trace::CallResult {
			gas_used: U256::from(79_000),
			output: vec![]
		}),
		subs: vec![Trace {
			depth: 1,
			action: trace::Action::Call(trace::Call {
				from: x!(0xa),
				to: x!(0xb),
				value: x!(0),
				gas: x!(78934),
				input: vec![],
			}),
			result: trace::Res::FailedCall,
			subs: vec![Trace {
				depth: 2,
				action: trace::Action::Call(trace::Call {
					from: x!(0xb),
					to: x!(0xc),
					value: x!(0),
					gas: x!(78868),
					input: vec![],
				}),
				result: trace::Res::Call(trace::CallResult {
					gas_used: U256::from(3),
					output: vec![]
				}),
				subs: vec![]
			}]
		}]
	});

	assert_eq!(result.trace, expected_trace);
}

#[test]
fn code_from_database() {
	let a = Address::zero();
	let temp = RandomTempPath::new();
	let (root, db) = {
		let mut state = get_temp_state_in(temp.as_path());
		state.require_or_from(&a, false, ||Account::new_contract(x!(42), x!(0)), |_|{});
		state.init_code(&a, vec![1, 2, 3]);
		assert_eq!(state.code(&a), Some([1u8, 2, 3].to_vec()));
		state.commit();
		assert_eq!(state.code(&a), Some([1u8, 2, 3].to_vec()));
		state.drop()
	};

	let state = State::from_existing(db, root, U256::from(0u8));
	assert_eq!(state.code(&a), Some([1u8, 2, 3].to_vec()));
}

#[test]
fn storage_at_from_database() {
	let a = Address::zero();
	let temp = RandomTempPath::new();
	let (root, db) = {
		let mut state = get_temp_state_in(temp.as_path());
		state.set_storage(&a, H256::from(&U256::from(01u64)), H256::from(&U256::from(69u64)));
		state.commit();
		state.drop()
	};

	let s = State::from_existing(db, root, U256::from(0u8));
	assert_eq!(s.storage_at(&a, &H256::from(&U256::from(01u64))), H256::from(&U256::from(69u64)));
}

#[test]
fn get_from_database() {
	let a = Address::zero();
	let temp = RandomTempPath::new();
	let (root, db) = {
		let mut state = get_temp_state_in(temp.as_path());
		state.inc_nonce(&a);
		state.add_balance(&a, &U256::from(69u64));
		state.commit();
		assert_eq!(state.balance(&a), U256::from(69u64));
		state.drop()
	};

	let state = State::from_existing(db, root, U256::from(0u8));
	assert_eq!(state.balance(&a), U256::from(69u64));
	assert_eq!(state.nonce(&a), U256::from(1u64));
}

#[test]
fn remove() {
	let a = Address::zero();
	let mut state_result = get_temp_state();
	let mut state = state_result.reference_mut();
	assert_eq!(state.exists(&a), false);
	state.inc_nonce(&a);
	assert_eq!(state.exists(&a), true);
	assert_eq!(state.nonce(&a), U256::from(1u64));
	state.kill_account(&a);
	assert_eq!(state.exists(&a), false);
	assert_eq!(state.nonce(&a), U256::from(0u64));
}

#[test]
fn remove_from_database() {
	let a = Address::zero();
	let temp = RandomTempPath::new();
	let (root, db) = {
		let mut state = get_temp_state_in(temp.as_path());
		state.inc_nonce(&a);
		state.commit();
		assert_eq!(state.exists(&a), true);
		assert_eq!(state.nonce(&a), U256::from(1u64));
		state.drop()
	};

	let (root, db) = {
		let mut state = State::from_existing(db, root, U256::from(0u8));
		assert_eq!(state.exists(&a), true);
		assert_eq!(state.nonce(&a), U256::from(1u64));
		state.kill_account(&a);
		state.commit();
		assert_eq!(state.exists(&a), false);
		assert_eq!(state.nonce(&a), U256::from(0u64));
		state.drop()
	};

	let state = State::from_existing(db, root, U256::from(0u8));
	assert_eq!(state.exists(&a), false);
	assert_eq!(state.nonce(&a), U256::from(0u64));
}

#[test]
fn alter_balance() {
	let mut state_result = get_temp_state();
	let mut state = state_result.reference_mut();
	let a = Address::zero();
	let b = address_from_u64(1u64);
	state.add_balance(&a, &U256::from(69u64));
	assert_eq!(state.balance(&a), U256::from(69u64));
	state.commit();
	assert_eq!(state.balance(&a), U256::from(69u64));
	state.sub_balance(&a, &U256::from(42u64));
	assert_eq!(state.balance(&a), U256::from(27u64));
	state.commit();
	assert_eq!(state.balance(&a), U256::from(27u64));
	state.transfer_balance(&a, &b, &U256::from(18u64));
	assert_eq!(state.balance(&a), U256::from(9u64));
	assert_eq!(state.balance(&b), U256::from(18u64));
	state.commit();
	assert_eq!(state.balance(&a), U256::from(9u64));
	assert_eq!(state.balance(&b), U256::from(18u64));
}

#[test]
fn alter_nonce() {
	let mut state_result = get_temp_state();
	let mut state = state_result.reference_mut();
	let a = Address::zero();
	state.inc_nonce(&a);
	assert_eq!(state.nonce(&a), U256::from(1u64));
	state.inc_nonce(&a);
	assert_eq!(state.nonce(&a), U256::from(2u64));
	state.commit();
	assert_eq!(state.nonce(&a), U256::from(2u64));
	state.inc_nonce(&a);
	assert_eq!(state.nonce(&a), U256::from(3u64));
	state.commit();
	assert_eq!(state.nonce(&a), U256::from(3u64));
}

#[test]
fn balance_nonce() {
	let mut state_result = get_temp_state();
	let mut state = state_result.reference_mut();
	let a = Address::zero();
	assert_eq!(state.balance(&a), U256::from(0u64));
	assert_eq!(state.nonce(&a), U256::from(0u64));
	state.commit();
	assert_eq!(state.balance(&a), U256::from(0u64));
	assert_eq!(state.nonce(&a), U256::from(0u64));
}

#[test]
fn ensure_cached() {
	let mut state_result = get_temp_state();
	let mut state = state_result.reference_mut();
	let a = Address::zero();
	state.require(&a, false);
	state.commit();
	assert_eq!(state.root().hex(), "0ce23f3c809de377b008a4a3ee94a0834aac8bec1f86e28ffe4fdb5a15b0c785");
}

#[test]
fn snapshot_basic() {
	let mut state_result = get_temp_state();
	let mut state = state_result.reference_mut();
	let a = Address::zero();
	state.snapshot();
	state.add_balance(&a, &U256::from(69u64));
	assert_eq!(state.balance(&a), U256::from(69u64));
	state.clear_snapshot();
	assert_eq!(state.balance(&a), U256::from(69u64));
	state.snapshot();
	state.add_balance(&a, &U256::from(1u64));
	assert_eq!(state.balance(&a), U256::from(70u64));
	state.revert_snapshot();
	assert_eq!(state.balance(&a), U256::from(69u64));
}

#[test]
fn snapshot_nested() {
	let mut state_result = get_temp_state();
	let mut state = state_result.reference_mut();
	let a = Address::zero();
	state.snapshot();
	state.snapshot();
	state.add_balance(&a, &U256::from(69u64));
	assert_eq!(state.balance(&a), U256::from(69u64));
	state.clear_snapshot();
	assert_eq!(state.balance(&a), U256::from(69u64));
	state.revert_snapshot();
	assert_eq!(state.balance(&a), U256::from(0));
}

#[test]
fn create_empty() {
	let mut state_result = get_temp_state();
	let mut state = state_result.reference_mut();
	state.commit();
	assert_eq!(state.root().hex(), "56e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421");
}

}
