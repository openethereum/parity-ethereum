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

/// Validator set maintained in a contract, updated using `getValidators` method.
/// It can also report validators for misbehaviour with two levels: `reportMalicious` and `reportBenign`.

use std::sync::Weak;

use bytes::Bytes;
use ethereum_types::{H256, Address};
use parking_lot::RwLock;

use client::EngineClient;
use header::{Header, BlockNumber};
use machine::{AuxiliaryData, Call, EthereumMachine};

use super::{ValidatorSet, SimpleList, SystemCall};
use super::safe_contract::ValidatorSafeContract;

use_contract!(validator_report, "ValidatorReport", "res/contracts/validator_report.json");

/// A validator contract with reporting.
pub struct ValidatorContract {
	contract_address: Address,
	validators: ValidatorSafeContract,
	provider: validator_report::ValidatorReport,
	client: RwLock<Option<Weak<EngineClient>>>, // TODO [keorn]: remove
}

impl ValidatorContract {
	pub fn new(contract_address: Address) -> Self {
		ValidatorContract {
			contract_address,
			validators: ValidatorSafeContract::new(contract_address),
			provider: validator_report::ValidatorReport::default(),
			client: RwLock::new(None),
		}
	}
}

impl ValidatorContract {
	fn transact(&self, data: Bytes) -> Result<(), String> {
		let client = self.client.read().as_ref()
			.and_then(Weak::upgrade)
			.ok_or_else(|| "No client!")?;

		match client.as_full_client() {
			Some(c) => {
				c.transact_contract(self.contract_address, data)
					.map_err(|e| format!("Transaction import error: {}", e))?;
				Ok(())
			},
			None => Err("No full client!".into()),
		}
	}
}

impl ValidatorSet for ValidatorContract {
	fn default_caller(&self, id: ::ids::BlockId) -> Box<Call> {
		self.validators.default_caller(id)
	}

	fn on_epoch_begin(&self, first: bool, header: &Header, call: &mut SystemCall) -> Result<(), ::error::Error> {
		self.validators.on_epoch_begin(first, header, call)
	}

	fn genesis_epoch_data(&self, header: &Header, call: &Call) -> Result<Vec<u8>, String> {
		self.validators.genesis_epoch_data(header, call)
	}

	fn is_epoch_end(&self, first: bool, chain_head: &Header) -> Option<Vec<u8>> {
		self.validators.is_epoch_end(first, chain_head)
	}

	fn signals_epoch_end(
		&self,
		first: bool,
		header: &Header,
		aux: AuxiliaryData,
	) -> ::engines::EpochChange<EthereumMachine> {
		self.validators.signals_epoch_end(first, header, aux)
	}

	fn epoch_set(&self, first: bool, machine: &EthereumMachine, number: BlockNumber, proof: &[u8]) -> Result<(SimpleList, Option<H256>), ::error::Error> {
		self.validators.epoch_set(first, machine, number, proof)
	}

	fn contains_with_caller(&self, bh: &H256, address: &Address, caller: &Call) -> bool {
		self.validators.contains_with_caller(bh, address, caller)
	}

	fn get_with_caller(&self, bh: &H256, nonce: usize, caller: &Call) -> Address {
		self.validators.get_with_caller(bh, nonce, caller)
	}

	fn count_with_caller(&self, bh: &H256, caller: &Call) -> usize {
		self.validators.count_with_caller(bh, caller)
	}

	fn report_malicious(&self, address: &Address, _set_block: BlockNumber, block: BlockNumber, proof: Bytes) {
		let data = self.provider.functions().report_malicious().input(*address, block, proof);
		match self.transact(data) {
			Ok(_) => warn!(target: "engine", "Reported malicious validator {}", address),
			Err(s) => warn!(target: "engine", "Validator {} could not be reported {}", address, s),
		}
	}

	fn report_benign(&self, address: &Address, _set_block: BlockNumber, block: BlockNumber) {
		let data = self.provider.functions().report_benign().input(*address, block);
		match self.transact(data) {
			Ok(_) => warn!(target: "engine", "Reported benign validator misbehaviour {}", address),
			Err(s) => warn!(target: "engine", "Validator {} could not be reported {}", address, s),
		}
	}

	fn register_client(&self, client: Weak<EngineClient>) {
		self.validators.register_client(client.clone());
		*self.client.write() = Some(client);
	}
}

#[cfg(test)]
mod tests {
	use std::sync::Arc;
	use rustc_hex::FromHex;
	use hash::keccak;
	use ethereum_types::{H520, Address};
	use bytes::ToPretty;
	use rlp::encode;
	use spec::Spec;
	use header::Header;
	use account_provider::AccountProvider;
	use miner::MinerService;
	use types::ids::BlockId;
	use client::{BlockChainClient, ChainInfo, BlockInfo, CallContract};
	use tests::helpers::generate_dummy_client_with_spec_and_accounts;
	use super::super::ValidatorSet;
	use super::ValidatorContract;

	#[test]
	fn fetches_validators() {
		let client = generate_dummy_client_with_spec_and_accounts(Spec::new_validator_contract, None);
		let vc = Arc::new(ValidatorContract::new("0000000000000000000000000000000000000005".parse::<Address>().unwrap()));
		vc.register_client(Arc::downgrade(&client) as _);
		let last_hash = client.best_block_header().hash();
		assert!(vc.contains(&last_hash, &"7d577a597b2742b498cb5cf0c26cdcd726d39e6e".parse::<Address>().unwrap()));
		assert!(vc.contains(&last_hash, &"82a978b3f5962a5b0957d9ee9eef472ee55b42f1".parse::<Address>().unwrap()));
	}

	#[test]
	fn reports_validators() {
		let tap = Arc::new(AccountProvider::transient_provider());
		let v1 = tap.insert_account(keccak("1").into(), "").unwrap();
		let client = generate_dummy_client_with_spec_and_accounts(Spec::new_validator_contract, Some(tap.clone()));
		client.engine().register_client(Arc::downgrade(&client) as _);
		let validator_contract = "0000000000000000000000000000000000000005".parse::<Address>().unwrap();

		// Make sure reporting can be done.
		client.miner().set_gas_floor_target(1_000_000.into());
		client.miner().set_engine_signer(v1, "".into()).unwrap();

		// Check a block that is a bit in future, reject it but don't report the validator.
		let mut header = Header::default();
		let seal = vec![encode(&4u8).into_vec(), encode(&(&H520::default() as &[u8])).into_vec()];
		header.set_seal(seal);
		header.set_author(v1);
		header.set_number(2);
		header.set_parent_hash(client.chain_info().best_block_hash);
		assert!(client.engine().verify_block_external(&header).is_err());
		client.engine().step();
		assert_eq!(client.chain_info().best_block_number, 0);

		// Now create one that is more in future. That one should be rejected and validator should be reported.
		let mut header = Header::default();
		let seal = vec![encode(&8u8).into_vec(), encode(&(&H520::default() as &[u8])).into_vec()];
		header.set_seal(seal);
		header.set_author(v1);
		header.set_number(2);
		header.set_parent_hash(client.chain_info().best_block_hash);
		// `reportBenign` when the designated proposer releases block from the future (bad clock).
		assert!(client.engine().verify_block_basic(&header).is_err());
		// Seal a block.
		client.engine().step();
		assert_eq!(client.chain_info().best_block_number, 1);
		// Check if the unresponsive validator is `disliked`.
		assert_eq!(
			client.call_contract(BlockId::Latest, validator_contract, "d8f2e0bf".from_hex().unwrap()).unwrap().to_hex(),
			"0000000000000000000000007d577a597b2742b498cb5cf0c26cdcd726d39e6e"
		);
		// Simulate a misbehaving validator by handling a double proposal.
		let header = client.best_block_header();
		assert!(client.engine().verify_block_family(&header, &header).is_err());
		// Seal a block.
		client.engine().step();
		client.engine().step();
		assert_eq!(client.chain_info().best_block_number, 2);

		// Check if misbehaving validator was removed.
		client.transact_contract(Default::default(), Default::default()).unwrap();
		client.engine().step();
		client.engine().step();
		assert_eq!(client.chain_info().best_block_number, 2);
	}
}
