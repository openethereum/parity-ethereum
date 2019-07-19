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

//! Parameters for a block chain.

use std::collections::BTreeMap;
use std::io::Read;
use std::path::Path;
use std::sync::Arc;

use bytes::Bytes;
use ethereum_types::{H256, Bloom, U256, Address};
use ethjson;
use hash::{KECCAK_NULL_RLP, keccak};
use parking_lot::RwLock;
use rlp::{Rlp, RlpStream};
use rustc_hex::{FromHex, ToHex};
use types::{
	BlockNumber,
	header::Header,
	encoded,
	engines::params::CommonParams,
	errors::EthcoreError as Error,
};
use vm::{EnvInfo, CallType, ActionValue, ActionParams, ParamsType};

use builtin::Builtin;
use engines::{
	Engine, NullEngine, InstantSeal, InstantSealParams, BasicAuthority, Clique,
	AuthorityRound, Ethash,
};
use executive::Executive;
use trie_vm_factories::Factories;
use machine::Machine;
use pod::PodState;
use spec::Genesis;
use spec::seal::Generic as GenericSeal;
use account_state::{Backend, State, backend::Basic as BasicBackend};
use substate::Substate;
use trace::{NoopTracer, NoopVMTracer};

pub use ethash::OptimizeFor;

// helper for formatting errors.
fn fmt_err<F: ::std::fmt::Display>(f: F) -> String {
	format!("Spec json is invalid: {}", f)
}

/// Runtime parameters for the spec that are related to how the software should run the chain,
/// rather than integral properties of the chain itself.
#[derive(Debug, Clone, Copy)]
pub struct SpecParams<'a> {
	/// The path to the folder used to cache nodes. This is typically /tmp/ on Unix-like systems
	pub cache_dir: &'a Path,
	/// Whether to run slower at the expense of better memory usage, or run faster while using
	/// more
	/// memory. This may get more fine-grained in the future but for now is simply a binary
	/// option.
	pub optimization_setting: Option<OptimizeFor>,
}

impl<'a> SpecParams<'a> {
	/// Create from a cache path, with null values for the other fields
	pub fn from_path(path: &'a Path) -> Self {
		SpecParams {
			cache_dir: path,
			optimization_setting: None,
		}
	}

	/// Create from a cache path and an optimization setting
	pub fn new(path: &'a Path, optimization: OptimizeFor) -> Self {
		SpecParams {
			cache_dir: path,
			optimization_setting: Some(optimization),
		}
	}
}

impl<'a, T: AsRef<Path>> From<&'a T> for SpecParams<'a> {
	fn from(path: &'a T) -> Self {
		Self::from_path(path.as_ref())
	}
}

/// Parameters for a block chain; includes both those intrinsic to the design of the
/// chain and those to be interpreted by the active chain engine.
pub struct Spec {
	/// User friendly spec name
	pub name: String,
	/// What engine are we using for this?
	pub engine: Arc<dyn Engine>,
	/// Name of the subdir inside the main data dir to use for chain data and settings.
	pub data_dir: String,

	/// Known nodes on the network in enode format.
	pub nodes: Vec<String>,

	/// The genesis block's parent hash field.
	pub parent_hash: H256,
	/// The genesis block's author field.
	pub author: Address,
	/// The genesis block's difficulty field.
	pub difficulty: U256,
	/// The genesis block's gas limit field.
	pub gas_limit: U256,
	/// The genesis block's gas used field.
	pub gas_used: U256,
	/// The genesis block's timestamp field.
	pub timestamp: u64,
	/// Transactions root of the genesis block. Should be KECCAK_NULL_RLP.
	pub transactions_root: H256,
	/// Receipts root of the genesis block. Should be KECCAK_NULL_RLP.
	pub receipts_root: H256,
	/// The genesis block's extra data field.
	pub extra_data: Bytes,
	/// Each seal field, expressed as RLP, concatenated.
	pub seal_rlp: Bytes,

	/// Hardcoded synchronization. Allows the light client to immediately jump to a specific block.
	pub hardcoded_sync: Option<SpecHardcodedSync>,

	/// Contract constructors to be executed on genesis.
	constructors: Vec<(Address, Bytes)>,

	/// May be prepopulated if we know this in advance.
	state_root_memo: RwLock<H256>,

	/// Genesis state as plain old data.
	genesis_state: PodState,
}

#[cfg(test)]
impl Clone for Spec {
	fn clone(&self) -> Spec {
		Spec {
			name: self.name.clone(),
			engine: self.engine.clone(),
			data_dir: self.data_dir.clone(),
			nodes: self.nodes.clone(),
			parent_hash: self.parent_hash.clone(),
			transactions_root: self.transactions_root.clone(),
			receipts_root: self.receipts_root.clone(),
			author: self.author.clone(),
			difficulty: self.difficulty.clone(),
			gas_limit: self.gas_limit.clone(),
			gas_used: self.gas_used.clone(),
			timestamp: self.timestamp.clone(),
			extra_data: self.extra_data.clone(),
			seal_rlp: self.seal_rlp.clone(),
			hardcoded_sync: self.hardcoded_sync.clone(),
			constructors: self.constructors.clone(),
			state_root_memo: RwLock::new(*self.state_root_memo.read()),
			genesis_state: self.genesis_state.clone(),
		}
	}
}

/// Part of `Spec`. Describes the hardcoded synchronization parameters.
pub struct SpecHardcodedSync {
	/// Header of the block to jump to for hardcoded sync, and total difficulty.
	pub header: encoded::Header,
	/// Total difficulty of the block to jump to.
	pub total_difficulty: U256,
	/// List of hardcoded CHTs, in order. If `hardcoded_sync` is set, the CHTs should include the
	/// header of `hardcoded_sync`.
	pub chts: Vec<H256>,
}

impl SpecHardcodedSync {
	/// Turns this specifications back into JSON. Useful for pretty printing.
	pub fn to_json(self) -> ethjson::spec::HardcodedSync {
		self.into()
	}
}

#[cfg(test)]
impl Clone for SpecHardcodedSync {
	fn clone(&self) -> SpecHardcodedSync {
		SpecHardcodedSync {
			header: self.header.clone(),
			total_difficulty: self.total_difficulty.clone(),
			chts: self.chts.clone(),
		}
	}
}

impl From<SpecHardcodedSync> for ethjson::spec::HardcodedSync {
	fn from(sync: SpecHardcodedSync) -> ethjson::spec::HardcodedSync {
		ethjson::spec::HardcodedSync {
			header: sync.header.into_inner().to_hex(),
			total_difficulty: ethjson::uint::Uint(sync.total_difficulty),
			chts: sync.chts.into_iter().map(Into::into).collect(),
		}
	}
}

fn load_machine_from(s: ethjson::spec::Spec) -> Machine {
	let builtins = s.accounts.builtins().into_iter().map(|p| (p.0.into(), From::from(p.1))).collect();
	let params = CommonParams::from(s.params);

	Spec::machine(&s.engine, params, builtins)
}

/// Load from JSON object.
fn load_from(spec_params: SpecParams, s: ethjson::spec::Spec) -> Result<Spec, Error> {
	let builtins = s.accounts
		.builtins()
		.into_iter()
		.map(|p| (p.0.into(), From::from(p.1)))
		.collect();
	let g = Genesis::from(s.genesis);
	let GenericSeal(seal_rlp) = g.seal.into();
	let params = CommonParams::from(s.params);

	let hardcoded_sync = if let Some(ref hs) = s.hardcoded_sync {
		if let Ok(header) = hs.header.from_hex() {
			Some(SpecHardcodedSync {
				header: encoded::Header::new(header),
				total_difficulty: hs.total_difficulty.into(),
				chts: s.hardcoded_sync
					.as_ref()
					.map(|s| s.chts.iter().map(|c| c.clone().into()).collect())
					.unwrap_or_default()
			})
		} else {
			None
		}
	} else {
		None
	};

	let mut s = Spec {
		name: s.name.clone().into(),
		engine: Spec::engine(spec_params, s.engine, params, builtins),
		data_dir: s.data_dir.unwrap_or(s.name).into(),
		nodes: s.nodes.unwrap_or_else(Vec::new),
		parent_hash: g.parent_hash,
		transactions_root: g.transactions_root,
		receipts_root: g.receipts_root,
		author: g.author,
		difficulty: g.difficulty,
		gas_limit: g.gas_limit,
		gas_used: g.gas_used,
		timestamp: g.timestamp,
		extra_data: g.extra_data,
		seal_rlp,
		hardcoded_sync,
		constructors: s.accounts
			.constructors()
			.into_iter()
			.map(|(a, c)| (a.into(), c.into()))
			.collect(),
		state_root_memo: RwLock::new(Default::default()), // will be overwritten right after.
		genesis_state: s.accounts.into(),
	};

	// use memoized state root if provided.
	match g.state_root {
		Some(root) => *s.state_root_memo.get_mut() = root,
		None => {
			let _ = s.run_constructors(
				&Default::default(),
				BasicBackend(journaldb::new_memory_db()),
			)?;
		}
	}

	Ok(s)
}

macro_rules! load_bundled {
	($e:expr) => {
		Spec::load(
			&::std::env::temp_dir(),
			include_bytes!(concat!("../../res/", $e, ".json")) as &[u8]
		).expect(concat!("Chain spec ", $e, " is invalid."))
	};
}

#[cfg(any(test, feature = "test-helpers"))]
macro_rules! load_machine_bundled {
	($e:expr) => {
		Spec::load_machine(
			include_bytes!(concat!("../../res/", $e, ".json")) as &[u8]
		).expect(concat!("Chain spec ", $e, " is invalid."))
	};
}

impl Spec {
	// create an instance of an Ethereum state machine, minus consensus logic.
	fn machine(
		engine_spec: &ethjson::spec::Engine,
		params: CommonParams,
		builtins: BTreeMap<Address, Builtin>,
	) -> Machine {
		if let ethjson::spec::Engine::Ethash(ref ethash) = *engine_spec {
			Machine::with_ethash_extensions(params, builtins, ethash.params.clone().into())
		} else {
			Machine::regular(params, builtins)
		}
	}

	/// Convert engine spec into a arc'd Engine of the right underlying type.
	/// TODO avoid this hard-coded nastiness - use dynamic-linked plugin framework instead.
	fn engine(
		spec_params: SpecParams,
		engine_spec: ethjson::spec::Engine,
		params: CommonParams,
		builtins: BTreeMap<Address, Builtin>,
	) -> Arc<dyn Engine> {
		let machine = Self::machine(&engine_spec, params, builtins);

		match engine_spec {
			ethjson::spec::Engine::Null(null) => Arc::new(NullEngine::new(null.params.into(), machine)),
			ethjson::spec::Engine::Ethash(ethash) => Arc::new(Ethash::new(spec_params.cache_dir, ethash.params.into(), machine, spec_params.optimization_setting)),
			ethjson::spec::Engine::InstantSeal(Some(instant_seal)) => Arc::new(InstantSeal::new(instant_seal.params.into(), machine)),
			ethjson::spec::Engine::InstantSeal(None) => Arc::new(InstantSeal::new(InstantSealParams::default(), machine)),
			ethjson::spec::Engine::BasicAuthority(basic_authority) => Arc::new(BasicAuthority::new(basic_authority.params.into(), machine)),
			ethjson::spec::Engine::Clique(clique) => Clique::new(clique.params.into(), machine)
								.expect("Failed to start Clique consensus engine."),
			ethjson::spec::Engine::AuthorityRound(authority_round) => AuthorityRound::new(authority_round.params.into(), machine)
				.expect("Failed to start AuthorityRound consensus engine."),
		}
	}

	// given a pre-constructor state, run all the given constructors and produce a new state and
	// state root.
	fn run_constructors<T: Backend>(&self, factories: &Factories, mut db: T) -> Result<T, Error> {
		let mut root = KECCAK_NULL_RLP;

		// basic accounts in spec.
		{
			let mut t = factories.trie.create(db.as_hash_db_mut(), &mut root);

			for (address, account) in self.genesis_state.get().iter() {
				t.insert(address.as_bytes(), &account.rlp())?;
			}
		}

		for (address, account) in self.genesis_state.get().iter() {
			db.note_non_null_account(address);
			account.insert_additional(
				&mut *factories.accountdb.create(
					db.as_hash_db_mut(),
					keccak(address),
				),
				&factories.trie,
			);
		}

		let start_nonce = self.engine.account_start_nonce(0);

		let (root, db) = {
			let mut state = State::from_existing(db, root, start_nonce, factories.clone())?;

			// Execute contract constructors.
			let env_info = EnvInfo {
				number: 0,
				author: self.author,
				timestamp: self.timestamp,
				difficulty: self.difficulty,
				last_hashes: Default::default(),
				gas_used: U256::zero(),
				gas_limit: U256::max_value(),
			};

			let from = Address::zero();
			for &(ref address, ref constructor) in self.constructors.iter() {
				trace!(target: "spec", "run_constructors: Creating a contract at {}.", address);
				trace!(target: "spec", "  .. root before = {}", state.root());
				let params = ActionParams {
					code_address: address.clone(),
					code_hash: Some(keccak(constructor)),
					code_version: U256::zero(),
					address: address.clone(),
					sender: from.clone(),
					origin: from.clone(),
					gas: U256::max_value(),
					gas_price: Default::default(),
					value: ActionValue::Transfer(Default::default()),
					code: Some(Arc::new(constructor.clone())),
					data: None,
					call_type: CallType::None,
					params_type: ParamsType::Embedded,
				};

				let mut substate = Substate::new();

				{
					let machine = self.engine.machine();
					let schedule = machine.schedule(env_info.number);
					let mut exec = Executive::new(&mut state, &env_info, &machine, &schedule);
					if let Err(e) = exec.create(params, &mut substate, &mut NoopTracer, &mut NoopVMTracer) {
						warn!(target: "spec", "Genesis constructor execution at {} failed: {}.", address, e);
					}
				}

				if let Err(e) = state.commit() {
					warn!(target: "spec", "Genesis constructor trie commit at {} failed: {}.", address, e);
				}

				trace!(target: "spec", "  .. root after = {}", state.root());
			}

			state.drop()
		};

		*self.state_root_memo.write() = root;
		Ok(db)
	}

	/// Return the state root for the genesis state, memoising accordingly.
	pub fn state_root(&self) -> H256 {
		self.state_root_memo.read().clone()
	}

	/// Get common blockchain parameters.
	pub fn params(&self) -> &CommonParams {
		&self.engine.params()
	}

	/// Get the known knodes of the network in enode format.
	pub fn nodes(&self) -> &[String] {
		&self.nodes
	}

	/// Get the configured Network ID.
	pub fn network_id(&self) -> u64 {
		self.params().network_id
	}

	/// Get the chain ID used for signing.
	pub fn chain_id(&self) -> u64 {
		self.params().chain_id
	}

	/// Get the configured subprotocol name.
	pub fn subprotocol_name(&self) -> String {
		self.params().subprotocol_name.clone()
	}

	/// Get the configured network fork block.
	pub fn fork_block(&self) -> Option<(BlockNumber, H256)> {
		self.params().fork_block
	}

	/// Get the header of the genesis block.
	pub fn genesis_header(&self) -> Header {
		let mut header: Header = Default::default();
		header.set_parent_hash(self.parent_hash.clone());
		header.set_timestamp(self.timestamp);
		header.set_number(0);
		header.set_author(self.author.clone());
		header.set_transactions_root(self.transactions_root.clone());
		header.set_uncles_hash(keccak(RlpStream::new_list(0).out()));
		header.set_extra_data(self.extra_data.clone());
		header.set_state_root(self.state_root());
		header.set_receipts_root(self.receipts_root.clone());
		header.set_log_bloom(Bloom::default());
		header.set_gas_used(self.gas_used.clone());
		header.set_gas_limit(self.gas_limit.clone());
		header.set_difficulty(self.difficulty.clone());
		header.set_seal({
			let r = Rlp::new(&self.seal_rlp);
			r.iter().map(|f| f.as_raw().to_vec()).collect()
		});
		trace!(target: "spec", "Header hash is {}", header.hash());
		header
	}

	/// Compose the genesis block for this chain.
	pub fn genesis_block(&self) -> Bytes {
		let empty_list = RlpStream::new_list(0).out();
		let header = self.genesis_header();
		let mut ret = RlpStream::new_list(3);
		ret.append(&header);
		ret.append_raw(&empty_list, 1);
		ret.append_raw(&empty_list, 1);
		ret.out()
	}

	/// Overwrite the genesis components.
	pub fn overwrite_genesis_params(&mut self, g: Genesis) {
		let GenericSeal(seal_rlp) = g.seal.into();
		self.parent_hash = g.parent_hash;
		self.transactions_root = g.transactions_root;
		self.receipts_root = g.receipts_root;
		self.author = g.author;
		self.difficulty = g.difficulty;
		self.gas_limit = g.gas_limit;
		self.gas_used = g.gas_used;
		self.timestamp = g.timestamp;
		self.extra_data = g.extra_data;
		self.seal_rlp = seal_rlp;
	}

	/// Alter the value of the genesis state.
	pub fn set_genesis_state(&mut self, s: PodState) -> Result<(), Error> {
		self.genesis_state = s;
		let _ = self.run_constructors(
			&Default::default(),
			BasicBackend(journaldb::new_memory_db()),
		)?;

		Ok(())
	}

	/// Return genesis state as Plain old data.
	pub fn genesis_state(&self) -> &PodState {
		&self.genesis_state
	}

	/// Returns `false` if the memoized state root is invalid. `true` otherwise.
	pub fn is_state_root_valid(&self) -> bool {
		// TODO: get rid of this function and ensure state root always is valid.
		// we're mostly there, but `self.genesis_state.root()` doesn't encompass
		// post-constructor state.
		*self.state_root_memo.read() == self.genesis_state.root()
	}

	/// Ensure that the given state DB has the trie nodes in for the genesis state.
	pub fn ensure_db_good<T: Backend>(&self, db: T, factories: &Factories) -> Result<T, Error> {
		if db.as_hash_db().contains(&self.state_root(), hash_db::EMPTY_PREFIX) {
			return Ok(db);
		}

		// TODO: could optimize so we don't re-run, but `ensure_db_good` is barely ever
		// called anyway.
		let db = self.run_constructors(factories, db)?;
		Ok(db)
	}

	/// Loads just the state machine from a json file.
	pub fn load_machine<R: Read>(reader: R) -> Result<Machine, String> {
		ethjson::spec::Spec::load(reader)
			.map_err(fmt_err)
			.map(load_machine_from)
	}

	/// Loads spec from json file. Provide factories for executing contracts and ensuring
	/// storage goes to the right place.
	pub fn load<'a, T: Into<SpecParams<'a>>, R>(params: T, reader: R) -> Result<Self, String>
	where
		R: Read,
	{
		ethjson::spec::Spec::load(reader).map_err(fmt_err).and_then(
			|x| {
				load_from(params.into(), x).map_err(fmt_err)
			},
		)
	}

	/// initialize genesis epoch data, using in-memory database for
	/// constructor.
	pub fn genesis_epoch_data(&self) -> Result<Vec<u8>, String> {
		use types::transaction::{Action, Transaction};

		let genesis = self.genesis_header();

		let factories = Default::default();
		let mut db = journaldb::new(
			Arc::new(kvdb_memorydb::create(0)),
			journaldb::Algorithm::Archive,
			None,
		);

		self.ensure_db_good(BasicBackend(db.as_hash_db_mut()), &factories)
			.map_err(|e| format!("Unable to initialize genesis state: {}", e))?;

		let call = |a, d| {
			let mut db = db.boxed_clone();
			let env_info = ::evm::EnvInfo {
				number: 0,
				author: *genesis.author(),
				timestamp: genesis.timestamp(),
				difficulty: *genesis.difficulty(),
				gas_limit: U256::max_value(),
				last_hashes: Arc::new(Vec::new()),
				gas_used: 0.into(),
			};

			let from = Address::zero();
			let tx = Transaction {
				nonce: self.engine.account_start_nonce(0),
				action: Action::Call(a),
				gas: U256::max_value(),
				gas_price: U256::default(),
				value: U256::default(),
				data: d,
			}.fake_sign(from);

			let res = ::executive_state::prove_transaction_virtual(
				db.as_hash_db_mut(),
				*genesis.state_root(),
				&tx,
				self.engine.machine(),
				&env_info,
				factories.clone(),
			);

			res.map(|(out, proof)| {
				(out, proof.into_iter().map(|x| x.into_vec()).collect())
			}).ok_or_else(|| "Failed to prove call: insufficient state".into())
		};

		self.engine.genesis_epoch_data(&genesis, &call)
	}

	/// Create a new Spec with InstantSeal consensus which does internal sealing (not requiring
	/// work).
	pub fn new_instant() -> Spec {
		load_bundled!("instant_seal")
	}

	/// Create a new Spec which conforms to the Frontier-era Morden chain except that it's a
	/// NullEngine consensus.
	#[cfg(any(test, feature = "test-helpers"))]
	pub fn new_test() -> Spec {
		load_bundled!("null_morden")
	}

	/// Create the Machine corresponding to Spec::new_test.
	#[cfg(any(test, feature = "test-helpers"))]
	pub fn new_test_machine() -> Machine { load_machine_bundled!("null_morden") }

	/// Create a new Spec which conforms to the Frontier-era Morden chain except that it's a NullEngine consensus with applying reward on block close.
	#[cfg(any(test, feature = "test-helpers"))]
	pub fn new_test_with_reward() -> Spec { load_bundled!("null_morden_with_reward") }

	/// Create a new Spec which is a NullEngine consensus with a premine of address whose
	/// secret is keccak('').
	#[cfg(any(test, feature = "test-helpers"))]
	pub fn new_null() -> Spec {
		load_bundled!("null")
	}

	/// Create a new Spec which constructs a contract at address 5 with storage at 0 equal to 1.
	#[cfg(any(test, feature = "test-helpers"))]
	pub fn new_test_constructor() -> Spec {
		load_bundled!("constructor")
	}

	/// Create a new Spec with AuthorityRound consensus which does internal sealing (not
	/// requiring work).
	/// Accounts with secrets keccak("0") and keccak("1") are the validators.
	#[cfg(any(test, feature = "test-helpers"))]
	pub fn new_test_round() -> Self {
		load_bundled!("authority_round")
	}

	/// Create a new Spec with AuthorityRound consensus which does internal sealing (not
	/// requiring work) with empty step messages enabled.
	/// Accounts with secrets keccak("0") and keccak("1") are the validators.
	#[cfg(any(test, feature = "test-helpers"))]
	pub fn new_test_round_empty_steps() -> Self {
		load_bundled!("authority_round_empty_steps")
	}

	/// Create a new Spec with AuthorityRound consensus (with empty steps) using a block reward
	/// contract. The contract source code can be found at:
	/// https://github.com/parity-contracts/block-reward/blob/daf7d44383b6cdb11cb6b953b018648e2b027cfb/contracts/ExampleBlockReward.sol
	#[cfg(any(test, feature = "test-helpers"))]
	pub fn new_test_round_block_reward_contract() -> Self {
		load_bundled!("authority_round_block_reward_contract")
	}

	/// TestList.sol used in both specs: https://github.com/paritytech/contracts/pull/30/files
	/// Accounts with secrets keccak("0") and keccak("1") are initially the validators.
	/// Create a new Spec with BasicAuthority which uses a contract at address 5 to determine
	/// the current validators using `getValidators`.
	/// Second validator can be removed with
	/// "0xbfc708a000000000000000000000000082a978b3f5962a5b0957d9ee9eef472ee55b42f1" and added
	/// back in using
	/// "0x4d238c8e00000000000000000000000082a978b3f5962a5b0957d9ee9eef472ee55b42f1".
	#[cfg(any(test, feature = "test-helpers"))]
	pub fn new_validator_safe_contract() -> Self {
		load_bundled!("validator_safe_contract")
	}

	/// The same as the `safeContract`, but allows reporting and uses AuthorityRound.
	/// Account is marked with `reportBenign` it can be checked as disliked with "0xd8f2e0bf".
	/// Validator can be removed with `reportMalicious`.
	#[cfg(any(test, feature = "test-helpers"))]
	pub fn new_validator_contract() -> Self {
		load_bundled!("validator_contract")
	}

	/// Create a new Spec with BasicAuthority which uses multiple validator sets changing with
	/// height.
	/// Account with secrets keccak("0") is the validator for block 1 and with keccak("1")
	/// onwards.
	#[cfg(any(test, feature = "test-helpers"))]
	pub fn new_validator_multi() -> Self {
		load_bundled!("validator_multi")
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use account_state::State;
	use test_helpers::get_temp_state_db;
	use tempdir::TempDir;
	use types::view;
	use types::views::BlockView;
	use std::str::FromStr;

	#[test]
	fn test_load_empty() {
		let tempdir = TempDir::new("").unwrap();
		assert!(Spec::load(&tempdir.path(), &[] as &[u8]).is_err());
	}

	#[test]
	fn test_chain() {
		let test_spec = Spec::new_test();

		assert_eq!(
			test_spec.state_root(),
			H256::from_str("f3f4696bbf3b3b07775128eb7a3763279a394e382130f27c21e70233e04946a9").unwrap()
		);
		let genesis = test_spec.genesis_block();
		assert_eq!(
			view!(BlockView, &genesis).header_view().hash(),
			H256::from_str("0cd786a2425d16f152c658316c423e6ce1181e15c3295826d7c9904cba9ce303").unwrap()
		);
	}

	#[test]
	fn genesis_constructor() {
		let _ = ::env_logger::try_init();
		let spec = Spec::new_test_constructor();
		let db = spec.ensure_db_good(get_temp_state_db(), &Default::default())
			.unwrap();
		let state = State::from_existing(
			db.boxed_clone(),
			spec.state_root(),
			spec.engine.account_start_nonce(0),
			Default::default(),
		).unwrap();
		let expected = H256::from_str("0000000000000000000000000000000000000000000000000000000000000001").unwrap();
		let address = Address::from_str("0000000000000000000000000000000000001337").unwrap();

		assert_eq!(state.storage_at(&address, &H256::zero()).unwrap(), expected);
		assert_eq!(state.balance(&address).unwrap(), 1.into());
	}
}
