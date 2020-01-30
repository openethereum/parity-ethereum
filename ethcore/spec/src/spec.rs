// Copyright 2015-2020 Parity Technologies (UK) Ltd.
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

use std::{
	collections::BTreeMap,
	convert::TryFrom,
	fmt,
	io::Read,
	path::Path,
	sync::Arc,
};

use common_types::{
	BlockNumber,
	header::Header,
	encoded,
	engines::{OptimizeFor, params::CommonParams},
	errors::EthcoreError as Error,
	transaction::{Action, Transaction},
};
use account_state::{Backend, State, backend::Basic as BasicBackend};
use authority_round::AuthorityRound;
use basic_authority::BasicAuthority;
use bytes::Bytes;
use builtin::Builtin;
use clique::Clique;
use engine::Engine;
use ethash_engine::Ethash;
use ethereum_types::{H256, Bloom, U256, Address};
use ethjson;
use instant_seal::{InstantSeal, InstantSealParams};
use keccak_hash::{KECCAK_NULL_RLP, keccak};
use log::{trace, warn};
use machine::{executive::Executive, Machine, substate::Substate};
use null_engine::NullEngine;
use pod::PodState;
use rlp::{Rlp, RlpStream};
use trace::{NoopTracer, NoopVMTracer};
use trie_vm_factories::Factories;
use vm::{EnvInfo, CallType, ActionValue, ActionParams, ParamsType};

use crate::{
	Genesis,
	seal::Generic as GenericSeal,
};

/// Runtime parameters for the spec that are related to how the software should run the chain,
/// rather than integral properties of the chain itself.
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

/// given a pre-constructor state, run all the given constructors and produce a new state and
/// state root.
fn run_constructors<T: Backend>(
	genesis_state: &PodState,
	constructors: &[(Address, Bytes)],
	engine: &dyn Engine,
	author: Address,
	timestamp: u64,
	difficulty: U256,
	factories: &Factories,
	mut db: T
) -> Result<(H256, T), Error> {
	let mut root = KECCAK_NULL_RLP;

	// basic accounts in spec.
	{
		let mut t = factories.trie.create(db.as_hash_db_mut(), &mut root);

		for (address, account) in genesis_state.get().iter() {
			t.insert(address.as_bytes(), &account.rlp())?;
		}
	}

	for (address, account) in genesis_state.get().iter() {
		db.note_non_null_account(address);
		account.insert_additional(
			&mut *factories.accountdb.create(
				db.as_hash_db_mut(),
				keccak(address),
			),
			&factories.trie,
		);
	}

	let start_nonce = engine.account_start_nonce(0);

	let mut state = State::from_existing(db, root, start_nonce, factories.clone())?;
	if constructors.is_empty() {
		state.populate_from(genesis_state.clone());
		let _ = state.commit()?;
	} else {
		// Execute contract constructors.
		let env_info = EnvInfo {
			number: 0,
			author,
			timestamp,
			difficulty,
			last_hashes: Default::default(),
			gas_used: U256::zero(),
			gas_limit: U256::max_value(),
		};

		let from = Address::zero();
		for &(ref address, ref constructor) in constructors.iter() {
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
				let machine = engine.machine();
				let schedule = machine.schedule(env_info.number);
				let mut exec = Executive::new(&mut state, &env_info, &machine, &schedule);
				// failing create is not a bug
				if let Err(e) = exec.create(params, &mut substate, &mut NoopTracer, &mut NoopVMTracer) {
					warn!(target: "spec", "Genesis constructor execution at {} failed: {}.", address, e);
				}
			}

			let _ = state.commit()?;
		}
	}
	Ok(state.drop())
}

/// Parameters for a block chain; includes both those intrinsic to the design of the
/// chain and those to be interpreted by the active chain engine.
pub struct Spec {
	/// User friendly spec name.
	pub name: String,
	/// Engine specified by json file.
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
	pub constructors: Vec<(Address, Bytes)>,
	/// May be pre-populated if we know this in advance.
	pub state_root: H256,
	/// Genesis state as plain old data.
	pub genesis_state: PodState,
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

impl From<ethjson::spec::HardcodedSync> for SpecHardcodedSync {
	fn from(sync: ethjson::spec::HardcodedSync) -> Self {
		SpecHardcodedSync {
			header: encoded::Header::new(sync.header.into()),
			total_difficulty: sync.total_difficulty.into(),
			chts: sync.chts.into_iter().map(Into::into).collect(),
		}
	}
}

impl fmt::Display for SpecHardcodedSync {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		writeln!(f, "{{")?;
		writeln!(f, r#"header": "{:?},"#, self.header)?;
		writeln!(f, r#"total_difficulty": "{:?},"#, self.total_difficulty)?;
		writeln!(f, r#"chts": {:#?}"#, self.chts.iter().map(|x| format!(r#"{:?}"#, x)).collect::<Vec<_>>())?;
		writeln!(f, "}}")
	}
}

fn convert_json_to_spec(
	(address, builtin): (ethjson::hash::Address, ethjson::spec::builtin::Builtin),
) -> Result<(Address, Builtin), Error> {
	let builtin = Builtin::try_from(builtin)?;
	Ok((address.into(), builtin))
}

/// Load from JSON object.
fn load_from(spec_params: SpecParams, s: ethjson::spec::Spec) -> Result<Spec, Error> {
	let builtins: Result<BTreeMap<Address, Builtin>, _> = s
		.accounts
		.builtins()
		.into_iter()
		.map(convert_json_to_spec)
		.collect();
	let builtins = builtins?;
	let g = Genesis::from(s.genesis);
	let GenericSeal(seal_rlp) = g.seal.into();
	let params = CommonParams::from(s.params);

	let hardcoded_sync = s.hardcoded_sync.map(Into::into);

	let engine = Spec::engine(spec_params, s.engine, params, builtins);
	let author = g.author;
	let timestamp = g.timestamp;
	let difficulty = g.difficulty;
	let constructors: Vec<_> = s.accounts
		.constructors()
		.into_iter()
		.map(|(a, c)| (a.into(), c.into()))
		.collect();
	let genesis_state: PodState = s.accounts.into();

	let (state_root, _) = run_constructors(
		&genesis_state,
		&constructors,
		&*engine,
		author,
		timestamp,
		difficulty,
		&Default::default(),
		BasicBackend(journaldb::new_memory_db()),
	)?;

	let s = Spec {
		engine,
		name: s.name.clone().into(),
		data_dir: s.data_dir.unwrap_or(s.name).into(),
		nodes: s.nodes.unwrap_or_else(Vec::new),
		parent_hash: g.parent_hash,
		transactions_root: g.transactions_root,
		receipts_root: g.receipts_root,
		author,
		difficulty,
		gas_limit: g.gas_limit,
		gas_used: g.gas_used,
		timestamp,
		extra_data: g.extra_data,
		seal_rlp,
		hardcoded_sync,
		constructors,
		genesis_state,
		state_root,
	};

	Ok(s)
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

	/// Get common blockchain parameters.
	pub fn params(&self) -> &CommonParams {
		&self.engine.params()
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
		header.set_state_root(self.state_root);
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
		let (root, _) = run_constructors(
			&self.genesis_state,
			&self.constructors,
			&*self.engine,
			self.author,
			self.timestamp,
			self.difficulty,
			&Default::default(),
			BasicBackend(journaldb::new_memory_db()),
		)?;

		self.state_root = root;
		Ok(())
	}

	/// Ensure that the given state DB has the trie nodes in for the genesis state.
	pub fn ensure_db_good<T: Backend>(&self, db: T, factories: &Factories) -> Result<T, Error> {
		if db.as_hash_db().contains(&self.state_root, hash_db::EMPTY_PREFIX) {
			return Ok(db);
		}

		// TODO: could optimize so we don't re-run, but `ensure_db_good` is barely ever
		// called anyway.
		let (root, db) = run_constructors(
			&self.genesis_state,
			&self.constructors,
			&*self.engine,
			self.author,
			self.timestamp,
			self.difficulty,
			factories,
			db
		)?;

		assert_eq!(root, self.state_root, "Spec's state root has not been precomputed correctly.");
		Ok(db)
	}

	/// Loads just the state machine from a json file.
	pub fn load_machine<R: Read>(reader: R) -> Result<Machine, Error> {
		ethjson::spec::Spec::load(reader)
			.map_err(|e| Error::Msg(e.to_string()))
			.and_then(|s| {
				let builtins: Result<BTreeMap<Address, Builtin>, _> = s
					.accounts
					.builtins()
					.into_iter()
					.map(convert_json_to_spec)
					.collect();
				let builtins = builtins?;
				let params = CommonParams::from(s.params);
				Ok(Spec::machine(&s.engine, params, builtins))
			})
	}

	/// Loads spec from json file. Provide factories for executing contracts and ensuring
	/// storage goes to the right place.
	pub fn load<'a, T: Into<SpecParams<'a>>, R: Read>(params: T, reader: R) -> Result<Self, Error> {
		ethjson::spec::Spec::load(reader)
			.map_err(|e| Error::Msg(e.to_string()))
			.and_then(|x| load_from(params.into(), x))
	}

	/// initialize genesis epoch data, using in-memory database for
	/// constructor.
	pub fn genesis_epoch_data(&self) -> Result<Vec<u8>, String> {
		let genesis = self.genesis_header();

		let factories = Default::default();
		let mut db = journaldb::new(
			Arc::new(kvdb_memorydb::create(1)),
			journaldb::Algorithm::Archive,
			0,
		);

		self.ensure_db_good(BasicBackend(db.as_hash_db_mut()), &factories)
			.map_err(|e| format!("Unable to initialize genesis state: {}", e))?;

		let call = |a, d| {
			let mut db = db.boxed_clone();
			let env_info = evm::EnvInfo {
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

			executive_state::prove_transaction_virtual(
				db.as_hash_db_mut(),
				*genesis.state_root(),
				&tx,
				self.engine.machine(),
				&env_info,
				factories.clone(),
			).ok_or_else(|| "Failed to prove call: insufficient state".into())
		};

		self.engine.genesis_epoch_data(&genesis, &call)
	}
}

#[cfg(test)]
mod tests {
	use std::str::FromStr;

	use account_state::State;
	use common_types::{view, views::BlockView};
	use ethereum_types::{Address, H256};
	use ethcore::test_helpers::get_temp_state_db;
	use tempdir::TempDir;

	use super::Spec;

	#[test]
	fn test_load_empty() {
		let tempdir = TempDir::new("").unwrap();
		assert!(Spec::load(&tempdir.path(), &[] as &[u8]).is_err());
	}

	#[test]
	fn test_chain() {
		let test_spec = crate::new_test();

		assert_eq!(
			test_spec.state_root,
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
		let spec = crate::new_test_constructor();
		let db = spec.ensure_db_good(get_temp_state_db(), &Default::default())
			.unwrap();
		let state = State::from_existing(
			db.boxed_clone(),
			spec.state_root,
			spec.engine.account_start_nonce(0),
			Default::default(),
		).unwrap();
		let expected = H256::from_str("0000000000000000000000000000000000000000000000000000000000000001").unwrap();
		let address = Address::from_str("0000000000000000000000000000000000001337").unwrap();

		assert_eq!(state.storage_at(&address, &H256::zero()).unwrap(), expected);
		assert_eq!(state.balance(&address).unwrap(), 1.into());
	}
}
