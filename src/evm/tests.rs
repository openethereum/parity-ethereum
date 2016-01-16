use common::*;
use evm;
use evm::{Ext, Schedule, Factory};

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
	info: EnvInfo
}

impl FakeExt {
	fn new() -> Self { FakeExt::default() }
}

impl Ext for FakeExt {
	fn storage_at(&self, key: &H256) -> H256 {
		self.store.get(key).unwrap_or(&H256::new()).clone()
	}

	fn set_storage_at(&mut self, key: H256, value: H256) {
		self.store.insert(key, value);
	}

	fn exists(&self, address: &Address) -> bool {
		unimplemented!();
	}

	fn balance(&self, _address: &Address) -> U256 {
		unimplemented!();
	}

	fn blockhash(&self, number: &U256) -> H256 {
		self.blockhashes.get(number).unwrap_or(&H256::new()).clone()
	}

	fn create(&mut self, _gas: &U256, _value: &U256, _code: &[u8]) -> (U256, Option<Address>) {
		unimplemented!();
	}

	fn call(&mut self, 
			_gas: &U256, 
			_call_gas: &U256, 
			_receive_address: &Address, 
			_value: &U256, 
			_data: &[u8], 
			_code_address: &Address, 
			_output: &mut [u8]) -> result::Result<(U256, bool), evm::Error> {
		unimplemented!();
	}

	fn extcode(&self, address: &Address) -> Vec<u8> {
		self.codes.get(address).unwrap_or(&Bytes::new()).clone()
	}

	fn log(&mut self, topics: Vec<H256>, data: Bytes) {
		self.logs.push(FakeLogEntry {
			topics: topics,
			data: data
		});
	}

	fn ret(&mut self, _gas: &U256, _data: &[u8]) -> result::Result<U256, evm::Error> {
		unimplemented!();
	}

	fn suicide(&mut self, _refund_address: &Address) {
		unimplemented!();
	}

	fn schedule(&self) -> &Schedule {
		unimplemented!();
	}

	fn env_info(&self) -> &EnvInfo {
		&self.info
	}

	fn depth(&self) -> usize {
		unimplemented!();
	}
}

#[test]
fn test_add() {
	let address = Address::from_str("0f572e5295c57f15886f9b263e2f6d2d6c7b5ec6").unwrap();
	let code = "7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff01600055".from_hex().unwrap();

	let mut params = ActionParams::new();
	params.address = address.clone();
	params.gas = U256::from(100_000);
	params.code = Some(code);
	let mut ext = FakeExt::new();

	let gas_left = {
		let vm = Factory::create();
		vm.exec(&params, &mut ext).unwrap()
	};

	assert_eq!(gas_left, U256::from(79_988));
	assert_eq!(ext.store.get(&H256::new()).unwrap(), &H256::from_str("fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe").unwrap());
}

#[test]
fn test_sha3() {
	let address = Address::from_str("0f572e5295c57f15886f9b263e2f6d2d6c7b5ec6").unwrap();
	let code = "6000600020600055".from_hex().unwrap();

	let mut params = ActionParams::new();
	params.address = address.clone();
	params.gas = U256::from(100_000);
	params.code = Some(code);
	let mut ext = FakeExt::new();

	let gas_left = {
		let vm = Factory::create();
		vm.exec(&params, &mut ext).unwrap()
	};

	assert_eq!(gas_left, U256::from(79_961));
	assert_eq!(ext.store.get(&H256::new()).unwrap(), &H256::from_str("c5d2460186f7233c927e7db2dcc703c0e500b653ca82273b7bfad8045d85a470").unwrap());
}

#[test]
fn test_address() {
	let address = Address::from_str("0f572e5295c57f15886f9b263e2f6d2d6c7b5ec6").unwrap();
	let code = "30600055".from_hex().unwrap();

	let mut params = ActionParams::new();
	params.address = address.clone();
	params.gas = U256::from(100_000);
	params.code = Some(code);
	let mut ext = FakeExt::new();

	let gas_left = {
		let vm = Factory::create();
		vm.exec(&params, &mut ext).unwrap()
	};

	assert_eq!(gas_left, U256::from(79_995));
	assert_eq!(ext.store.get(&H256::new()).unwrap(), &H256::from_str("0000000000000000000000000f572e5295c57f15886f9b263e2f6d2d6c7b5ec6").unwrap());
}

#[test]
fn test_origin() {
	let address = Address::from_str("0f572e5295c57f15886f9b263e2f6d2d6c7b5ec6").unwrap();
	let origin = Address::from_str("cd1722f2947def4cf144679da39c4c32bdc35681").unwrap();
	let code = "32600055".from_hex().unwrap();

	let mut params = ActionParams::new();
	params.address = address.clone();
	params.origin = origin.clone();
	params.gas = U256::from(100_000);
	params.code = Some(code);
	let mut ext = FakeExt::new();

	let gas_left = {
		let vm = Factory::create();
		vm.exec(&params, &mut ext).unwrap()
	};

	assert_eq!(gas_left, U256::from(79_995));
	assert_eq!(ext.store.get(&H256::new()).unwrap(), &H256::from_str("000000000000000000000000cd1722f2947def4cf144679da39c4c32bdc35681").unwrap());
}

#[test]
fn test_sender() {
	let address = Address::from_str("0f572e5295c57f15886f9b263e2f6d2d6c7b5ec6").unwrap();
	let sender = Address::from_str("cd1722f2947def4cf144679da39c4c32bdc35681").unwrap();
	let code = "33600055".from_hex().unwrap();

	let mut params = ActionParams::new();
	params.address = address.clone();
	params.sender = sender.clone();
	params.gas = U256::from(100_000);
	params.code = Some(code);
	let mut ext = FakeExt::new();

	let gas_left = {
		let vm = Factory::create();
		vm.exec(&params, &mut ext).unwrap()
	};

	assert_eq!(gas_left, U256::from(79_995));
	assert_eq!(ext.store.get(&H256::new()).unwrap(), &H256::from_str("000000000000000000000000cd1722f2947def4cf144679da39c4c32bdc35681").unwrap());
}

#[test]
fn test_extcodecopy() {
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
	params.code = Some(code);
	let mut ext = FakeExt::new();
	ext.codes.insert(sender, sender_code);

	let gas_left = {
		let vm = Factory::create();
		vm.exec(&params, &mut ext).unwrap()
	};

	assert_eq!(gas_left, U256::from(79_935));
	assert_eq!(ext.store.get(&H256::new()).unwrap(), &H256::from_str("6005600055000000000000000000000000000000000000000000000000000000").unwrap());
}

#[test]
fn test_log_empty() {
	let address = Address::from_str("0f572e5295c57f15886f9b263e2f6d2d6c7b5ec6").unwrap();
	let code = "60006000a0".from_hex().unwrap();

	let mut params = ActionParams::new();
	params.address = address.clone();
	params.gas = U256::from(100_000);
	params.code = Some(code);
	let mut ext = FakeExt::new();

	let gas_left = {
		let vm = Factory::create();
		vm.exec(&params, &mut ext).unwrap()
	};

	assert_eq!(gas_left, U256::from(99_619));
	assert_eq!(ext.logs.len(), 1);
	assert_eq!(ext.logs[0].topics.len(), 0);
	assert_eq!(ext.logs[0].data, vec![]);
}

#[test]
fn test_log_sender() {
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
	params.code = Some(code);
	let mut ext = FakeExt::new();

	let gas_left = {
		let vm = Factory::create();
		vm.exec(&params, &mut ext).unwrap()
	};

	assert_eq!(gas_left, U256::from(98_974));
	assert_eq!(ext.logs.len(), 1);
	assert_eq!(ext.logs[0].topics.len(), 1);
	assert_eq!(ext.logs[0].topics[0], H256::from_str("000000000000000000000000cd1722f3947def4cf144679da39c4c32bdc35681").unwrap());
	assert_eq!(ext.logs[0].data, "ff00000000000000000000000000000000000000000000000000000000000000".from_hex().unwrap());
}

#[test]
fn test_blockhash() {
	let address = Address::from_str("0f572e5295c57f15886f9b263e2f6d2d6c7b5ec6").unwrap();
	let code = "600040600055".from_hex().unwrap();
	let blockhash = H256::from_str("123400000000000000000000cd1722f2947def4cf144679da39c4c32bdc35681").unwrap();

	let mut params = ActionParams::new();
	params.address = address.clone();
	params.gas = U256::from(100_000);
	params.code = Some(code);
	let mut ext = FakeExt::new();
	ext.blockhashes.insert(U256::zero(), blockhash.clone());

	let gas_left = {
		let vm = Factory::create();
		vm.exec(&params, &mut ext).unwrap()
	};

	assert_eq!(gas_left, U256::from(79_974));
	assert_eq!(ext.store.get(&H256::new()).unwrap(), &blockhash);
}

#[test]
fn test_calldataload() {
	let address = Address::from_str("0f572e5295c57f15886f9b263e2f6d2d6c7b5ec6").unwrap();
	let code = "600135600055".from_hex().unwrap();
	let data = "0123ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff23".from_hex().unwrap();

	let mut params = ActionParams::new();
	params.address = address.clone();
	params.gas = U256::from(100_000);
	params.code = Some(code);
	params.data = Some(data);
	let mut ext = FakeExt::new();

	let gas_left = {
		let vm = Factory::create();
		vm.exec(&params, &mut ext).unwrap()
	};

	assert_eq!(gas_left, U256::from(79_991));
	assert_eq!(ext.store.get(&H256::new()).unwrap(), &H256::from_str("23ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff23").unwrap());

}

#[test]
fn test_author() {
	let author = Address::from_str("0f572e5295c57f15886f9b263e2f6d2d6c7b5ec6").unwrap();
	let code = "41600055".from_hex().unwrap();

	let mut params = ActionParams::new();
	params.gas = U256::from(100_000);
	params.code = Some(code);
	let mut ext = FakeExt::new();
	ext.info.author = author;

	let gas_left = {
		let vm = Factory::create();
		vm.exec(&params, &mut ext).unwrap()
	};

	assert_eq!(gas_left, U256::from(79_995));
	assert_eq!(ext.store.get(&H256::new()).unwrap(), &H256::from_str("0000000000000000000000000f572e5295c57f15886f9b263e2f6d2d6c7b5ec6").unwrap());
}

#[test]
fn test_timestamp() {
	let timestamp = 0x1234; 
	let code = "42600055".from_hex().unwrap();

	let mut params = ActionParams::new();
	params.gas = U256::from(100_000);
	params.code = Some(code);
	let mut ext = FakeExt::new();
	ext.info.timestamp = timestamp;

	let gas_left = {
		let vm = Factory::create();
		vm.exec(&params, &mut ext).unwrap()
	};

	assert_eq!(gas_left, U256::from(79_995));
	assert_eq!(ext.store.get(&H256::new()).unwrap(), &H256::from_str("0000000000000000000000000000000000000000000000000000000000001234").unwrap());
}

#[test]
fn test_number() {
	let number = 0x1234; 
	let code = "43600055".from_hex().unwrap();

	let mut params = ActionParams::new();
	params.gas = U256::from(100_000);
	params.code = Some(code);
	let mut ext = FakeExt::new();
	ext.info.number = number;

	let gas_left = {
		let vm = Factory::create();
		vm.exec(&params, &mut ext).unwrap()
	};

	assert_eq!(gas_left, U256::from(79_995));
	assert_eq!(ext.store.get(&H256::new()).unwrap(), &H256::from_str("0000000000000000000000000000000000000000000000000000000000001234").unwrap());
}

#[test]
fn test_difficulty() {
	let difficulty = U256::from(0x1234);
	let code = "44600055".from_hex().unwrap();

	let mut params = ActionParams::new();
	params.gas = U256::from(100_000);
	params.code = Some(code);
	let mut ext = FakeExt::new();
	ext.info.difficulty = difficulty;

	let gas_left = {
		let vm = Factory::create();
		vm.exec(&params, &mut ext).unwrap()
	};

	assert_eq!(gas_left, U256::from(79_995));
	assert_eq!(ext.store.get(&H256::new()).unwrap(), &H256::from_str("0000000000000000000000000000000000000000000000000000000000001234").unwrap());
}

#[test]
fn test_gas_limit() {
	let gas_limit = U256::from(0x1234);
	let code = "45600055".from_hex().unwrap();

	let mut params = ActionParams::new();
	params.gas = U256::from(100_000);
	params.code = Some(code);
	let mut ext = FakeExt::new();
	ext.info.gas_limit = gas_limit;

	let gas_left = {
		let vm = Factory::create();
		vm.exec(&params, &mut ext).unwrap()
	};

	assert_eq!(gas_left, U256::from(79_995));
	assert_eq!(ext.store.get(&H256::new()).unwrap(), &H256::from_str("0000000000000000000000000000000000000000000000000000000000001234").unwrap());
}
