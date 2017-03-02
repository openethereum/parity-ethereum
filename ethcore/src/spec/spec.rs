// Copyright 2015-2017 Parity Technologies (UK) Ltd.
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

//! Parameters for a block chain.

use util::*;
use builtin::Builtin;
use engines::{Engine, NullEngine, InstantSeal, BasicAuthority, AuthorityRound, Tendermint};
use factory::Factories;
use executive::Executive;
use trace::{NoopTracer, NoopVMTracer};
use action_params::{ActionValue, ActionParams};
use types::executed::CallType;
use state::{Backend, State, Substate};
use env_info::EnvInfo;
use pod_state::*;
use account_db::*;
use header::{BlockNumber, Header};
use state_db::StateDB;
use super::genesis::Genesis;
use super::seal::Generic as GenericSeal;
use ethereum;
use ethjson;
use rlp::{Rlp, RlpStream, View, Stream};

/// Parameters common to all engines.
#[derive(Debug, PartialEq, Clone, Default)]
pub struct CommonParams {
	/// Account start nonce.
	pub account_start_nonce: U256,
	/// Maximum size of extra data.
	pub maximum_extra_data_size: usize,
	/// Network id.
	pub network_id: u64,
	/// Chain id.
	pub chain_id: u64,
	/// Main subprotocol name.
	pub subprotocol_name: String,
	/// Minimum gas limit.
	pub min_gas_limit: U256,
	/// Fork block to check.
	pub fork_block: Option<(BlockNumber, H256)>,
	/// Number of first block where EIP-98 rules begin.
	pub eip98_transition: BlockNumber,
}

impl From<ethjson::spec::Params> for CommonParams {
	fn from(p: ethjson::spec::Params) -> Self {
		CommonParams {
			account_start_nonce: p.account_start_nonce.map_or_else(U256::zero, Into::into),
			maximum_extra_data_size: p.maximum_extra_data_size.into(),
			network_id: p.network_id.into(),
			chain_id: if let Some(n) = p.chain_id { n.into() } else { p.network_id.into() },
			subprotocol_name: p.subprotocol_name.unwrap_or_else(|| "eth".to_owned()),
			min_gas_limit: p.min_gas_limit.into(),
			fork_block: if let (Some(n), Some(h)) = (p.fork_block, p.fork_hash) { Some((n.into(), h.into())) } else { None },
			eip98_transition: p.eip98_transition.map_or(0, Into::into),
		}
	}
}

/// Parameters for a block chain; includes both those intrinsic to the design of the
/// chain and those to be interpreted by the active chain engine.
pub struct Spec {
	/// User friendly spec name
	pub name: String,
	/// What engine are we using for this?
	pub engine: Arc<Engine>,
	/// Name of the subdir inside the main data dir to use for chain data and settings.
	pub data_dir: String,

	/// Known nodes on the network in enode format.
	pub nodes: Vec<String>,

	/// Parameters common to all engines.
	pub params: CommonParams,

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
	/// Transactions root of the genesis block. Should be SHA3_NULL_RLP.
	pub transactions_root: H256,
	/// Receipts root of the genesis block. Should be SHA3_NULL_RLP.
	pub receipts_root: H256,
	/// The genesis block's extra data field.
	pub extra_data: Bytes,
	/// Each seal field, expressed as RLP, concatenated.
	pub seal_rlp: Bytes,

	/// Contract constructors to be executed on genesis.
	constructors: Vec<(Address, Bytes)>,

	/// May be prepopulated if we know this in advance.
	state_root_memo: RwLock<Option<H256>>,

	/// Genesis state as plain old data.
	genesis_state: PodState,
}

impl From<ethjson::spec::Spec> for Spec {
	fn from(s: ethjson::spec::Spec) -> Self {
		let builtins = s.accounts.builtins().into_iter().map(|p| (p.0.into(), From::from(p.1))).collect();
		let g = Genesis::from(s.genesis);
		let GenericSeal(seal_rlp) = g.seal.into();
		let params = CommonParams::from(s.params);
		Spec {
			name: s.name.clone().into(),
			params: params.clone(),
			engine: Spec::engine(s.engine, params, builtins),
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
			seal_rlp: seal_rlp,
			constructors: s.accounts.constructors().into_iter().map(|(a, c)| (a.into(), c.into())).collect(),
			state_root_memo: RwLock::new(g.state_root),
			genesis_state: From::from(s.accounts),
		}
	}
}

macro_rules! load_bundled {
	($e:expr) => {
		Spec::load(include_bytes!(concat!("../../res/", $e, ".json")) as &[u8]).expect(concat!("Chain spec ", $e, " is invalid."))
	};
}

impl Spec {
	/// Convert engine spec into a arc'd Engine of the right underlying type.
	/// TODO avoid this hard-coded nastiness - use dynamic-linked plugin framework instead.
	fn engine(engine_spec: ethjson::spec::Engine, params: CommonParams, builtins: BTreeMap<Address, Builtin>) -> Arc<Engine> {
		match engine_spec {
			ethjson::spec::Engine::Null => Arc::new(NullEngine::new(params, builtins)),
			ethjson::spec::Engine::InstantSeal(instant) => Arc::new(InstantSeal::new(params, instant.params.registrar.map_or_else(Address::new, Into::into), builtins)),
			ethjson::spec::Engine::Ethash(ethash) => Arc::new(ethereum::Ethash::new(params, From::from(ethash.params), builtins)),
			ethjson::spec::Engine::BasicAuthority(basic_authority) => Arc::new(BasicAuthority::new(params, From::from(basic_authority.params), builtins)),
			ethjson::spec::Engine::AuthorityRound(authority_round) => AuthorityRound::new(params, From::from(authority_round.params), builtins).expect("Failed to start AuthorityRound consensus engine."),
			ethjson::spec::Engine::Tendermint(tendermint) => Tendermint::new(params, From::from(tendermint.params), builtins).expect("Failed to start the Tendermint consensus engine."),
		}
	}

	/// Return the state root for the genesis state, memoising accordingly.
	pub fn state_root(&self) -> H256 {
		if self.state_root_memo.read().is_none() {
			*self.state_root_memo.write() = Some(self.genesis_state.root());
		}
		self.state_root_memo.read().as_ref().cloned()
			.expect("state root memo ensured to be set at this point; qed")
	}

	/// Get the known knodes of the network in enode format.
	pub fn nodes(&self) -> &[String] { &self.nodes }

	/// Get the configured Network ID.
	pub fn network_id(&self) -> u64 { self.params.network_id }

	/// Get the configured subprotocol name.
	pub fn subprotocol_name(&self) -> String { self.params.subprotocol_name.clone() }

	/// Get the configured network fork block.
	pub fn fork_block(&self) -> Option<(BlockNumber, H256)> { self.params.fork_block }

	/// Get the header of the genesis block.
	pub fn genesis_header(&self) -> Header {
		let mut header: Header = Default::default();
		header.set_parent_hash(self.parent_hash.clone());
		header.set_timestamp(self.timestamp);
		header.set_number(0);
		header.set_author(self.author.clone());
		header.set_transactions_root(self.transactions_root.clone());
		header.set_uncles_hash(RlpStream::new_list(0).out().sha3());
		header.set_extra_data(self.extra_data.clone());
		header.set_state_root(self.state_root());
		header.set_receipts_root(self.receipts_root.clone());
		header.set_log_bloom(H2048::new().clone());
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
		self.state_root_memo = RwLock::new(g.state_root);
	}

	/// Alter the value of the genesis state.
	pub fn set_genesis_state(&mut self, s: PodState) {
		self.genesis_state = s;
		*self.state_root_memo.write() = None;
	}

	/// Returns `false` if the memoized state root is invalid. `true` otherwise.
	pub fn is_state_root_valid(&self) -> bool {
		self.state_root_memo.read().clone().map_or(true, |sr| sr == self.genesis_state.root())
	}

	/// Ensure that the given state DB has the trie nodes in for the genesis state.
	pub fn ensure_db_good(&self, mut db: StateDB, factories: &Factories) -> Result<StateDB, Box<TrieError>> {
		if db.as_hashdb().contains(&self.state_root()) {
			return Ok(db)
		}
		trace!(target: "spec", "ensure_db_good: Fresh database? Cannot find state root {}", self.state_root());
		let mut root = H256::new();

		{
			let mut t = factories.trie.create(db.as_hashdb_mut(), &mut root);
			for (address, account) in self.genesis_state.get().iter() {
				t.insert(&**address, &account.rlp())?;
			}
		}

		trace!(target: "spec", "ensure_db_good: Populated sec trie; root is {}", root);
		for (address, account) in self.genesis_state.get().iter() {
			db.note_non_null_account(address);
			account.insert_additional(&mut AccountDBMut::new(db.as_hashdb_mut(), address), &factories.trie);
		}

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
		let from = Address::default();
		let start_nonce = self.engine.account_start_nonce();

		let mut state = State::from_existing(db, root, start_nonce, factories.clone())?;
		// Mutate the state with each constructor.
		for &(ref address, ref constructor) in self.constructors.iter() {
			trace!(target: "spec", "ensure_db_good: Creating a contract at {}.", address);
			let params = ActionParams {
				code_address: address.clone(),
				code_hash: constructor.sha3(),
				address: address.clone(),
				sender: from.clone(),
				origin: from.clone(),
				gas: U256::max_value(),
				gas_price: Default::default(),
				value: ActionValue::Transfer(Default::default()),
				code: Some(Arc::new(constructor.clone())),
				data: None,
				call_type: CallType::None,
			};
			let mut substate = Substate::new();
			{
				let mut exec = Executive::new(&mut state, &env_info, self.engine.as_ref(), &factories.vm);
				if let Err(e) = exec.create(params, &mut substate, &mut NoopTracer, &mut NoopVMTracer) {
					warn!(target: "spec", "Genesis constructor execution at {} failed: {}.", address, e);
				}
			}
			if let Err(e) = state.commit() {
				warn!(target: "spec", "Genesis constructor trie commit at {} failed: {}.", address, e);
			}
		}
		let (root, db) = state.drop();

		*self.state_root_memo.write() = Some(root);
		Ok(db)
	}

	/// Loads spec from json file.
	pub fn load<R>(reader: R) -> Result<Self, String> where R: Read {
		match ethjson::spec::Spec::load(reader) {
			Ok(spec) => Ok(spec.into()),
			_ => Err("Spec json is invalid".into()),
		}
	}

	/// Create a new Spec which conforms to the Frontier-era Morden chain except that it's a NullEngine consensus.
	pub fn new_test() -> Spec { load_bundled!("null_morden") }

	/// Create a new Spec which is a NullEngine consensus with a premine of address whose secret is sha3('').
	pub fn new_null() -> Spec { load_bundled!("null") }

	/// Create a new Spec which constructs a contract at address 5 with storage at 0 equal to 1.
	pub fn new_test_constructor() -> Spec { load_bundled!("constructor") }

	/// Create a new Spec with InstantSeal consensus which does internal sealing (not requiring work).
	pub fn new_instant() -> Spec { load_bundled!("instant_seal") }

	/// Create a new Spec with AuthorityRound consensus which does internal sealing (not requiring work).
	/// Accounts with secrets "0".sha3() and "1".sha3() are the validators.
	pub fn new_test_round() -> Self { load_bundled!("authority_round") }

	/// Create a new Spec with Tendermint consensus which does internal sealing (not requiring work).
	/// Account "0".sha3() and "1".sha3() are a authorities.
	pub fn new_test_tendermint() -> Self { load_bundled!("tendermint") }

	/// TestList.sol used in both specs: https://github.com/ethcore/contracts/pull/30/files
	/// Accounts with secrets "0".sha3() and "1".sha3() are initially the validators.
	/// Create a new Spec with BasicAuthority which uses a contract at address 5 to determine the current validators using `getValidators`.
	/// Second validator can be removed with "0xbfc708a000000000000000000000000082a978b3f5962a5b0957d9ee9eef472ee55b42f1" and added back in using "0x4d238c8e00000000000000000000000082a978b3f5962a5b0957d9ee9eef472ee55b42f1".
	pub fn new_validator_safe_contract() -> Self { load_bundled!("validator_safe_contract") }

	/// The same as the `safeContract`, but allows reporting and uses AuthorityRound.
	/// Account is marked with `reportBenign` it can be checked as disliked with "0xd8f2e0bf".
	/// Validator can be removed with `reportMalicious`.
	pub fn new_validator_contract() -> Self { load_bundled!("validator_contract") }
}

#[cfg(test)]
mod tests {
	use util::*;
	use views::*;
	use tests::helpers::get_temp_state_db;
	use state::State;
	use super::*;

	// https://github.com/ethcore/parity/issues/1840
	#[test]
	fn test_load_empty() {
		assert!(Spec::load(&[] as &[u8]).is_err());
	}

	#[test]
	fn test_chain() {
		let test_spec = Spec::new_test();

		assert_eq!(test_spec.state_root(), H256::from_str("f3f4696bbf3b3b07775128eb7a3763279a394e382130f27c21e70233e04946a9").unwrap());
		let genesis = test_spec.genesis_block();
		assert_eq!(BlockView::new(&genesis).header_view().sha3(), H256::from_str("0cd786a2425d16f152c658316c423e6ce1181e15c3295826d7c9904cba9ce303").unwrap());
	}

	#[test]
	fn genesis_constructor() {
		let spec = Spec::new_test_constructor();
		let mut db_result = get_temp_state_db();
		let db = spec.ensure_db_good(db_result.take(), &Default::default()).unwrap();
		let state = State::from_existing(db.boxed_clone(), spec.state_root(), spec.engine.account_start_nonce(), Default::default()).unwrap();
		let expected = H256::from_str("0000000000000000000000000000000000000000000000000000000000000001").unwrap();
		assert_eq!(state.storage_at(&Address::from_str("0000000000000000000000000000000000000005").unwrap(), &H256::zero()).unwrap(), expected);
	}
}
