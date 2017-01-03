// Copyright 2015, 2016 Parity Technologies (UK) Ltd.
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

/// Validator set maintained in a contract.

use std::sync::Weak;
use util::*;
use client::{Client, BlockChainClient};
use client::chain_notify::ChainNotify;
use super::ValidatorSet;
use super::simple_list::SimpleList;

/// The validator contract should have the following interface:
/// [{"constant":true,"inputs":[],"name":"getValidators","outputs":[{"name":"","type":"address[]"}],"payable":false,"type":"function"}]
pub struct ValidatorContract {
	address: Address,
	validators: RwLock<SimpleList>,
	provider: RwLock<Option<provider::Contract>>,
}

impl ValidatorContract {
	pub fn new(contract_address: Address) -> Self {
		ValidatorContract {
			address: contract_address,
			validators: Default::default(),
			provider: RwLock::new(None),
		}
	}

	/// Queries the state and updates the set of validators.
	pub fn update(&self) {
		if let Some(ref provider) = *self.provider.read() {
			match provider.get_validators() {
				Ok(new) => {
					debug!(target: "engine", "Set of validators obtained: {:?}", new);
					*self.validators.write() = SimpleList::new(new);
				},
				Err(s) => warn!(target: "engine", "Set of validators could not be updated: {}", s),
			}
		} else {
			warn!(target: "engine", "Set of validators could not be updated: no provider contract.")
		}
	}
}

/// Checks validators on every block.
impl ChainNotify for ValidatorContract {
	fn new_blocks(
		&self,
		_imported: Vec<H256>,
		_: Vec<H256>,
		_: Vec<H256>,
		_: Vec<H256>,
		_: Vec<H256>,
		_: Vec<Bytes>,
		_duration: u64) {
		self.update();
	}
}

impl ValidatorSet for Arc<ValidatorContract> {
	fn contains(&self, address: &Address) -> bool {
		self.validators.read().contains(address)
	}

	fn get(&self, nonce: usize) -> Address {
		self.validators.read().get(nonce).clone()
	}

	fn count(&self) -> usize {
		self.validators.read().count()
	}

	fn register_call_contract(&self, client: Weak<Client>) {
		if let Some(c) = client.upgrade() {
			c.add_notify(self.clone());
		}
		{
			*self.provider.write() = Some(provider::Contract::new(self.address, move |a, d| client.upgrade().ok_or("No client!".into()).and_then(|c| c.call_contract(a, d))));
		}
		self.update();
	}
}

mod provider {
	// Autogenerated from JSON contract definition using Rust contract convertor.
	#![allow(unused_imports)]
	use std::string::String;
	use std::result::Result;
	use std::fmt;
	use {util, ethabi};
	use util::{FixedHash, Uint};

	pub struct Contract {
		contract: ethabi::Contract,
		address: util::Address,
		do_call: Box<Fn(util::Address, Vec<u8>) -> Result<Vec<u8>, String> + Send + Sync + 'static>,
	}
	impl Contract {
		pub fn new<F>(address: util::Address, do_call: F) -> Self where F: Fn(util::Address, Vec<u8>) -> Result<Vec<u8>, String> + Send + Sync + 'static {
			Contract {
				contract: ethabi::Contract::new(ethabi::Interface::load(b"[{\"constant\":true,\"inputs\":[],\"name\":\"getValidators\",\"outputs\":[{\"name\":\"\",\"type\":\"address[]\"}],\"payable\":false,\"type\":\"function\"}]").expect("JSON is autogenerated; qed")),
				address: address,
				do_call: Box::new(do_call),
			}
		}
		fn as_string<T: fmt::Debug>(e: T) -> String { format!("{:?}", e) }
		
		/// Auto-generated from: `{"constant":true,"inputs":[],"name":"getValidators","outputs":[{"name":"","type":"address[]"}],"payable":false,"type":"function"}`
		#[allow(dead_code)]
		pub fn get_validators(&self) -> Result<Vec<util::Address>, String> { 
			let call = self.contract.function("getValidators".into()).map_err(Self::as_string)?;
			let data = call.encode_call(
				vec![]
			).map_err(Self::as_string)?;
			let output = call.decode_output((self.do_call)(self.address.clone(), data)?).map_err(Self::as_string)?;
			let mut result = output.into_iter().rev().collect::<Vec<_>>();
			Ok(({ let r = result.pop().ok_or("Invalid return arity")?; let r = r.to_array().and_then(|v| v.into_iter().map(|a| a.to_address()).collect::<Option<Vec<[u8; 20]>>>()).ok_or("Invalid type returned")?; r.into_iter().map(|a| util::Address::from(a)).collect::<Vec<_>>() })) 
		}
	}
}

#[cfg(test)]
mod tests {
	use util::*;
	use spec::Spec;
	use account_provider::AccountProvider;
	use transaction::{Transaction, Action};
	use client::{BlockChainClient, MiningBlockChainClient};
	use miner::MinerService;
	use tests::helpers::generate_dummy_client_with_spec_and_data;
	use super::super::ValidatorSet;
	use super::ValidatorContract;

	#[test]
	fn fetches_validators() {
		let client = generate_dummy_client_with_spec_and_data(Spec::new_validator_contract, 0, 0, &[]);
		let vc = Arc::new(ValidatorContract::new(Address::from_str("0000000000000000000000000000000000000005").unwrap()));
		vc.register_call_contract(Arc::downgrade(&client));
		vc.update();
		assert!(vc.contains(&Address::from_str("7d577a597b2742b498cb5cf0c26cdcd726d39e6e").unwrap()));
		assert!(vc.contains(&Address::from_str("82a978b3f5962a5b0957d9ee9eef472ee55b42f1").unwrap()));
	}

	#[test]
	fn changes_validators() {
		let tap = Arc::new(AccountProvider::transient_provider());
		let v0 = tap.insert_account("1".sha3(), "").unwrap();
		let v1 = tap.insert_account("0".sha3(), "").unwrap();
		let spec_factory = || {
			let spec = Spec::new_validator_contract();
			spec.engine.register_account_provider(tap.clone());
			spec
		};
		let client = generate_dummy_client_with_spec_and_data(spec_factory, 0, 0, &[]);
		client.engine().register_client(Arc::downgrade(&client));
		let validator_contract = Address::from_str("0000000000000000000000000000000000000005").unwrap();

		client.miner().set_engine_signer(v1, "".into()).unwrap();
		// Remove "1" validator.
		let tx = Transaction {
			nonce: 0.into(),
			gas_price: 0.into(),
			gas: 500_000.into(),
			action: Action::Call(validator_contract),
			value: 0.into(),
			data: "f94e18670000000000000000000000000000000000000000000000000000000000000001".from_hex().unwrap(),
		}.sign(&"1".sha3(), None);
		client.miner().import_own_transaction(client.as_ref(), tx.into()).unwrap();
		client.update_sealing();
		assert_eq!(client.chain_info().best_block_number, 1);
		// Add "1" validator back in.
		let tx = Transaction {
			nonce: 1.into(),
			gas_price: 0.into(),
			gas: 500_000.into(),
			action: Action::Call(validator_contract),
			value: 0.into(),
			data: "4d238c8e00000000000000000000000082a978b3f5962a5b0957d9ee9eef472ee55b42f1".from_hex().unwrap(),
		}.sign(&"1".sha3(), None);
		client.miner().import_own_transaction(client.as_ref(), tx.into()).unwrap();
		client.update_sealing();
		// The transaction is not yet included so still unable to seal.
		assert_eq!(client.chain_info().best_block_number, 1);

		// Switch to the validator that is still there.
		client.miner().set_engine_signer(v0, "".into()).unwrap();
		client.update_sealing();
		assert_eq!(client.chain_info().best_block_number, 2);
		// Switch back to the added validator, since the state is updated.
		client.miner().set_engine_signer(v1, "".into()).unwrap();
		let tx = Transaction {
			nonce: 2.into(),
			gas_price: 0.into(),
			gas: 21000.into(),
			action: Action::Call(Address::default()),
			value: 0.into(),
			data: Vec::new(),
		}.sign(&"1".sha3(), None);
		client.miner().import_own_transaction(client.as_ref(), tx.into()).unwrap();
		client.update_sealing();
		// Able to seal again.
		assert_eq!(client.chain_info().best_block_number, 3);
	}
}
