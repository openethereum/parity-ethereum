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

//! Parameters for a block chain.

use common::*;
use engine::*;
use pod_state::*;
use null_engine::*;
use account_db::*;
use super::genesis::Genesis;
use super::seal::Generic as GenericSeal;
use ethereum;
use ethjson;

/// Parameters common to all engines.
#[derive(Debug, PartialEq, Clone)]
pub struct CommonParams {
	/// Account start nonce.
	pub account_start_nonce: U256,
	/// Frontier compatibility mode limit.
	pub frontier_compatibility_mode_limit: u64,
	/// Maximum size of extra data.
	pub maximum_extra_data_size: usize,
	/// Network id.
	pub network_id: U256,
	/// Minimum gas limit.
	pub min_gas_limit: U256,
}

impl From<ethjson::spec::Params> for CommonParams {
	fn from(p: ethjson::spec::Params) -> Self {
		CommonParams {
			account_start_nonce: p.account_start_nonce.into(),
			frontier_compatibility_mode_limit: p.frontier_compatibility_mode_limit.into(),
			maximum_extra_data_size: p.maximum_extra_data_size.into(),
			network_id: p.network_id.into(),
			min_gas_limit: p.min_gas_limit.into(),
		}
	}
}

/// Parameters for a block chain; includes both those intrinsic to the design of the
/// chain and those to be interpreted by the active chain engine.
pub struct Spec {
	/// User friendly spec name
	pub name: String,
	/// What engine are we using for this?
	pub engine: Box<Engine>,

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
	/// The number of seal fields in the genesis block.
	pub seal_fields: usize,
	/// Each seal field, expressed as RLP, concatenated.
	pub seal_rlp: Bytes,

	// May be prepopulated if we know this in advance.
	state_root_memo: RwLock<Option<H256>>,

	// Genesis state as plain old data.
	genesis_state: PodState,
}

impl From<ethjson::spec::Spec> for Spec {
	fn from(s: ethjson::spec::Spec) -> Self {
		let builtins = s.accounts.builtins().into_iter().map(|p| (p.0.into(), From::from(p.1))).collect();
		let g = Genesis::from(s.genesis);
		let seal: GenericSeal = g.seal.into();
		let params = CommonParams::from(s.params);
		Spec {
			name: s.name.into(),
			params: params.clone(),
			engine: Spec::engine(s.engine, params, builtins),
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
			seal_fields: seal.fields,
			seal_rlp: seal.rlp,
			state_root_memo: RwLock::new(g.state_root),
			genesis_state: From::from(s.accounts)
		}
	}
}

impl Spec {
	/// Convert engine spec into a boxed Engine of the right underlying type.
	/// TODO avoid this hard-coded nastiness - use dynamic-linked plugin framework instead.
	fn engine(engine_spec: ethjson::spec::Engine, params: CommonParams, builtins: BTreeMap<Address, Builtin>) -> Box<Engine> {
		match engine_spec {
			ethjson::spec::Engine::Null => Box::new(NullEngine::new(params, builtins)),
			ethjson::spec::Engine::Ethash(ethash) => Box::new(ethereum::Ethash::new(params, From::from(ethash.params), builtins))
		}
	}

	/// Return the state root for the genesis state, memoising accordingly.
	pub fn state_root(&self) -> H256 {
		if self.state_root_memo.read().unwrap().is_none() {
			*self.state_root_memo.write().unwrap() = Some(self.genesis_state.root());
		}
		self.state_root_memo.read().unwrap().as_ref().unwrap().clone()
	}

	/// Get the known knodes of the network in enode format.
	pub fn nodes(&self) -> &Vec<String> { &self.nodes }

	/// Get the configured Network ID.
	pub fn network_id(&self) -> U256 { self.params.network_id }

	/// Get the header of the genesis block.
	pub fn genesis_header(&self) -> Header {
		Header {
			parent_hash: self.parent_hash.clone(),
			timestamp: self.timestamp,
			number: 0,
			author: self.author.clone(),
			transactions_root: self.transactions_root.clone(),
			uncles_hash: RlpStream::new_list(0).out().sha3(),
			extra_data: self.extra_data.clone(),
			state_root: self.state_root().clone(),
			receipts_root: self.receipts_root.clone(),
			log_bloom: H2048::new().clone(),
			gas_used: self.gas_used.clone(),
			gas_limit: self.gas_limit.clone(),
			difficulty: self.difficulty.clone(),
			seal: {
				let seal = {
					let mut s = RlpStream::new_list(self.seal_fields);
					s.append_raw(&self.seal_rlp, self.seal_fields);
					s.out()
				};
				let r = Rlp::new(&seal);
				(0..self.seal_fields).map(|i| r.at(i).as_raw().to_vec()).collect()
			},
			hash: RefCell::new(None),
			bare_hash: RefCell::new(None),
		}
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
		let seal: GenericSeal = g.seal.into();
		self.parent_hash = g.parent_hash;
		self.transactions_root = g.transactions_root;
		self.receipts_root = g.receipts_root;
		self.author = g.author;
		self.difficulty = g.difficulty;
		self.gas_limit = g.gas_limit;
		self.gas_used = g.gas_used;
		self.timestamp = g.timestamp;
		self.extra_data = g.extra_data;
		self.seal_fields = seal.fields;
		self.seal_rlp = seal.rlp;
		self.state_root_memo = RwLock::new(g.state_root);
	}

	/// Alter the value of the genesis state.
	pub fn set_genesis_state(&mut self, s: PodState) {
		self.genesis_state = s;
		*self.state_root_memo.write().unwrap() = None;
	}

	/// Returns `false` if the memoized state root is invalid. `true` otherwise.
	pub fn is_state_root_valid(&self) -> bool {
		self.state_root_memo.read().unwrap().clone().map_or(true, |sr| sr == self.genesis_state.root())
	}

	/// Ensure that the given state DB has the trie nodes in for the genesis state.
	pub fn ensure_db_good(&self, db: &mut HashDB) -> bool {
		if !db.contains(&self.state_root()) {
			let mut root = H256::new();
			{
				let mut t = SecTrieDBMut::new(db, &mut root);
				for (address, account) in self.genesis_state.get().iter() {
					t.insert(address.as_slice(), &account.rlp());
				}
			}
			for (address, account) in self.genesis_state.get().iter() {
				account.insert_additional(&mut AccountDBMut::new(db, address));
			}
			assert!(db.contains(&self.state_root()));
			true
		} else { false }
	}

	/// Loads spec from json file.
	pub fn load(reader: &[u8]) -> Self {
		From::from(ethjson::spec::Spec::load(reader).expect("invalid json file"))
	}

	/// Create a new Spec which conforms to the Morden chain except that it's a NullEngine consensus.
	pub fn new_test() -> Spec {
		Spec::load(include_bytes!("../../res/null_morden.json"))
	}

	/// Create a new Spec which conforms to the Morden chain except that it's a NullEngine consensus.
	pub fn new_homestead_test() -> Spec {
		Spec::load(include_bytes!("../../res/null_homestead_morden.json"))
	}
}

#[cfg(test)]
mod tests {
	use std::str::FromStr;
	use util::hash::*;
	use util::sha3::*;
	use views::*;
	use super::*;

	#[test]
	fn test_chain() {
		let test_spec = Spec::new_test();

		assert_eq!(test_spec.state_root(), H256::from_str("f3f4696bbf3b3b07775128eb7a3763279a394e382130f27c21e70233e04946a9").unwrap());
		let genesis = test_spec.genesis_block();
		assert_eq!(BlockView::new(&genesis).header_view().sha3(), H256::from_str("0cd786a2425d16f152c658316c423e6ce1181e15c3295826d7c9904cba9ce303").unwrap());
	}
}
