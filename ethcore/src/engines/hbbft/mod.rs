// Copyright 2015-2018 Parity Technologies (UK) Ltd.
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

use std::sync::{Arc, Weak};

use block::ExecutedBlock;
use client::EngineClient;
use engines::{Engine, Seal, signer::EngineSigner, ForkChoice};
use ethjson;
use ethkey::Password;
use account_provider::AccountProvider;
use ethereum_types::Address;
use error::{BlockError, Error};
use header::{Header, ExtendedHeader};
use machine::EthereumMachine;
use parking_lot::RwLock;

/// `Hbbft` params.
#[derive(Debug, PartialEq)]
pub struct HbbftParams {
	/// Whether to use millisecond timestamp
	pub millisecond_timestamp: bool,
}

impl From<ethjson::spec::HbbftParams> for HbbftParams {
	fn from(p: ethjson::spec::HbbftParams) -> Self {
		HbbftParams {
			millisecond_timestamp: p.millisecond_timestamp,
		}
	}
}

/// An engine which does not provide any consensus mechanism, just seals blocks internally.
/// Only seals blocks which have transactions.
pub struct Hbbft {
	params: HbbftParams,
	machine: EthereumMachine,
	client: RwLock<Option<Weak<EngineClient>>>,
	signer: RwLock<EngineSigner>,
}

impl Hbbft {
	/// Returns new instance of Hbbft over the given state machine.
	pub fn new(params: HbbftParams, machine: EthereumMachine) -> Self {
		Hbbft {
			params,
			machine,
			client: RwLock::new(None),
			signer: Default::default(),
		}
	}
}
/// A temporary fixed seal code. The seal has only a single field, containing this string.
// TODO: Use a threshold signature of the block.
const SEAL: &[u8] = b"Don't care.";

impl Engine<EthereumMachine> for Hbbft {
	fn name(&self) -> &str {
		"Hbbft"
	}

	fn machine(&self) -> &EthereumMachine { &self.machine }

	fn seals_internally(&self) -> Option<bool> { Some(true) }

	fn seal_fields(&self, _header: &Header) -> usize { 1 }

	fn generate_seal(&self, _block: &ExecutedBlock, _parent: &Header) -> Seal {
		Seal::Regular(vec!(SEAL.to_vec()))
	}

	fn verify_local_seal(&self, header: &Header) -> Result<(), Error> {
		if header.seal() == &[SEAL] {
			Ok(())
		} else {
			Err(BlockError::InvalidSeal.into())
		}
	}

	fn open_block_header_timestamp(&self, parent_timestamp: u64) -> u64 {
		use std::{time, cmp};

		let dur = time::SystemTime::now().duration_since(time::UNIX_EPOCH).unwrap_or_default();
		let mut now = dur.as_secs();
		if self.params.millisecond_timestamp {
			now = now * 1000 + dur.subsec_millis() as u64;
		}
		cmp::max(now, parent_timestamp)
	}

	fn is_timestamp_valid(&self, header_timestamp: u64, parent_timestamp: u64) -> bool {
		header_timestamp >= parent_timestamp
	}

	fn fork_choice(&self, new: &ExtendedHeader, current: &ExtendedHeader) -> ForkChoice {
		if new.header.number() > current.header.number() {
			ForkChoice::New
		} else if new.header.number() < current.header.number() {
			ForkChoice::Old
		} else {
			panic!("Consensus failure: blocks {:?} and {:?} have equal number {}.",
				   new.header.hash(), current.header.hash(), new.header.number());
		}
	}

	fn register_client(&self, client: Weak<EngineClient>) {
		*self.client.write() = Some(client.clone());
	}

	fn set_signer(&self, ap: Arc<AccountProvider>, address: Address, password: Password) {
		self.signer.write().set(ap, address, password);
	}
}

#[cfg(test)]
mod tests {
	use std::sync::Arc;
	use ethereum_types::{H520, Address};
	use test_helpers::get_temp_state_db;
	use spec::Spec;
	use header::Header;
	use block::*;
	use engines::Seal;

	#[test]
	fn instant_can_seal() {
		let spec = Spec::new_instant();
		let engine = &*spec.engine;
		let db = spec.ensure_db_good(get_temp_state_db(), &Default::default()).unwrap();
		let genesis_header = spec.genesis_header();
		let last_hashes = Arc::new(vec![genesis_header.hash()]);
		let b = OpenBlock::new(engine, Default::default(), false, db, &genesis_header, last_hashes, Address::default(), (3141562.into(), 31415620.into()), vec![], false, &mut Vec::new().into_iter()).unwrap();
		let b = b.close_and_lock().unwrap();
		if let Seal::Regular(seal) = engine.generate_seal(b.block(), &genesis_header) {
			assert!(b.try_seal(engine, seal).is_ok());
		}
	}

	#[test]
	fn instant_cant_verify() {
		let engine = Spec::new_instant().engine;
		let mut header: Header = Header::default();

		assert!(engine.verify_block_basic(&header).is_ok());

		header.set_seal(vec![::rlp::encode(&H520::default()).into_vec()]);

		assert!(engine.verify_block_unordered(&header).is_ok());
	}
}
