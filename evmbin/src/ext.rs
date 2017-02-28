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

//! Externalities implementation.

use std::sync::Arc;
use std::collections::HashMap;
use util::{U256, H256, Address, Bytes, FixedHash, trie};
use ethcore::client::EnvInfo;
use ethcore::evm::{self, Ext, ContractCreateResult, MessageCallResult, Schedule, CallType};

pub struct FakeExt {
	schedule: Schedule,
	store: HashMap<H256, H256>,
	depth: usize,
}

impl Default for FakeExt {
	fn default() -> Self {
		FakeExt {
			schedule: Schedule::new_post_eip150(usize::max_value(), true, true, true),
			store: HashMap::new(),
			depth: 1,
		}
	}
}

impl Ext for FakeExt {
	fn storage_at(&self, key: &H256) -> trie::Result<H256> {
		Ok(self.store.get(key).unwrap_or(&H256::new()).clone())
	}

	fn set_storage(&mut self, key: H256, value: H256) -> trie::Result<()> {
		self.store.insert(key, value);
		Ok(())
	}

	fn exists(&self, _address: &Address) -> trie::Result<bool> {
		unimplemented!();
	}

	fn exists_and_not_null(&self, _address: &Address) -> trie::Result<bool> {
		unimplemented!();
	}

	fn origin_balance(&self) -> trie::Result<U256> {
		unimplemented!();
	}

	fn balance(&self, _address: &Address) -> trie::Result<U256> {
		unimplemented!();
	}

	fn blockhash(&self, _number: &U256) -> H256 {
		unimplemented!();
	}

	fn create(&mut self, _gas: &U256, _value: &U256, _code: &[u8]) -> ContractCreateResult {
		unimplemented!();
	}

	fn call(&mut self,
			_gas: &U256,
			_sender_address: &Address,
			_receive_address: &Address,
			_value: Option<U256>,
			_data: &[u8],
			_code_address: &Address,
			_output: &mut [u8],
			_call_type: CallType) -> MessageCallResult {
		unimplemented!();
	}

	fn extcode(&self, _address: &Address) -> trie::Result<Arc<Bytes>> {
		unimplemented!();
	}

	fn extcodesize(&self, _address: &Address) -> trie::Result<usize> {
		unimplemented!();
	}

	fn log(&mut self, _topics: Vec<H256>, _data: &[u8]) {
		unimplemented!();
	}

	fn ret(self, gas: &U256, _data: &[u8]) -> evm::Result<U256> {
		Ok(*gas)
	}

	fn suicide(&mut self, _refund_address: &Address) -> trie::Result<()> {
		unimplemented!();
	}

	fn schedule(&self) -> &Schedule {
		&self.schedule
	}

	fn env_info(&self) -> &EnvInfo {
		unimplemented!()
	}

	fn depth(&self) -> usize {
		self.depth
	}

	fn inc_sstore_clears(&mut self) {
		unimplemented!();
		// self.sstore_clears += 1;
	}
}
