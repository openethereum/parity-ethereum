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
use evm;
use evm::{Ext, Schedule, Factory, VMType, ContractCreateResult, MessageCallResult};
use std::fmt::Debug;

struct FakeLogEntry {
	topics: Vec<H256>,
	data: Bytes
}

#[derive(PartialEq, Eq, Hash, Debug)]
enum FakeCallType {
	Call, Create
}

#[derive(PartialEq, Eq, Hash, Debug)]
struct FakeCall {
	call_type: FakeCallType,
	gas: U256,
	sender_address: Option<Address>,
	receive_address: Option<Address>,
	value: Option<U256>,
	data: Bytes,
	code_address: Option<Address>
}

/// Fake externalities test structure.
///
/// Can't do recursive calls.
#[derive(Default)]
struct FakeExt {
	sstore_clears: usize,
	depth: usize,
	store: HashMap<H256, H256>,
	blockhashes: HashMap<U256, H256>,
	codes: HashMap<Address, Bytes>,
	logs: Vec<FakeLogEntry>,
	_suicides: HashSet<Address>,
	info: EnvInfo,
	schedule: Schedule,
	balances: HashMap<Address, U256>,
	calls: HashSet<FakeCall>,
}

impl FakeExt {
	fn new() -> Self {
		FakeExt::default()
	}
}

impl Default for Schedule {
	fn default() -> Self {
		Schedule::new_frontier()
	}
}

impl Ext for FakeExt {
	fn storage_at(&self, key: &H256) -> H256 {
		self.store.get(key).unwrap_or(&H256::new()).clone()
	}

	fn set_storage(&mut self, key: H256, value: H256) {
		self.store.insert(key, value);
	}

	fn exists(&self, address: &Address) -> bool {
		self.balances.contains_key(address)
	}

	fn balance(&self, address: &Address) -> U256 {
		*self.balances.get(address).unwrap()
	}

	fn blockhash(&self, number: &U256) -> H256 {
		self.blockhashes.get(number).unwrap_or(&H256::new()).clone()
	}

	fn create(&mut self, gas: &U256, value: &U256, code: &[u8]) -> ContractCreateResult {
		self.calls.insert(FakeCall {
			call_type: FakeCallType::Create,
			gas: *gas,
			sender_address: None,
			receive_address: None,
			value: Some(*value),
			data: code.to_vec(),
			code_address: None
		});
		ContractCreateResult::Failed
	}

	fn call(&mut self,
			gas: &U256,
			sender_address: &Address,
			receive_address: &Address,
			value: Option<U256>,
			data: &[u8],
			code_address: &Address,
			_output: &mut [u8]) -> MessageCallResult {

		self.calls.insert(FakeCall {
			call_type: FakeCallType::Call,
			gas: *gas,
			sender_address: Some(sender_address.clone()),
			receive_address: Some(receive_address.clone()),
			value: value,
			data: data.to_vec(),
			code_address: Some(code_address.clone())
		});
		MessageCallResult::Success(*gas)
	}

	fn extcode(&self, address: &Address) -> Bytes {
		self.codes.get(address).unwrap_or(&Bytes::new()).clone()
	}

	fn log(&mut self, topics: Vec<H256>, data: &[u8]) {
		self.logs.push(FakeLogEntry {
			topics: topics,
			data: data.to_vec()
		});
	}

	fn ret(&mut self, _gas: &U256, _data: &[u8]) -> result::Result<U256, evm::Error> {
		unimplemented!();
	}

	fn suicide(&mut self, _refund_address: &Address) {
		unimplemented!();
	}

	fn schedule(&self) -> &Schedule {
		&self.schedule
	}

	fn env_info(&self) -> &EnvInfo {
		&self.info
	}

	fn depth(&self) -> usize {
		self.depth
	}

	fn inc_sstore_clears(&mut self) {
		self.sstore_clears += 1;
	}
}

#[test]
fn test_stack_underflow() {
	let address = Address::from_str("0f572e5295c57f15886f9b263e2f6d2d6c7b5ec6").unwrap();
	let code = "01600055".from_hex().unwrap();

	let mut params = ActionParams::default();
	params.address = address.clone();
	params.gas = U256::from(100_000);
	params.code = Some(code);
	let mut ext = FakeExt::new();

	let err = {
		let vm : Box<evm::Evm> = Box::new(super::interpreter::Interpreter);
		vm.exec(params, &mut ext).unwrap_err()
	};

	match err {
		evm::Error::StackUnderflow {wanted, on_stack, ..} => {
			assert_eq!(wanted, 2);
			assert_eq!(on_stack, 0);
		}
		_ => {
			assert!(false, "Expected StackUndeflow")
		}
	};
}

evm_test!{test_add: test_add_jit, test_add_int}
fn test_add(factory: super::Factory) {
  let address = Address::from_str("0f572e5295c57f15886f9b263e2f6d2d6c7b5ec6").unwrap();
	let code = "7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff01600055".from_hex().unwrap();

	let mut params = ActionParams::default();
	params.address = address.clone();
	params.gas = U256::from(100_000);
	params.code = Some(code);
	let mut ext = FakeExt::new();

	let gas_left = {
		let vm = factory.create();
		vm.exec(params, &mut ext).unwrap()
	};

	assert_eq!(gas_left, U256::from(79_988));
	assert_store(&ext, 0, "fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe");
}

evm_test!{test_sha3: test_sha3_jit, test_sha3_int}
fn test_sha3(factory: super::Factory) {
	let address = Address::from_str("0f572e5295c57f15886f9b263e2f6d2d6c7b5ec6").unwrap();
	let code = "6000600020600055".from_hex().unwrap();

	let mut params = ActionParams::default();
	params.address = address.clone();
	params.gas = U256::from(100_000);
	params.code = Some(code);
	let mut ext = FakeExt::new();

	let gas_left = {
		let vm = factory.create();
		vm.exec(params, &mut ext).unwrap()
	};

	assert_eq!(gas_left, U256::from(79_961));
	assert_store(&ext, 0, "c5d2460186f7233c927e7db2dcc703c0e500b653ca82273b7bfad8045d85a470");
}

evm_test!{test_address: test_address_jit, test_address_int}
fn test_address(factory: super::Factory) {
	let address = Address::from_str("0f572e5295c57f15886f9b263e2f6d2d6c7b5ec6").unwrap();
	let code = "30600055".from_hex().unwrap();

	let mut params = ActionParams::default();
	params.address = address.clone();
	params.gas = U256::from(100_000);
	params.code = Some(code);
	let mut ext = FakeExt::new();

	let gas_left = {
		let vm = factory.create();
		vm.exec(params, &mut ext).unwrap()
	};

	assert_eq!(gas_left, U256::from(79_995));
	assert_store(&ext, 0, "0000000000000000000000000f572e5295c57f15886f9b263e2f6d2d6c7b5ec6");
}

evm_test!{test_origin: test_origin_jit, test_origin_int}
fn test_origin(factory: super::Factory) {
	let address = Address::from_str("0f572e5295c57f15886f9b263e2f6d2d6c7b5ec6").unwrap();
	let origin = Address::from_str("cd1722f2947def4cf144679da39c4c32bdc35681").unwrap();
	let code = "32600055".from_hex().unwrap();

	let mut params = ActionParams::default();
	params.address = address.clone();
	params.origin = origin.clone();
	params.gas = U256::from(100_000);
	params.code = Some(code);
	let mut ext = FakeExt::new();

	let gas_left = {
		let vm = factory.create();
		vm.exec(params, &mut ext).unwrap()
	};

	assert_eq!(gas_left, U256::from(79_995));
	assert_store(&ext, 0, "000000000000000000000000cd1722f2947def4cf144679da39c4c32bdc35681");
}

evm_test!{test_sender: test_sender_jit, test_sender_int}
fn test_sender(factory: super::Factory) {
	let address = Address::from_str("0f572e5295c57f15886f9b263e2f6d2d6c7b5ec6").unwrap();
	let sender = Address::from_str("cd1722f2947def4cf144679da39c4c32bdc35681").unwrap();
	let code = "33600055".from_hex().unwrap();

	let mut params = ActionParams::default();
	params.address = address.clone();
	params.sender = sender.clone();
	params.gas = U256::from(100_000);
	params.code = Some(code);
	let mut ext = FakeExt::new();

	let gas_left = {
		let vm = factory.create();
		vm.exec(params, &mut ext).unwrap()
	};

	assert_eq!(gas_left, U256::from(79_995));
	assert_store(&ext, 0, "000000000000000000000000cd1722f2947def4cf144679da39c4c32bdc35681");
}

evm_test!{test_extcodecopy: test_extcodecopy_jit, test_extcodecopy_int}
fn test_extcodecopy(factory: super::Factory) {
		// 33 - sender
		// 3b - extcodesize
		// 60 00 - push 0
		// 60 00 - push 0
		// 33 - sender
		// 3c - extcodecopy
		// 60 00 - push 0
		// 51 - load word from memory
		// 60 00 - push 0
		// 55 - sstore

	let address = Address::from_str("0f572e5295c57f15886f9b263e2f6d2d6c7b5ec6").unwrap();
	let sender = Address::from_str("cd1722f2947def4cf144679da39c4c32bdc35681").unwrap();
	let code = "333b60006000333c600051600055".from_hex().unwrap();
	let sender_code = "6005600055".from_hex().unwrap();

	let mut params = ActionParams::default();
	params.address = address.clone();
	params.sender = sender.clone();
	params.gas = U256::from(100_000);
	params.code = Some(code);
	let mut ext = FakeExt::new();
	ext.codes.insert(sender, sender_code);

	let gas_left = {
		let vm = factory.create();
		vm.exec(params, &mut ext).unwrap()
	};

	assert_eq!(gas_left, U256::from(79_935));
	assert_store(&ext, 0, "6005600055000000000000000000000000000000000000000000000000000000");
}

evm_test!{test_log_empty: test_log_empty_jit, test_log_empty_int}
fn test_log_empty(factory: super::Factory) {
	let address = Address::from_str("0f572e5295c57f15886f9b263e2f6d2d6c7b5ec6").unwrap();
	let code = "60006000a0".from_hex().unwrap();

	let mut params = ActionParams::default();
	params.address = address.clone();
	params.gas = U256::from(100_000);
	params.code = Some(code);
	let mut ext = FakeExt::new();

	let gas_left = {
		let vm = factory.create();
		vm.exec(params, &mut ext).unwrap()
	};

	assert_eq!(gas_left, U256::from(99_619));
	assert_eq!(ext.logs.len(), 1);
	assert_eq!(ext.logs[0].topics.len(), 0);
	assert!(ext.logs[0].data.is_empty());
}

evm_test!{test_log_sender: test_log_sender_jit, test_log_sender_int}
fn test_log_sender(factory: super::Factory) {
	// 60 ff - push ff
	// 60 00 - push 00
	// 53 - mstore
	// 33 - sender
	// 60 20 - push 20
	// 60 00 - push 0
	// a1 - log with 1 topic

	let address = Address::from_str("0f572e5295c57f15886f9b263e2f6d2d6c7b5ec6").unwrap();
	let sender = Address::from_str("cd1722f3947def4cf144679da39c4c32bdc35681").unwrap();
	let code = "60ff6000533360206000a1".from_hex().unwrap();

	let mut params = ActionParams::default();
	params.address = address.clone();
	params.sender = sender.clone();
	params.gas = U256::from(100_000);
	params.code = Some(code);
	let mut ext = FakeExt::new();

	let gas_left = {
		let vm = factory.create();
		vm.exec(params, &mut ext).unwrap()
	};

	assert_eq!(gas_left, U256::from(98_974));
	assert_eq!(ext.logs.len(), 1);
	assert_eq!(ext.logs[0].topics.len(), 1);
	assert_eq!(ext.logs[0].topics[0], H256::from_str("000000000000000000000000cd1722f3947def4cf144679da39c4c32bdc35681").unwrap());
	assert_eq!(ext.logs[0].data, "ff00000000000000000000000000000000000000000000000000000000000000".from_hex().unwrap());
}

evm_test!{test_blockhash: test_blockhash_jit, test_blockhash_int}
fn test_blockhash(factory: super::Factory) {
	let address = Address::from_str("0f572e5295c57f15886f9b263e2f6d2d6c7b5ec6").unwrap();
	let code = "600040600055".from_hex().unwrap();
	let blockhash = H256::from_str("123400000000000000000000cd1722f2947def4cf144679da39c4c32bdc35681").unwrap();

	let mut params = ActionParams::default();
	params.address = address.clone();
	params.gas = U256::from(100_000);
	params.code = Some(code);
	let mut ext = FakeExt::new();
	ext.blockhashes.insert(U256::zero(), blockhash.clone());

	let gas_left = {
		let vm = factory.create();
		vm.exec(params, &mut ext).unwrap()
	};

	assert_eq!(gas_left, U256::from(79_974));
	assert_eq!(ext.store.get(&H256::new()).unwrap(), &blockhash);
}

evm_test!{test_calldataload: test_calldataload_jit, test_calldataload_int}
fn test_calldataload(factory: super::Factory) {
	let address = Address::from_str("0f572e5295c57f15886f9b263e2f6d2d6c7b5ec6").unwrap();
	let code = "600135600055".from_hex().unwrap();
	let data = "0123ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff23".from_hex().unwrap();

	let mut params = ActionParams::default();
	params.address = address.clone();
	params.gas = U256::from(100_000);
	params.code = Some(code);
	params.data = Some(data);
	let mut ext = FakeExt::new();

	let gas_left = {
		let vm = factory.create();
		vm.exec(params, &mut ext).unwrap()
	};

	assert_eq!(gas_left, U256::from(79_991));
	assert_store(&ext, 0, "23ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff23");

}

evm_test!{test_author: test_author_jit, test_author_int}
fn test_author(factory: super::Factory) {
	let author = Address::from_str("0f572e5295c57f15886f9b263e2f6d2d6c7b5ec6").unwrap();
	let code = "41600055".from_hex().unwrap();

	let mut params = ActionParams::default();
	params.gas = U256::from(100_000);
	params.code = Some(code);
	let mut ext = FakeExt::new();
	ext.info.author = author;

	let gas_left = {
		let vm = factory.create();
		vm.exec(params, &mut ext).unwrap()
	};

	assert_eq!(gas_left, U256::from(79_995));
	assert_store(&ext, 0, "0000000000000000000000000f572e5295c57f15886f9b263e2f6d2d6c7b5ec6");
}

evm_test!{test_timestamp: test_timestamp_jit, test_timestamp_int}
fn test_timestamp(factory: super::Factory) {
	let timestamp = 0x1234;
	let code = "42600055".from_hex().unwrap();

	let mut params = ActionParams::default();
	params.gas = U256::from(100_000);
	params.code = Some(code);
	let mut ext = FakeExt::new();
	ext.info.timestamp = timestamp;

	let gas_left = {
		let vm = factory.create();
		vm.exec(params, &mut ext).unwrap()
	};

	assert_eq!(gas_left, U256::from(79_995));
	assert_store(&ext, 0, "0000000000000000000000000000000000000000000000000000000000001234");
}

evm_test!{test_number: test_number_jit, test_number_int}
fn test_number(factory: super::Factory) {
	let number = 0x1234;
	let code = "43600055".from_hex().unwrap();

	let mut params = ActionParams::default();
	params.gas = U256::from(100_000);
	params.code = Some(code);
	let mut ext = FakeExt::new();
	ext.info.number = number;

	let gas_left = {
		let vm = factory.create();
		vm.exec(params, &mut ext).unwrap()
	};

	assert_eq!(gas_left, U256::from(79_995));
	assert_store(&ext, 0, "0000000000000000000000000000000000000000000000000000000000001234");
}

evm_test!{test_difficulty: test_difficulty_jit, test_difficulty_int}
fn test_difficulty(factory: super::Factory) {
	let difficulty = U256::from(0x1234);
	let code = "44600055".from_hex().unwrap();

	let mut params = ActionParams::default();
	params.gas = U256::from(100_000);
	params.code = Some(code);
	let mut ext = FakeExt::new();
	ext.info.difficulty = difficulty;

	let gas_left = {
		let vm = factory.create();
		vm.exec(params, &mut ext).unwrap()
	};

	assert_eq!(gas_left, U256::from(79_995));
	assert_store(&ext, 0, "0000000000000000000000000000000000000000000000000000000000001234");
}

evm_test!{test_gas_limit: test_gas_limit_jit, test_gas_limit_int}
fn test_gas_limit(factory: super::Factory) {
	let gas_limit = U256::from(0x1234);
	let code = "45600055".from_hex().unwrap();

	let mut params = ActionParams::default();
	params.gas = U256::from(100_000);
	params.code = Some(code);
	let mut ext = FakeExt::new();
	ext.info.gas_limit = gas_limit;

	let gas_left = {
		let vm = factory.create();
		vm.exec(params, &mut ext).unwrap()
	};

	assert_eq!(gas_left, U256::from(79_995));
	assert_store(&ext, 0, "0000000000000000000000000000000000000000000000000000000000001234");
}

evm_test!{test_mul: test_mul_jit, test_mul_int}
fn test_mul(factory: super::Factory) {
	let code = "65012365124623626543219002600055".from_hex().unwrap();

	let mut params = ActionParams::default();
	params.gas = U256::from(100_000);
	params.code = Some(code);
	let mut ext = FakeExt::new();

	let gas_left = {
		let vm = factory.create();
		vm.exec(params, &mut ext).unwrap()
	};

	assert_store(&ext, 0, "000000000000000000000000000000000000000000000000734349397b853383");
	assert_eq!(gas_left, U256::from(79_983));
}

evm_test!{test_sub: test_sub_jit, test_sub_int}
fn test_sub(factory: super::Factory) {
	let code = "65012365124623626543219003600055".from_hex().unwrap();

	let mut params = ActionParams::default();
	params.gas = U256::from(100_000);
	params.code = Some(code);
	let mut ext = FakeExt::new();

	let gas_left = {
		let vm = factory.create();
		vm.exec(params, &mut ext).unwrap()
	};

	assert_store(&ext, 0, "0000000000000000000000000000000000000000000000000000012364ad0302");
	assert_eq!(gas_left, U256::from(79_985));
}

evm_test!{test_div: test_div_jit, test_div_int}
fn test_div(factory: super::Factory) {
	let code = "65012365124623626543219004600055".from_hex().unwrap();

	let mut params = ActionParams::default();
	params.gas = U256::from(100_000);
	params.code = Some(code);
	let mut ext = FakeExt::new();

	let gas_left = {
		let vm = factory.create();
		vm.exec(params, &mut ext).unwrap()
	};

	assert_store(&ext, 0, "000000000000000000000000000000000000000000000000000000000002e0ac");
	assert_eq!(gas_left, U256::from(79_983));
}

evm_test!{test_div_zero: test_div_zero_jit, test_div_zero_int}
fn test_div_zero(factory: super::Factory) {
	let code = "6501236512462360009004600055".from_hex().unwrap();

	let mut params = ActionParams::default();
	params.gas = U256::from(100_000);
	params.code = Some(code);
	let mut ext = FakeExt::new();

	let gas_left = {
		let vm = factory.create();
		vm.exec(params, &mut ext).unwrap()
	};

	assert_store(&ext, 0, "0000000000000000000000000000000000000000000000000000000000000000");
	assert_eq!(gas_left, U256::from(94_983));
}

evm_test!{test_mod: test_mod_jit, test_mod_int}
fn test_mod(factory: super::Factory) {
	let code = "650123651246236265432290066000556501236512462360009006600155".from_hex().unwrap();

	let mut params = ActionParams::default();
	params.gas = U256::from(100_000);
	params.code = Some(code);
	let mut ext = FakeExt::new();

	let gas_left = {
		let vm = factory.create();
		vm.exec(params, &mut ext).unwrap()
	};

	assert_store(&ext, 0, "0000000000000000000000000000000000000000000000000000000000076b4b");
	assert_store(&ext, 1, "0000000000000000000000000000000000000000000000000000000000000000");
	assert_eq!(gas_left, U256::from(74_966));
}

evm_test!{test_smod: test_smod_jit, test_smod_int}
fn test_smod(factory: super::Factory) {
	let code = "650123651246236265432290076000556501236512462360009007600155".from_hex().unwrap();

	let mut params = ActionParams::default();
	params.gas = U256::from(100_000);
	params.code = Some(code);
	let mut ext = FakeExt::new();

	let gas_left = {
		let vm = factory.create();
		vm.exec(params, &mut ext).unwrap()
	};

	assert_store(&ext, 0, "0000000000000000000000000000000000000000000000000000000000076b4b");
	assert_store(&ext, 1, "0000000000000000000000000000000000000000000000000000000000000000");
	assert_eq!(gas_left, U256::from(74_966));
}

evm_test!{test_sdiv: test_sdiv_jit, test_sdiv_int}
fn test_sdiv(factory: super::Factory) {
	let code = "650123651246236265432290056000556501236512462360009005600155".from_hex().unwrap();

	let mut params = ActionParams::default();
	params.gas = U256::from(100_000);
	params.code = Some(code);
	let mut ext = FakeExt::new();

	let gas_left = {
		let vm = factory.create();
		vm.exec(params, &mut ext).unwrap()
	};

	assert_store(&ext, 0, "000000000000000000000000000000000000000000000000000000000002e0ac");
	assert_store(&ext, 1, "0000000000000000000000000000000000000000000000000000000000000000");
	assert_eq!(gas_left, U256::from(74_966));
}

evm_test!{test_exp: test_exp_jit, test_exp_int}
fn test_exp(factory: super::Factory) {
	let code = "6016650123651246230a6000556001650123651246230a6001556000650123651246230a600255".from_hex().unwrap();

	let mut params = ActionParams::default();
	params.gas = U256::from(100_000);
	params.code = Some(code);
	let mut ext = FakeExt::new();

	let gas_left = {
		let vm = factory.create();
		vm.exec(params, &mut ext).unwrap()
	};

	assert_store(&ext, 0, "90fd23767b60204c3d6fc8aec9e70a42a3f127140879c133a20129a597ed0c59");
	assert_store(&ext, 1, "0000000000000000000000000000000000000000000000000000012365124623");
	assert_store(&ext, 2, "0000000000000000000000000000000000000000000000000000000000000001");
	assert_eq!(gas_left, U256::from(39_923));
}

evm_test!{test_comparison: test_comparison_jit, test_comparison_int}
fn test_comparison(factory: super::Factory) {
	let code = "601665012365124623818181811060005511600155146002556415235412358014600355".from_hex().unwrap();

	let mut params = ActionParams::default();
	params.gas = U256::from(100_000);
	params.code = Some(code);
	let mut ext = FakeExt::new();

	let gas_left = {
		let vm = factory.create();
		vm.exec(params, &mut ext).unwrap()
	};

	assert_store(&ext, 0, "0000000000000000000000000000000000000000000000000000000000000000");
	assert_store(&ext, 1, "0000000000000000000000000000000000000000000000000000000000000001");
	assert_store(&ext, 2, "0000000000000000000000000000000000000000000000000000000000000000");
	assert_store(&ext, 3, "0000000000000000000000000000000000000000000000000000000000000001");
	assert_eq!(gas_left, U256::from(49_952));
}

evm_test!{test_signed_comparison: test_signed_comparison_jit, test_signed_comparison_int}
fn test_signed_comparison(factory: super::Factory) {
	let code = "60106000036010818112600055136001556010601060000381811260025513600355".from_hex().unwrap();

	let mut params = ActionParams::default();
	params.gas = U256::from(100_000);
	params.code = Some(code);
	let mut ext = FakeExt::new();

	let gas_left = {
		let vm = factory.create();
		vm.exec(params, &mut ext).unwrap()
	};

	assert_store(&ext, 0, "0000000000000000000000000000000000000000000000000000000000000000");
	assert_store(&ext, 1, "0000000000000000000000000000000000000000000000000000000000000001");
	assert_store(&ext, 2, "0000000000000000000000000000000000000000000000000000000000000001");
	assert_store(&ext, 3, "0000000000000000000000000000000000000000000000000000000000000000");
	assert_eq!(gas_left, U256::from(49_940));
}

evm_test!{test_bitops: test_bitops_jit, test_bitops_int}
fn test_bitops(factory: super::Factory) {
	let code = "60ff610ff08181818116600055176001551860025560008015600355198015600455600555".from_hex().unwrap();

	let mut params = ActionParams::default();
	params.gas = U256::from(150_000);
	params.code = Some(code);
	let mut ext = FakeExt::new();

	let gas_left = {
		let vm = factory.create();
		vm.exec(params, &mut ext).unwrap()
	};

	assert_store(&ext, 0, "00000000000000000000000000000000000000000000000000000000000000f0");
	assert_store(&ext, 1, "0000000000000000000000000000000000000000000000000000000000000fff");
	assert_store(&ext, 2, "0000000000000000000000000000000000000000000000000000000000000f0f");
	assert_store(&ext, 3, "0000000000000000000000000000000000000000000000000000000000000001");
	assert_store(&ext, 4, "0000000000000000000000000000000000000000000000000000000000000000");
	assert_store(&ext, 5, "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff");
	assert_eq!(gas_left, U256::from(44_937));
}

evm_test!{test_addmod_mulmod: test_addmod_mulmod_jit, test_addmod_mulmod_int}
fn test_addmod_mulmod(factory: super::Factory) {
	let code = "60ff60f060108282820860005509600155600060f0601082828208196002550919600355".from_hex().unwrap();

	let mut params = ActionParams::default();
	params.gas = U256::from(100_000);
	params.code = Some(code);
	let mut ext = FakeExt::new();

	let gas_left = {
		let vm = factory.create();
		vm.exec(params, &mut ext).unwrap()
	};

	assert_store(&ext, 0, "0000000000000000000000000000000000000000000000000000000000000001");
	assert_store(&ext, 1, "000000000000000000000000000000000000000000000000000000000000000f");
	assert_store(&ext, 2, "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff");
	assert_store(&ext, 3, "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff");
	assert_eq!(gas_left, U256::from(19_914));
}

evm_test!{test_byte: test_byte_jit, test_byte_int}
fn test_byte(factory: super::Factory) {
	let code = "60f061ffff1a600055610fff601f1a600155".from_hex().unwrap();

	let mut params = ActionParams::default();
	params.gas = U256::from(100_000);
	params.code = Some(code);
	let mut ext = FakeExt::new();

	let gas_left = {
		let vm = factory.create();
		vm.exec(params, &mut ext).unwrap()
	};

	assert_store(&ext, 0, "0000000000000000000000000000000000000000000000000000000000000000");
	assert_store(&ext, 1, "00000000000000000000000000000000000000000000000000000000000000ff");
	assert_eq!(gas_left, U256::from(74_976));
}

evm_test!{test_signextend: test_signextend_jit, test_signextend_int}
fn test_signextend(factory: super::Factory) {
	let code = "610fff60020b60005560ff60200b600155".from_hex().unwrap();

	let mut params = ActionParams::default();
	params.gas = U256::from(100_000);
	params.code = Some(code);
	let mut ext = FakeExt::new();

	let gas_left = {
		let vm = factory.create();
		vm.exec(params, &mut ext).unwrap()
	};

	assert_store(&ext, 0, "0000000000000000000000000000000000000000000000000000000000000fff");
	assert_store(&ext, 1, "00000000000000000000000000000000000000000000000000000000000000ff");
	assert_eq!(gas_left, U256::from(59_972));
}

#[test] // JIT just returns out of gas
fn test_badinstruction_int() {
	let factory = super::Factory::new(VMType::Interpreter);
	let code = "af".from_hex().unwrap();

	let mut params = ActionParams::default();
	params.gas = U256::from(100_000);
	params.code = Some(code);
	let mut ext = FakeExt::new();

	let err = {
		let vm = factory.create();
		vm.exec(params, &mut ext).unwrap_err()
	};

	match err {
		evm::Error::BadInstruction { instruction: 0xaf } => (),
		_ => assert!(false, "Expected bad instruction")
	}
}

evm_test!{test_pop: test_pop_jit, test_pop_int}
fn test_pop(factory: super::Factory) {
	let code = "60f060aa50600055".from_hex().unwrap();

	let mut params = ActionParams::default();
	params.gas = U256::from(100_000);
	params.code = Some(code);
	let mut ext = FakeExt::new();

	let gas_left = {
		let vm = factory.create();
		vm.exec(params, &mut ext).unwrap()
	};

	assert_store(&ext, 0, "00000000000000000000000000000000000000000000000000000000000000f0");
	assert_eq!(gas_left, U256::from(79_989));
}

evm_test!{test_extops: test_extops_jit, test_extops_int}
fn test_extops(factory: super::Factory) {
	let code = "5a6001555836553a600255386003553460045560016001526016590454600555".from_hex().unwrap();

	let mut params = ActionParams::default();
	params.gas = U256::from(150_000);
	params.gas_price = U256::from(0x32);
	params.value = ActionValue::Transfer(U256::from(0x99));
	params.code = Some(code);
	let mut ext = FakeExt::new();

	let gas_left = {
		let vm = factory.create();
		vm.exec(params, &mut ext).unwrap()
	};

	assert_store(&ext, 0, "0000000000000000000000000000000000000000000000000000000000000004"); // PC / CALLDATASIZE
	assert_store(&ext, 1, "00000000000000000000000000000000000000000000000000000000000249ee"); // GAS
	assert_store(&ext, 2, "0000000000000000000000000000000000000000000000000000000000000032"); // GASPRICE
	assert_store(&ext, 3, "0000000000000000000000000000000000000000000000000000000000000020"); // CODESIZE
	assert_store(&ext, 4, "0000000000000000000000000000000000000000000000000000000000000099"); // CALLVALUE
	assert_store(&ext, 5, "0000000000000000000000000000000000000000000000000000000000000032");
	assert_eq!(gas_left, U256::from(29_898));
}

evm_test!{test_jumps: test_jumps_jit, test_jumps_int}
fn test_jumps(factory: super::Factory) {
	let code = "600160015560066000555b60016000540380806000551560245760015402600155600a565b".from_hex().unwrap();

	let mut params = ActionParams::default();
	params.gas = U256::from(150_000);
	params.code = Some(code);
	let mut ext = FakeExt::new();

	let gas_left = {
		let vm = factory.create();
		vm.exec(params, &mut ext).unwrap()
	};

	assert_eq!(ext.sstore_clears, 1);
	assert_store(&ext, 0, "0000000000000000000000000000000000000000000000000000000000000000"); // 5!
	assert_store(&ext, 1, "0000000000000000000000000000000000000000000000000000000000000078"); // 5!
	assert_eq!(gas_left, U256::from(54_117));
}


evm_test!{test_calls: test_calls_jit, test_calls_int}
fn test_calls(factory: super::Factory) {
	let code = "600054602d57600160005560006000600060006050610998610100f160006000600060006050610998610100f25b".from_hex().unwrap();

	let address = Address::from(0x155);
	let code_address = Address::from(0x998);
	let mut params = ActionParams::default();
	params.gas = U256::from(150_000);
	params.code = Some(code);
	params.address = address.clone();
	let mut ext = FakeExt::new();
	ext.balances = {
		let mut s = HashMap::new();
		s.insert(params.address.clone(), params.gas);
		s
	};

	let gas_left = {
		let vm = factory.create();
		vm.exec(params, &mut ext).unwrap()
	};

	assert_set_contains(&ext.calls, &FakeCall {
		call_type: FakeCallType::Call,
		gas: U256::from(2556),
		sender_address: Some(address.clone()),
		receive_address: Some(code_address.clone()),
		value: Some(U256::from(0x50)),
		data: vec!(),
		code_address: Some(code_address.clone())
	});
	assert_set_contains(&ext.calls, &FakeCall {
		call_type: FakeCallType::Call,
		gas: U256::from(2556),
		sender_address: Some(address.clone()),
		receive_address: Some(address.clone()),
		value: Some(U256::from(0x50)),
		data: vec!(),
		code_address: Some(code_address.clone())
	});
	assert_eq!(gas_left, U256::from(91_405));
	assert_eq!(ext.calls.len(), 2);
}

fn assert_set_contains<T : Debug + Eq + PartialEq + Hash>(set: &HashSet<T>, val: &T) {
	let contains = set.contains(val);
	if !contains {
		println!("Set: {:?}", set);
		println!("Elem: {:?}", val);
	}
	assert!(contains, "Element not found in HashSet");
}

fn assert_store(ext: &FakeExt, pos: u64, val: &str) {
	assert_eq!(ext.store.get(&H256::from(pos)).unwrap(), &H256::from_str(val).unwrap());
}

