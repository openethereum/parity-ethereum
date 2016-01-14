use common::*;
use evm;
use evm::{Ext, Schedule};

struct FakeLogEntry {
	topics: Vec<H256>,
	data: Bytes
}

/// Fake externalities test structure.
///
/// Can't do recursive calls.
#[derive(Default)]
struct FakeExt {
	store: HashMap<H256, H256>,
	_balances: HashMap<Address, U256>,
	blockhashes: HashMap<U256, H256>,
	codes: HashMap<Address, Bytes>,
	logs: Vec<FakeLogEntry>,
	_suicides: HashSet<Address>,
	info: EnvInfo,
	_schedule: Schedule
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
  fn sload(&self, key: &H256) -> H256 {
		self.store.get(key).unwrap_or(&H256::new()).clone()
	}

	fn sstore(&mut self, key: H256, value: H256) {
		self.store.insert(key, value);
	}

	fn balance(&self, _address: &Address) -> U256 {
		unimplemented!();
	}

	fn blockhash(&self, number: &U256) -> H256 {
		self.blockhashes.get(number).unwrap_or(&H256::new()).clone()
	}

	fn create(&mut self, _gas: &U256, _value: &U256, _code: &[u8]) -> result::Result<(U256, Option<Address>), evm::Error> {
		unimplemented!();
	}

	fn call(&mut self, 
			_gas: &U256, 
			_call_gas: &U256, 
			_receive_address: &Address, 
			_value: &U256, 
			_data: &[u8], 
			_code_address: &Address, 
			_output: &mut [u8]) -> result::Result<U256, evm::Error> {
		unimplemented!();
	}

	fn extcode(&self, address: &Address) -> Vec<u8> {
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
		&self._schedule
	}

	fn env_info(&self) -> &EnvInfo {
		&self.info
	}
}

#[test]

fn test_stack_underflow() {
	let address = Address::from_str("0f572e5295c57f15886f9b263e2f6d2d6c7b5ec6").unwrap();
	let code = "01600055".from_hex().unwrap();

	let mut params = ActionParams::new();
	params.address = address.clone();
	params.gas = U256::from(100_000);
	params.code = code;
	let mut ext = FakeExt::new();

	let err = {
		let vm : Box<evm::Evm> = Box::new(super::interpreter::Interpreter);
		vm.exec(&params, &mut ext).unwrap_err()
	};
	
	match err {
		evm::Error::StackUnderflow {instruction: _, wanted, on_stack} => {
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

	let mut params = ActionParams::new();
	params.address = address.clone();
	params.gas = U256::from(100_000);
	params.code = code;
	let mut ext = FakeExt::new();

	let gas_left = {
		let vm = factory.create();
		vm.exec(&params, &mut ext).unwrap()
	};

	assert_eq!(gas_left, U256::from(79_988));
	assert_eq!(ext.store.get(&H256::new()).unwrap(), &H256::from_str("fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe").unwrap());
}

evm_test!{test_sha3: test_sha3_jit, test_sha3_int}
fn test_sha3(factory: super::Factory) {
	let address = Address::from_str("0f572e5295c57f15886f9b263e2f6d2d6c7b5ec6").unwrap();
	let code = "6000600020600055".from_hex().unwrap();

	let mut params = ActionParams::new();
	params.address = address.clone();
	params.gas = U256::from(100_000);
	params.code = code;
	let mut ext = FakeExt::new();

	let gas_left = {
		let vm = factory.create();
		vm.exec(&params, &mut ext).unwrap()
	};

	assert_eq!(gas_left, U256::from(79_961));
	assert_eq!(ext.store.get(&H256::new()).unwrap(), &H256::from_str("c5d2460186f7233c927e7db2dcc703c0e500b653ca82273b7bfad8045d85a470").unwrap());
}

evm_test!{test_address: test_address_jit, test_address_int}
fn test_address(factory: super::Factory) {
	let address = Address::from_str("0f572e5295c57f15886f9b263e2f6d2d6c7b5ec6").unwrap();
	let code = "30600055".from_hex().unwrap();

	let mut params = ActionParams::new();
	params.address = address.clone();
	params.gas = U256::from(100_000);
	params.code = code;
	let mut ext = FakeExt::new();

	let gas_left = {
		let vm = factory.create();
		vm.exec(&params, &mut ext).unwrap()
	};

	assert_eq!(gas_left, U256::from(79_995));
	assert_eq!(ext.store.get(&H256::new()).unwrap(), &H256::from_str("0000000000000000000000000f572e5295c57f15886f9b263e2f6d2d6c7b5ec6").unwrap());
}

evm_test!{test_origin: test_origin_jit, test_origin_int}
fn test_origin(factory: super::Factory) {
	let address = Address::from_str("0f572e5295c57f15886f9b263e2f6d2d6c7b5ec6").unwrap();
	let origin = Address::from_str("cd1722f2947def4cf144679da39c4c32bdc35681").unwrap();
	let code = "32600055".from_hex().unwrap();

	let mut params = ActionParams::new();
	params.address = address.clone();
	params.origin = origin.clone();
	params.gas = U256::from(100_000);
	params.code = code;
	let mut ext = FakeExt::new();

	let gas_left = {
		let vm = factory.create();
		vm.exec(&params, &mut ext).unwrap()
	};

	assert_eq!(gas_left, U256::from(79_995));
	assert_eq!(ext.store.get(&H256::new()).unwrap(), &H256::from_str("000000000000000000000000cd1722f2947def4cf144679da39c4c32bdc35681").unwrap());
}

evm_test!{test_sender: test_sender_jit, test_sender_int}
fn test_sender(factory: super::Factory) {
	let address = Address::from_str("0f572e5295c57f15886f9b263e2f6d2d6c7b5ec6").unwrap();
	let sender = Address::from_str("cd1722f2947def4cf144679da39c4c32bdc35681").unwrap();
	let code = "33600055".from_hex().unwrap();

	let mut params = ActionParams::new();
	params.address = address.clone();
	params.sender = sender.clone();
	params.gas = U256::from(100_000);
	params.code = code;
	let mut ext = FakeExt::new();

	let gas_left = {
		let vm = factory.create();
		vm.exec(&params, &mut ext).unwrap()
	};

	assert_eq!(gas_left, U256::from(79_995));
	assert_eq!(ext.store.get(&H256::new()).unwrap(), &H256::from_str("000000000000000000000000cd1722f2947def4cf144679da39c4c32bdc35681").unwrap());
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

	let mut params = ActionParams::new();
	params.address = address.clone();
	params.sender = sender.clone();
	params.gas = U256::from(100_000);
	params.code = code;
	let mut ext = FakeExt::new();
	ext.codes.insert(sender, sender_code);

	let gas_left = {
		let vm = factory.create();
		vm.exec(&params, &mut ext).unwrap()
	};

	assert_eq!(gas_left, U256::from(79_935));
	assert_eq!(ext.store.get(&H256::new()).unwrap(), &H256::from_str("6005600055000000000000000000000000000000000000000000000000000000").unwrap());
}

evm_test!{test_log_empty: test_log_empty_jit, test_log_empty_int}
fn test_log_empty(factory: super::Factory) {
	let address = Address::from_str("0f572e5295c57f15886f9b263e2f6d2d6c7b5ec6").unwrap();
	let code = "60006000a0".from_hex().unwrap();

	let mut params = ActionParams::new();
	params.address = address.clone();
	params.gas = U256::from(100_000);
	params.code = code;
	let mut ext = FakeExt::new();

	let gas_left = {
		let vm = factory.create();
		vm.exec(&params, &mut ext).unwrap()
	};

	assert_eq!(gas_left, U256::from(99_619));
	assert_eq!(ext.logs.len(), 1);
	assert_eq!(ext.logs[0].topics.len(), 0);
	assert_eq!(ext.logs[0].data, vec![]);
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

	let mut params = ActionParams::new();
	params.address = address.clone();
	params.sender = sender.clone();
	params.gas = U256::from(100_000);
	params.code = code;
	let mut ext = FakeExt::new();

	let gas_left = {
		let vm = factory.create();
		vm.exec(&params, &mut ext).unwrap()
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

	let mut params = ActionParams::new();
	params.address = address.clone();
	params.gas = U256::from(100_000);
	params.code = code;
	let mut ext = FakeExt::new();
	ext.blockhashes.insert(U256::zero(), blockhash.clone());

	let gas_left = {
		let vm = factory.create();
		vm.exec(&params, &mut ext).unwrap()
	};

	assert_eq!(gas_left, U256::from(79_974));
	assert_eq!(ext.store.get(&H256::new()).unwrap(), &blockhash);
}

evm_test!{test_calldataload: test_calldataload_jit, test_calldataload_int}
fn test_calldataload(factory: super::Factory) {
	let address = Address::from_str("0f572e5295c57f15886f9b263e2f6d2d6c7b5ec6").unwrap();
	let code = "600135600055".from_hex().unwrap();
	let data = "0123ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff23".from_hex().unwrap();

	let mut params = ActionParams::new();
	params.address = address.clone();
	params.gas = U256::from(100_000);
	params.code = code;
	params.data = data;
	let mut ext = FakeExt::new();

	let gas_left = {
		let vm = factory.create();
		vm.exec(&params, &mut ext).unwrap()
	};

	assert_eq!(gas_left, U256::from(79_991));
	assert_eq!(ext.store.get(&H256::new()).unwrap(), &H256::from_str("23ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff23").unwrap());

}

evm_test!{test_author: test_author_jit, test_author_int}
fn test_author(factory: super::Factory) {
	let author = Address::from_str("0f572e5295c57f15886f9b263e2f6d2d6c7b5ec6").unwrap();
	let code = "41600055".from_hex().unwrap();

	let mut params = ActionParams::new();
	params.gas = U256::from(100_000);
	params.code = code;
	let mut ext = FakeExt::new();
	ext.info.author = author;

	let gas_left = {
		let vm = factory.create();
		vm.exec(&params, &mut ext).unwrap()
	};

	assert_eq!(gas_left, U256::from(79_995));
	assert_eq!(ext.store.get(&H256::new()).unwrap(), &H256::from_str("0000000000000000000000000f572e5295c57f15886f9b263e2f6d2d6c7b5ec6").unwrap());
}

evm_test!{test_timestamp: test_timestamp_jit, test_timestamp_int}
fn test_timestamp(factory: super::Factory) {
	let timestamp = 0x1234; 
	let code = "42600055".from_hex().unwrap();

	let mut params = ActionParams::new();
	params.gas = U256::from(100_000);
	params.code = code;
	let mut ext = FakeExt::new();
	ext.info.timestamp = timestamp;

	let gas_left = {
		let vm = factory.create();
		vm.exec(&params, &mut ext).unwrap()
	};

	assert_eq!(gas_left, U256::from(79_995));
	assert_eq!(ext.store.get(&H256::new()).unwrap(), &H256::from_str("0000000000000000000000000000000000000000000000000000000000001234").unwrap());
}

evm_test!{test_number: test_number_jit, test_number_int}
fn test_number(factory: super::Factory) {
	let number = 0x1234; 
	let code = "43600055".from_hex().unwrap();

	let mut params = ActionParams::new();
	params.gas = U256::from(100_000);
	params.code = code;
	let mut ext = FakeExt::new();
	ext.info.number = number;

	let gas_left = {
		let vm = factory.create();
		vm.exec(&params, &mut ext).unwrap()
	};

	assert_eq!(gas_left, U256::from(79_995));
	assert_eq!(ext.store.get(&H256::new()).unwrap(), &H256::from_str("0000000000000000000000000000000000000000000000000000000000001234").unwrap());
}

evm_test!{test_difficulty: test_difficulty_jit, test_difficulty_int}
fn test_difficulty(factory: super::Factory) {
	let difficulty = U256::from(0x1234);
	let code = "44600055".from_hex().unwrap();

	let mut params = ActionParams::new();
	params.gas = U256::from(100_000);
	params.code = code;
	let mut ext = FakeExt::new();
	ext.info.difficulty = difficulty;

	let gas_left = {
		let vm = factory.create();
		vm.exec(&params, &mut ext).unwrap()
	};

	assert_eq!(gas_left, U256::from(79_995));
	assert_eq!(ext.store.get(&H256::new()).unwrap(), &H256::from_str("0000000000000000000000000000000000000000000000000000000000001234").unwrap());
}

evm_test!{test_gas_limit: test_gas_limit_jit, test_gas_limit_int}
fn test_gas_limit(factory: super::Factory) {
	let gas_limit = U256::from(0x1234);
	let code = "45600055".from_hex().unwrap();

	let mut params = ActionParams::new();
	params.gas = U256::from(100_000);
	params.code = code;
	let mut ext = FakeExt::new();
	ext.info.gas_limit = gas_limit;

	let gas_left = {
		let vm = factory.create();
		vm.exec(&params, &mut ext).unwrap()
	};

	assert_eq!(gas_left, U256::from(79_995));
	assert_eq!(ext.store.get(&H256::new()).unwrap(), &H256::from_str("0000000000000000000000000000000000000000000000000000000000001234").unwrap());
}

