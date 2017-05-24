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

/// Validator lists.

#[cfg(test)]
mod test;
mod simple_list;
mod safe_contract;
mod contract;
mod multi;

use std::sync::Weak;
use ids::BlockId;
use util::{Bytes, Address, H256};
use ethjson::spec::ValidatorSet as ValidatorSpec;
use client::Client;
use header::{Header, BlockNumber};

#[cfg(test)]
pub use self::test::TestSet;
pub use self::simple_list::SimpleList;
use self::contract::ValidatorContract;
use self::safe_contract::ValidatorSafeContract;
use self::multi::Multi;

use super::Call;

/// Creates a validator set from spec.
pub fn new_validator_set(spec: ValidatorSpec) -> Box<ValidatorSet> {
	match spec {
		ValidatorSpec::List(list) => Box::new(SimpleList::new(list.into_iter().map(Into::into).collect())),
		ValidatorSpec::SafeContract(address) => Box::new(ValidatorSafeContract::new(address.into())),
		ValidatorSpec::Contract(address) => Box::new(ValidatorContract::new(address.into())),
		ValidatorSpec::Multi(sequence) => Box::new(
			Multi::new(sequence.into_iter().map(|(block, set)| (block.into(), new_validator_set(set))).collect())
		),
	}
}

/// A validator set.
pub trait ValidatorSet: Send + Sync {
	/// Get the default "Call" helper, for use in general operation.
	// TODO [keorn]: this is a hack intended to migrate off of
	// a strict dependency on state always being available.
	fn default_caller(&self, block_id: BlockId) -> Box<Call>;

	/// Checks if a given address is a validator,
	/// using underlying, default call mechanism.
	fn contains(&self, parent: &H256, address: &Address) -> bool {
		let default = self.default_caller(BlockId::Hash(*parent));
		self.contains_with_caller(parent, address, &*default)
	}
	/// Draws an validator nonce modulo number of validators.
	fn get(&self, parent: &H256, nonce: usize) -> Address {
		let default = self.default_caller(BlockId::Hash(*parent));
		self.get_with_caller(parent, nonce, &*default)
	}

	/// Returns the current number of validators.
	fn count(&self, parent: &H256) -> usize {
		let default = self.default_caller(BlockId::Hash(*parent));
		self.count_with_caller(parent, &*default)
	}

	/// Whether this block is the last one in its epoch.
	/// Usually indicates that the validator set changed at the given block.
	///
	/// Should not inspect state! This is used in situations where
	/// state is not generally available.
	///
	/// Return `Yes` or `No` indicating whether it changed at the given header,
	/// or `Unsure` indicating a need for more information.
	///
	/// If block or receipts are provided, do not return `Unsure` indicating
	/// need for them.
	fn is_epoch_end(&self, header: &Header, block: Option<&[u8]>, receipts: Option<&[::receipt::Receipt]>)
		-> super::EpochChange;

	/// Generate epoch proof.
	/// Must interact with state only through the given caller!
	/// Otherwise, generated proofs may be wrong.
	fn epoch_proof(&self, header: &Header, caller: &Call) -> Result<Vec<u8>, String>;

	/// Recover the validator set for all
	///
	/// May fail if the given header doesn't kick off an epoch or
	/// the proof is invalid.
	///
	/// Returns the epoch number and proof.
	fn epoch_set(&self, header: &Header, proof: &[u8]) -> Result<(u64, SimpleList), ::error::Error>;

	/// Checks if a given address is a validator, with the given function
	/// for executing synchronous calls to contracts.
	fn contains_with_caller(&self, parent_block_hash: &H256, address: &Address, caller: &Call) -> bool;

	/// Draws an validator nonce modulo number of validators.
	///
	fn get_with_caller(&self, parent_block_hash: &H256, nonce: usize, caller: &Call) -> Address;

	/// Returns the current number of validators.
	fn count_with_caller(&self, parent_block_hash: &H256, caller: &Call) -> usize;

	/// Notifies about malicious behaviour.
	fn report_malicious(&self, _validator: &Address, _block: BlockNumber, _proof: Bytes) {}
	/// Notifies about benign misbehaviour.
	fn report_benign(&self, _validator: &Address, _block: BlockNumber) {}
	/// Allows blockchain state access.
	fn register_contract(&self, _client: Weak<Client>) {}
}
