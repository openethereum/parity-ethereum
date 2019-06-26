// Copyright 2015-2019 Parity Technologies (UK) Ltd.
// This file is part of Parity Ethereum.

// Parity Ethereum is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity Ethereum is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity Ethereum.  If not, see <http://www.gnu.org/licenses/>.

use std::sync::Arc;
use std::collections::HashMap;
use std::str::FromStr;
use byteorder::{LittleEndian, ByteOrder};
use ethereum_types::{H256, U256, Address, BigEndianHash as _};

use super::WasmInterpreter;
use vm::{self, Exec, GasLeft, ActionParams, ActionValue, CreateContractAddress};
use vm::tests::{FakeCall, FakeExt, FakeCallType};

macro_rules! load_sample {
	($name: expr) => {
		include_bytes!(concat!("../../res/wasm-tests/compiled/", $name)).to_vec()
	}
}

macro_rules! reqrep_test {
	($name: expr, $input: expr) => {
		reqrep_test!($name, $input, vm::EnvInfo::default(), HashMap::new())
	};
	($name: expr, $input: expr, $info: expr, $block_hashes: expr) => {
		{
			let _ = ::env_logger::try_init();
			let code = load_sample!($name);

			let mut params = ActionParams::default();
			params.gas = U256::from(100_000);
			params.code = Some(Arc::new(code));
			params.data = Some($input);

			let mut fake_ext = FakeExt::new().with_wasm();
			fake_ext.info = $info;
			fake_ext.blockhashes = $block_hashes;

			let mut interpreter = wasm_interpreter(params);
			interpreter.exec(&mut fake_ext).ok().unwrap()
				.map(|result| match result {
					GasLeft::Known(_) => { panic!("Test is expected to return payload to check"); },
					GasLeft::NeedsReturn { gas_left: gas, data: result, apply_state: _apply } => (gas, result.to_vec()),
				})
		}
	};
}

fn test_finalize(res: Result<GasLeft, vm::Error>) -> Result<U256, vm::Error> {
	match res {
		Ok(GasLeft::Known(gas)) => Ok(gas),
		Ok(GasLeft::NeedsReturn{..}) => unimplemented!(), // since ret is unimplemented.
		Err(e) => Err(e),
	}
}

fn wasm_interpreter(params: ActionParams) -> Box<WasmInterpreter> {
	Box::new(WasmInterpreter::new(params))
}

/// Empty contract does almost nothing except producing 1 (one) local node debug log message
#[test]
fn empty() {
	let code = load_sample!("empty.wasm");
	let address: Address = "0f572e5295c57f15886f9b263e2f6d2d6c7b5ec6".parse().unwrap();

	let mut params = ActionParams::default();
	params.address = address.clone();
	params.gas = U256::from(100_000);
	params.code = Some(Arc::new(code));
	let mut ext = FakeExt::new().with_wasm();

	let gas_left = {
		let mut interpreter = wasm_interpreter(params);
		test_finalize(interpreter.exec(&mut ext).ok().unwrap()).unwrap()
	};

	assert_eq!(gas_left, U256::from(96_926));
}

// This test checks if the contract deserializes payload header properly.
//   Contract is provided with receiver(address), sender, origin and transaction value
//   logger.wasm writes all these provided fixed header fields to some arbitrary storage keys.
#[test]
fn logger() {
	let _ = ::env_logger::try_init();

	let code = load_sample!("logger.wasm");
	let address: Address = "0f572e5295c57f15886f9b263e2f6d2d6c7b5ec6".parse().unwrap();
	let sender: Address = "0d0d0d0d0d0d0d0d0d0d0d0d0d0d0d0d0d0d0d0d".parse().unwrap();
	let origin: Address = "0102030405060708090a0b0c0d0e0f1011121314".parse().unwrap();

	let mut params = ActionParams::default();
	params.address = address.clone();
	params.sender = sender.clone();
	params.origin = origin.clone();
	params.gas = U256::from(100_000);
	params.value = ActionValue::transfer(1_000_000_000);
	params.code = Some(Arc::new(code));
	let mut ext = FakeExt::new().with_wasm();

	let gas_left = {
		let mut interpreter = wasm_interpreter(params);
		test_finalize(interpreter.exec(&mut ext).ok().unwrap()).unwrap()
	};

	let address_val: H256 = address.into();
	assert_eq!(
		ext.store.get(&"0100000000000000000000000000000000000000000000000000000000000000".parse().unwrap()).expect("storage key to exist"),
		&address_val,
		"Logger sets 0x01 key to the provided address"
	);
	let sender_val: H256 = sender.into();
	assert_eq!(
		ext.store.get(&"0200000000000000000000000000000000000000000000000000000000000000".parse().unwrap()).expect("storage key to exist"),
		&sender_val,
		"Logger sets 0x02 key to the provided sender"
	);
	let origin_val: H256 = origin.into();
	assert_eq!(
		ext.store.get(&"0300000000000000000000000000000000000000000000000000000000000000".parse().unwrap()).expect("storage key to exist"),
		&origin_val,
		"Logger sets 0x03 key to the provided origin"
	);
	assert_eq!(
		ext.store
			.get(&"0400000000000000000000000000000000000000000000000000000000000000".parse().unwrap())
			.expect("storage key to exist")
			.into_uint(),
		U256::from(1_000_000_000),
		"Logger sets 0x04 key to the trasferred value"
	);
	assert_eq!(gas_left, U256::from(17_716));
}

// This test checks if the contract can allocate memory and pass pointer to the result stream properly.
//   1. Contract is being provided with the call descriptor ptr
//   2. Descriptor ptr is 16 byte length
//   3. The last 8 bytes of call descriptor is the space for the contract to fill [result_ptr[4], result_len[4]]
//      if it has any result.
#[test]
fn identity() {
	let _ = ::env_logger::try_init();

	let code = load_sample!("identity.wasm");
	let sender: Address = "01030507090b0d0f11131517191b1d1f21232527".parse().unwrap();

	let mut params = ActionParams::default();
	params.sender = sender.clone();
	params.gas = U256::from(100_000);
	params.code = Some(Arc::new(code));
	let mut ext = FakeExt::new().with_wasm();

	let (gas_left, result) = {
		let mut interpreter = wasm_interpreter(params);
		let result = interpreter.exec(&mut ext).ok().unwrap().expect("Interpreter to execute without any errors");
		match result {
			GasLeft::Known(_) => { panic!("Identity contract should return payload"); },
			GasLeft::NeedsReturn { gas_left: gas, data: result, apply_state: _apply } => (gas, result.to_vec()),
		}
	};

	assert_eq!(
		Address::from_slice(&result),
		sender,
		"Idenity test contract does not return the sender passed"
	);
	assert_eq!(gas_left, U256::from(98_419));
}

// Dispersion test sends byte array and expect the contract to 'disperse' the original elements with
// their modulo 19 dopant.
// The result is always twice as long as the input.
// This also tests byte-perfect memory allocation and in/out ptr lifecycle.
#[test]
fn dispersion() {
	let _ = ::env_logger::try_init();

	let code = load_sample!("dispersion.wasm");

	let mut params = ActionParams::default();
	params.gas = U256::from(100_000);
	params.code = Some(Arc::new(code));
	params.data = Some(vec![
		0u8, 125, 197, 255, 19
	]);
	let mut ext = FakeExt::new().with_wasm();

	let (gas_left, result) = {
		let mut interpreter = wasm_interpreter(params);
		let result = interpreter.exec(&mut ext).ok().unwrap().expect("Interpreter to execute without any errors");
		match result {
			GasLeft::Known(_) => { panic!("Dispersion routine should return payload"); },
			GasLeft::NeedsReturn { gas_left: gas, data: result, apply_state: _apply } => (gas, result.to_vec()),
		}
	};

	assert_eq!(
		result,
		vec![0u8, 0, 125, 11, 197, 7, 255, 8, 19, 0]
	);
	assert_eq!(gas_left, U256::from(92_377));
}

#[test]
fn suicide_not() {
	let code = load_sample!("suicidal.wasm");

	let mut params = ActionParams::default();
	params.gas = U256::from(100_000);
	params.code = Some(Arc::new(code));
	params.data = Some(vec![
		0u8
	]);
	let mut ext = FakeExt::new().with_wasm();

	let (gas_left, result) = {
		let mut interpreter = wasm_interpreter(params);
		let result = interpreter.exec(&mut ext).ok().unwrap().expect("Interpreter to execute without any errors");
		match result {
			GasLeft::Known(_) => { panic!("Suicidal contract should return payload when had not actualy killed himself"); },
			GasLeft::NeedsReturn { gas_left: gas, data: result, apply_state: _apply } => (gas, result.to_vec()),
		}
	};

	assert_eq!(
		result,
		vec![0u8]
	);
	assert_eq!(gas_left, U256::from(93_378));
}

#[test]
fn suicide() {
	let _ = ::env_logger::try_init();

	let code = load_sample!("suicidal.wasm");

	let refund: Address = "01030507090b0d0f11131517191b1d1f21232527".parse().unwrap();
	let mut params = ActionParams::default();
	params.gas = U256::from(100_000);
	params.code = Some(Arc::new(code));

	let mut args = vec![127u8];
	args.extend(refund.as_bytes().to_vec());
	params.data = Some(args);

	let mut ext = FakeExt::new().with_wasm();

	let gas_left = {
		let mut interpreter = wasm_interpreter(params);
		let result = interpreter.exec(&mut ext).ok().unwrap().expect("Interpreter to execute without any errors");
		match result {
			GasLeft::Known(gas) => gas,
			GasLeft::NeedsReturn { .. } => {
				panic!("Suicidal contract should not return anything when had killed itself");
			},
		}
	};

	assert!(ext.suicides.contains(&refund));
	assert_eq!(gas_left, U256::from(93_346));
}

#[test]
fn create() {
	let _ = ::env_logger::try_init();

	let mut params = ActionParams::default();
	params.gas = U256::from(100_000);
	params.code = Some(Arc::new(load_sample!("creator.wasm")));
	params.data = Some(vec![0u8, 2, 4, 8, 16, 32, 64, 128]);
	params.value = ActionValue::transfer(1_000_000_000);

	let mut ext = FakeExt::new().with_wasm();
	ext.schedule.wasm.as_mut().unwrap().have_create2 = true;

	let gas_left = {
		let mut interpreter = wasm_interpreter(params);
		let result = interpreter.exec(&mut ext).ok().unwrap().expect("Interpreter to execute without any errors");
		match result {
			GasLeft::Known(_) => {
				panic!("Create contract always return 40 bytes of the creation address, or in the case where it fails, return 40 bytes of zero.");
			},
			GasLeft::NeedsReturn { gas_left, data, apply_state } => {
				assert!(apply_state);
				assert_eq!(data.as_ref(), [0u8; 40].as_ref()); // FakeExt never succeeds in create.
				gas_left
			},
		}
	};

	trace!(target: "wasm", "fake_calls: {:?}", &ext.calls);
	assert!(ext.calls.contains(
		&FakeCall {
			call_type: FakeCallType::Create,
			create_scheme: Some(CreateContractAddress::FromSenderAndCodeHash),
			gas: U256::from(49_674),
			sender_address: None,
			receive_address: None,
			value: Some((1_000_000_000 / 2).into()),
			data: vec![0u8, 2, 4, 8, 16, 32, 64, 128],
			code_address: None,
		}
	));
	let mut salt = [0u8; 32];
	salt[0] = 5;
	let salt = H256::from_slice(salt.as_ref());
	assert!(ext.calls.contains(
		&FakeCall {
			call_type: FakeCallType::Create,
			create_scheme: Some(CreateContractAddress::FromSenderSaltAndCodeHash(salt)),
			gas: U256::from(6039),
			sender_address: None,
			receive_address: None,
			value: Some((1_000_000_000 / 2).into()),
			data: vec![0u8, 2, 4, 8, 16, 32, 64, 128],
			code_address: None,
		}
	));
	assert_eq!(gas_left, U256::from(5974));
}

#[test]
fn call_msg() {
	let _ = ::env_logger::try_init();

	let sender: Address = "01030507090b0d0f11131517191b1d1f21232527".parse().unwrap();
	let receiver: Address = "0f572e5295c57f15886f9b263e2f6d2d6c7b5ec6".parse().unwrap();
	let contract_address: Address = "0d461d4174b4ae35775c4a342f1e5e1e4e6c4db5".parse().unwrap();

	let mut params = ActionParams::default();
	params.sender = sender.clone();
	params.address = receiver.clone();
	params.code_address = contract_address.clone();
	params.gas = U256::from(100_000);
	params.code = Some(Arc::new(load_sample!("call.wasm")));
	params.data = Some(Vec::new());

	let mut ext = FakeExt::new().with_wasm();
	ext.balances.insert(receiver.clone(), U256::from(10000000000u64));

	let gas_left = {
		let mut interpreter = wasm_interpreter(params);
		let result = interpreter.exec(&mut ext).ok().unwrap().expect("Interpreter to execute without any errors");
		match result {
			GasLeft::Known(gas_left) => gas_left,
			GasLeft::NeedsReturn { .. } => { panic!("Call test should not return payload"); },
		}
	};

	trace!(target: "wasm", "fake_calls: {:?}", &ext.calls);
	assert!(ext.calls.contains(
		&FakeCall {
			call_type: FakeCallType::Call,
			create_scheme: None,
			gas: U256::from(33_000),
			sender_address: Some(receiver),
			receive_address: Some(Address::from([99, 88, 77, 66, 55, 44, 33, 22, 11, 0, 11, 22, 33, 44, 55, 66, 77, 88, 99, 0])),
			value: Some(1000000000.into()),
			data: vec![129u8, 123, 113, 107, 101, 97],
			code_address: Some(Address::from([99, 88, 77, 66, 55, 44, 33, 22, 11, 0, 11, 22, 33, 44, 55, 66, 77, 88, 99, 0])),
		}
	));

	assert_eq!(gas_left, U256::from(91_672));
}

// The same as `call_msg`, but send a `pwasm_ethereum::gasleft`
// value as `gas` argument to the inner pwasm_ethereum::call
#[test]
fn call_msg_gasleft() {
	let _ = ::env_logger::try_init();

	let sender: Address = "01030507090b0d0f11131517191b1d1f21232527".parse().unwrap();
	let receiver: Address = "0f572e5295c57f15886f9b263e2f6d2d6c7b5ec6".parse().unwrap();
	let contract_address: Address = "0d461d4174b4ae35775c4a342f1e5e1e4e6c4db5".parse().unwrap();

	let mut params = ActionParams::default();
	params.sender = sender.clone();
	params.address = receiver.clone();
	params.code_address = contract_address.clone();
	params.gas = U256::from(100_000);
	params.code = Some(Arc::new(load_sample!("call_gasleft.wasm")));
	params.data = Some(Vec::new());

	let mut ext = FakeExt::new().with_wasm();
	ext.schedule.wasm.as_mut().unwrap().have_gasleft = true;
	ext.balances.insert(receiver.clone(), U256::from(10000000000u64));

	let gas_left = {
		let mut interpreter = wasm_interpreter(params);
		let result = interpreter.exec(&mut ext).ok().unwrap().expect("Interpreter to execute without any errors");
		match result {
			GasLeft::Known(gas_left) => gas_left,
			GasLeft::NeedsReturn { .. } => { panic!("Call test should not return payload"); },
		}
	};

	trace!(target: "wasm", "fake_calls: {:?}", &ext.calls);
	assert!(ext.calls.contains(
		&FakeCall {
			call_type: FakeCallType::Call,
			create_scheme: None,
			gas: U256::from(91_165),
			sender_address: Some(receiver),
			receive_address: Some(Address::from([99, 88, 77, 66, 55, 44, 33, 22, 11, 0, 11, 22, 33, 44, 55, 66, 77, 88, 99, 0])),
			value: Some(1000000000.into()),
			data: vec![129u8, 123, 113, 107, 101, 97],
			code_address: Some(Address::from([99, 88, 77, 66, 55, 44, 33, 22, 11, 0, 11, 22, 33, 44, 55, 66, 77, 88, 99, 0])),
		}
	));

	assert_eq!(gas_left, U256::from(91_671));
}

#[test]
fn call_code() {
	let _ = ::env_logger::try_init();

	let sender: Address = "01030507090b0d0f11131517191b1d1f21232527".parse().unwrap();
	let receiver: Address = "0f572e5295c57f15886f9b263e2f6d2d6c7b5ec6".parse().unwrap();

	let mut params = ActionParams::default();
	params.sender = sender.clone();
	params.address = receiver.clone();
	params.gas = U256::from(100_000);
	params.code = Some(Arc::new(load_sample!("call_code.wasm")));
	params.data = Some(Vec::new());
	params.value = ActionValue::transfer(1_000_000_000);

	let mut ext = FakeExt::new().with_wasm();

	let (gas_left, result) = {
		let mut interpreter = wasm_interpreter(params);
		let result = interpreter.exec(&mut ext).ok().unwrap().expect("Interpreter to execute without any errors");
		match result {
			GasLeft::Known(_) => { panic!("Call test should return payload"); },
			GasLeft::NeedsReturn { gas_left: gas, data: result, apply_state: _apply } => (gas, result.to_vec()),
		}
	};

	trace!(target: "wasm", "fake_calls: {:?}", &ext.calls);
	assert!(ext.calls.contains(
		&FakeCall {
			call_type: FakeCallType::Call,
			create_scheme: None,
			gas: U256::from(20_000),
			sender_address: Some(sender),
			receive_address: Some(receiver),
			value: None,
			data: vec![1u8, 2, 3, 5, 7, 11],
			code_address: Some("0d13710000000000000000000000000000000000".parse().unwrap()),
		}
	));

	// siphash result
	let res = LittleEndian::read_u32(&result[..]);
	assert_eq!(res, 4198595614);
	assert_eq!(gas_left, U256::from(90_037));
}

#[test]
fn call_static() {
	let _ = ::env_logger::try_init();

	let sender: Address = "0f572e5295c57f15886f9b263e2f6d2d6c7b5ec6".parse().unwrap();
	let receiver: Address = "01030507090b0d0f11131517191b1d1f21232527".parse().unwrap();
	let contract_address: Address = "0d461d4174b4ae35775c4a342f1e5e1e4e6c4db5".parse().unwrap();

	let mut params = ActionParams::default();
	params.sender = sender.clone();
	params.address = receiver.clone();
	params.gas = U256::from(100_000);
	params.code = Some(Arc::new(load_sample!("call_static.wasm")));
	params.data = Some(Vec::new());
	params.value = ActionValue::transfer(1_000_000_000);
	params.code_address = contract_address.clone();

	let mut ext = FakeExt::new().with_wasm();

	let (gas_left, result) = {
		let mut interpreter = wasm_interpreter(params);
		let result = interpreter.exec(&mut ext).ok().unwrap().expect("Interpreter to execute without any errors");
		match result {
			GasLeft::Known(_) => { panic!("Static call test should return payload"); },
			GasLeft::NeedsReturn { gas_left: gas, data: result, apply_state: _apply } => (gas, result.to_vec()),
		}
	};

	trace!(target: "wasm", "fake_calls: {:?}", &ext.calls);
	assert!(ext.calls.contains(
		&FakeCall {
			call_type: FakeCallType::Call,
			create_scheme: None,
			gas: U256::from(20_000),
			sender_address: Some(receiver),
			receive_address: Some("13077bfb00000000000000000000000000000000".parse().unwrap()),
			value: None,
			data: vec![1u8, 2, 3, 5, 7, 11],
			code_address: Some("13077bfb00000000000000000000000000000000".parse().unwrap()),
		}
	));

	// siphash result
	let res = LittleEndian::read_u32(&result[..]);
	assert_eq!(res, 317632590);

	assert_eq!(gas_left, U256::from(90_042));
}

// Realloc test
#[test]
fn realloc() {
	let code = load_sample!("realloc.wasm");

	let mut params = ActionParams::default();
	params.gas = U256::from(100_000);
	params.code = Some(Arc::new(code));
	params.data = Some(vec![0u8]);
	let mut ext = FakeExt::new().with_wasm();

	let (gas_left, result) = {
		let mut interpreter = wasm_interpreter(params);
		let result = interpreter.exec(&mut ext).ok().unwrap().expect("Interpreter to execute without any errors");
		match result {
				GasLeft::Known(_) => { panic!("Realloc should return payload"); },
				GasLeft::NeedsReturn { gas_left: gas, data: result, apply_state: _apply } => (gas, result.to_vec()),
		}
	};
	assert_eq!(result, vec![0u8; 2]);
	assert_eq!(gas_left, U256::from(92_848));
}

#[test]
fn alloc() {
	let code = load_sample!("alloc.wasm");

	let mut params = ActionParams::default();
	params.gas = U256::from(10_000_000);
	params.code = Some(Arc::new(code));
	params.data = Some(vec![0u8]);
	let mut ext = FakeExt::new().with_wasm();

	let (gas_left, result) = {
		let mut interpreter = wasm_interpreter(params);
		let result = interpreter.exec(&mut ext).ok().unwrap().expect("Interpreter to execute without any errors");
		match result {
				GasLeft::Known(_) => { panic!("alloc test should return payload"); },
				GasLeft::NeedsReturn { gas_left: gas, data: result, apply_state: _apply } => (gas, result.to_vec()),
		}
	};
	assert_eq!(result, vec![5u8; 1024*400]);
	assert_eq!(gas_left, U256::from(6_893_881));
}

// Tests that contract's ability to read from a storage
// Test prepopulates address into storage, than executes a contract which read that address from storage and write this address into result
#[test]
fn storage_read() {
	let _ = ::env_logger::try_init();

	let code = load_sample!("storage_read.wasm");
	let address: Address = "0f572e5295c57f15886f9b263e2f6d2d6c7b5ec6".parse().unwrap();

	let mut params = ActionParams::default();
	params.gas = U256::from(100_000);
	params.code = Some(Arc::new(code));
	let mut ext = FakeExt::new().with_wasm();
	let hash = H256::from_str("0100000000000000000000000000000000000000000000000000000000000000").unwrap();
	ext.store.insert(hash, address.into());

	let (gas_left, result) = {
		let mut interpreter = wasm_interpreter(params);
		let result = interpreter.exec(&mut ext).ok().unwrap().expect("Interpreter to execute without any errors");
		match result {
				GasLeft::Known(_) => { panic!("storage_read should return payload"); },
				GasLeft::NeedsReturn { gas_left: gas, data: result, apply_state: _apply } => (gas, result.to_vec()),
		}
	};

	assert_eq!(Address::from_slice(&result[12..32]), address);
	assert_eq!(gas_left, U256::from(98_369));
}

// Tests keccak calculation
// keccak.wasm runs wasm-std::keccak function on data param and returns hash
#[test]
fn keccak() {
	let _ = ::env_logger::try_init();
	let code = load_sample!("keccak.wasm");

	let mut params = ActionParams::default();
	params.gas = U256::from(100_000);
	params.code = Some(Arc::new(code));
	params.data = Some(b"something".to_vec());
	let mut ext = FakeExt::new().with_wasm();

	let (gas_left, result) = {
		let mut interpreter = wasm_interpreter(params);
		let result = interpreter.exec(&mut ext).ok().unwrap().expect("Interpreter to execute without any errors");
		match result {
				GasLeft::Known(_) => { panic!("keccak should return payload"); },
				GasLeft::NeedsReturn { gas_left: gas, data: result, apply_state: _apply } => (gas, result.to_vec()),
		}
	};

	assert_eq!(
		H256::from_slice(&result),
		H256::from_str("68371d7e884c168ae2022c82bd837d51837718a7f7dfb7aa3f753074a35e1d87").unwrap(),
	);
	assert_eq!(gas_left, U256::from(85_949));
}

// math_* tests check the ability of wasm contract to perform big integer operations
// - addition
// - multiplication
// - substraction
// - division

// addition
#[test]
fn math_add() {

	let (gas_left, result) = reqrep_test!(
		"math.wasm",
		{
			let mut args = [0u8; 65];
			let arg_a = U256::from_dec_str("999999999999999999999999999999").unwrap();
			let arg_b = U256::from_dec_str("888888888888888888888888888888").unwrap();
			arg_a.to_big_endian(&mut args[1..33]);
			arg_b.to_big_endian(&mut args[33..65]);
			args.to_vec()
		}
	).expect("Interpreter to execute without any errors");

	assert_eq!(
		U256::from_dec_str("1888888888888888888888888888887").unwrap(),
		(&result[..]).into()
	);
	assert_eq!(gas_left, U256::from(92_072));
}

// multiplication
#[test]
fn math_mul() {
	let (gas_left, result) = reqrep_test!(
		"math.wasm",
		{
			let mut args = [1u8; 65];
			let arg_a = U256::from_dec_str("888888888888888888888888888888").unwrap();
			let arg_b = U256::from_dec_str("999999999999999999999999999999").unwrap();
			arg_a.to_big_endian(&mut args[1..33]);
			arg_b.to_big_endian(&mut args[33..65]);
			args.to_vec()
		}
	).expect("Interpreter to execute without any errors");

	assert_eq!(
		U256::from_dec_str("888888888888888888888888888887111111111111111111111111111112").unwrap(),
		(&result[..]).into()
	);
	assert_eq!(gas_left, U256::from(91_400));
}

// subtraction
#[test]
fn math_sub() {
	let (gas_left, result) = reqrep_test!(
		"math.wasm",
		{
			let mut args = [2u8; 65];
			let arg_a = U256::from_dec_str("999999999999999999999999999999").unwrap();
			let arg_b = U256::from_dec_str("888888888888888888888888888888").unwrap();
			arg_a.to_big_endian(&mut args[1..33]);
			arg_b.to_big_endian(&mut args[33..65]);
			args.to_vec()
		}
	).expect("Interpreter to execute without any errors");

	assert_eq!(
		U256::from_dec_str("111111111111111111111111111111").unwrap(),
		(&result[..]).into()
	);
	assert_eq!(gas_left, U256::from(92_072));
}

// subtraction with overflow
#[test]
fn math_sub_with_overflow() {
	let result = reqrep_test!(
		"math.wasm",
		{
			let mut args = [2u8; 65];
			let arg_a = U256::from_dec_str("888888888888888888888888888888").unwrap();
			let arg_b = U256::from_dec_str("999999999999999999999999999999").unwrap();
			arg_a.to_big_endian(&mut args[1..33]);
			arg_b.to_big_endian(&mut args[33..65]);
			args.to_vec()
		}
	);

	match result {
		Err(vm::Error::Wasm(_)) => {},
		_ => panic!("Unexpected result {:?}", result),
	}
}

#[test]
fn math_div() {
	let (gas_left, result) = reqrep_test!(
		"math.wasm",
		{
			let mut args = [3u8; 65];
			let arg_a = U256::from_dec_str("999999999999999999999999999999").unwrap();
			let arg_b = U256::from_dec_str("888888888888888888888888").unwrap();
			arg_a.to_big_endian(&mut args[1..33]);
			arg_b.to_big_endian(&mut args[33..65]);
			args.to_vec()
		}
	).expect("Interpreter to execute without any errors");

	assert_eq!(
		U256::from_dec_str("1125000").unwrap(),
		(&result[..]).into()
	);
	assert_eq!(gas_left, U256::from(85_700));
}

#[test]
fn storage_metering() {
	let _ = ::env_logger::try_init();

	// #1
	let mut ext = FakeExt::new().with_wasm();

	let code = Arc::new(load_sample!("setter.wasm"));
	let address: Address = "0f572e5295c57f15886f9b263e2f6d2d6c7b5ec6".parse().unwrap();

	let mut params = ActionParams::default();
	params.address = address.clone();
	params.gas = U256::from(100_000);
	params.code = Some(code.clone());
	params.data = Some(vec![
		0x9d, 0x9d, 0x9d, 0x9d, 0x9d, 0x9d, 0x9d, 0x9d, 0x9d, 0x9d, 0x9d, 0x9d, 0x9d, 0x9d, 0x9d, 0x9d, 0x9d, 0x9d, 0x9d, 0x9d, 0x9d, 0x9d, 0x9d, 0x9d, 0x9d, 0x9d, 0x9d, 0x9d, 0x9d, 0x9d, 0x9d, 0x9d,
		0x7b, 0x7b, 0x7b, 0x7b, 0x7b, 0x7b, 0x7b, 0x7b, 0x7b, 0x7b, 0x7b, 0x7b, 0x7b, 0x7b, 0x7b, 0x7b, 0x7b, 0x7b, 0x7b, 0x7b, 0x7b, 0x7b, 0x7b, 0x7b, 0x7b, 0x7b, 0x7b, 0x7b, 0x7b, 0x7b, 0x7b, 0x7b,
	]);

	let gas_left = {
		let mut interpreter = wasm_interpreter(params);
		test_finalize(interpreter.exec(&mut ext).ok().unwrap()).unwrap()
	};

	// 0 -> not 0
	assert_eq!(gas_left, U256::from(72_164));

	// #2

	let mut params = ActionParams::default();
	params.address = address.clone();
	params.gas = U256::from(100_000);
	params.code = Some(code.clone());
	params.data = Some(vec![
		0x9d, 0x9d, 0x9d, 0x9d, 0x9d, 0x9d, 0x9d, 0x9d, 0x9d, 0x9d, 0x9d, 0x9d, 0x9d, 0x9d, 0x9d, 0x9d, 0x9d, 0x9d, 0x9d, 0x9d, 0x9d, 0x9d, 0x9d, 0x9d, 0x9d, 0x9d, 0x9d, 0x9d, 0x9d, 0x9d, 0x9d, 0x9d,
		0x6b, 0x7b, 0x7b, 0x7b, 0x7b, 0x7b, 0x7b, 0x7b, 0x7b, 0x7b, 0x7b, 0x7b, 0x7b, 0x7b, 0x7b, 0x7b, 0x7b, 0x7b, 0x7b, 0x7b, 0x7b, 0x7b, 0x7b, 0x7b, 0x7b, 0x7b, 0x7b, 0x7b, 0x7b, 0x7b, 0x7b, 0x7b,
	]);

	let gas_left = {
		let mut interpreter = wasm_interpreter(params);
		test_finalize(interpreter.exec(&mut ext).ok().unwrap()).unwrap()
	};

	// not 0 -> not 0
	assert_eq!(gas_left, U256::from(87_164));
}

// This test checks the ability of wasm contract to invoke
// varios blockchain runtime methods
#[test]
fn externs() {
	let (gas_left, result) = reqrep_test!(
		"externs.wasm",
		Vec::new(),
		vm::EnvInfo {
			number: 0x9999999999u64.into(),
			author: "efefefefefefefefefefefefefefefefefefefef".parse().unwrap(),
			timestamp: 0x8888888888u64.into(),
			difficulty: U256::from_str("0f1f2f3f4f5f6f7f8f9fafbfcfdfefff0d1d2d3d4d5d6d7d8d9dadbdcdddedfd").unwrap(),
			gas_limit: 0x777777777777u64.into(),
			last_hashes: Default::default(),
			gas_used: 0.into(),
		},
		{
			let mut hashes = HashMap::new();
			hashes.insert(
				U256::from(0),
				H256::from_str("9d9d9d9d9d9d9d9d9d9d9d9d9d9d9d9d9d9d9d9d9d9d9d9d9d9d9d9d9d9d9d9d").unwrap(),
			);
			hashes.insert(
				U256::from(1),
				H256::from_str("7b7b7b7b7b7b7b7b7b7b7b7b7b7b7b7b7b7b7b7b7b7b7b7b7b7b7b7b7b7b7b7b").unwrap(),
			);
			hashes
		}
	).expect("Interpreter to execute without any errors");

	assert_eq!(
		&result[0..64].to_vec(),
		&vec![
			0x9d, 0x9d, 0x9d, 0x9d, 0x9d, 0x9d, 0x9d, 0x9d, 0x9d, 0x9d, 0x9d, 0x9d, 0x9d, 0x9d, 0x9d, 0x9d, 0x9d, 0x9d, 0x9d, 0x9d, 0x9d, 0x9d, 0x9d, 0x9d, 0x9d, 0x9d, 0x9d, 0x9d, 0x9d, 0x9d, 0x9d, 0x9d,
			0x7b, 0x7b, 0x7b, 0x7b, 0x7b, 0x7b, 0x7b, 0x7b, 0x7b, 0x7b, 0x7b, 0x7b, 0x7b, 0x7b, 0x7b, 0x7b, 0x7b, 0x7b, 0x7b, 0x7b, 0x7b, 0x7b, 0x7b, 0x7b, 0x7b, 0x7b,0x7b, 0x7b, 0x7b, 0x7b, 0x7b, 0x7b,
		],
		"Block hashes requested and returned do not match"
	);

	assert_eq!(
		&result[64..84].to_vec(),
		&vec![
			0xef, 0xef, 0xef, 0xef, 0xef, 0xef, 0xef, 0xef, 0xef, 0xef, 0xef, 0xef, 0xef, 0xef, 0xef, 0xef, 0xef, 0xef, 0xef, 0xef,
		],
		"Coinbase requested and returned does not match"
	);

	assert_eq!(
		&result[84..92].to_vec(),
		&vec![
			0x88, 0x88, 0x88, 0x88, 0x88, 0x00, 0x00, 0x00
		],
		"Timestamp requested and returned does not match"
	);

	assert_eq!(
		&result[92..100].to_vec(),
		&vec![
			0x99, 0x99, 0x99, 0x99, 0x99, 0x00, 0x00, 0x00
		],
		"Block number requested and returned does not match"
	);

	assert_eq!(
		&result[100..132].to_vec(),
		&vec![
			0x0f, 0x1f, 0x2f, 0x3f, 0x4f, 0x5f, 0x6f, 0x7f,
			0x8f, 0x9f, 0xaf, 0xbf, 0xcf, 0xdf, 0xef, 0xff,
			0x0d, 0x1d, 0x2d, 0x3d, 0x4d, 0x5d, 0x6d, 0x7d,
			0x8d, 0x9d, 0xad, 0xbd, 0xcd, 0xdd, 0xed, 0xfd,
		],
		"Difficulty requested and returned does not match"
	);

	assert_eq!(
		&result[132..164].to_vec(),
		&vec![
			0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
			0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
			0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
			0x00, 0x00, 0x77, 0x77, 0x77, 0x77, 0x77, 0x77,
		],
		"Gas limit requested and returned does not match"
	);

	assert_eq!(gas_left, U256::from(90_428));
}

// This test checks the ability of wasm contract to invoke gasleft
#[test]
fn gasleft() {
	let _ = ::env_logger::try_init();

	let mut params = ActionParams::default();
	params.gas = U256::from(100_000);
	params.code = Some(Arc::new(load_sample!("gasleft.wasm")));

	let mut ext = FakeExt::new().with_wasm();
	ext.schedule.wasm.as_mut().unwrap().have_gasleft = true;

	let interpreter = wasm_interpreter(params);
	let result = interpreter.exec(&mut ext).ok().unwrap().expect("Interpreter to execute without any errors");
	match result {
		GasLeft::Known(_) => {},
		GasLeft::NeedsReturn { gas_left, data, .. } => {
			let gas = LittleEndian::read_u64(data.as_ref());
			assert_eq!(gas, 93_423);
			assert_eq!(gas_left, U256::from(93_349));
		},
	}
}

// This test should fail because
// ext.schedule.wasm.as_mut().unwrap().have_gasleft = false;
#[test]
fn gasleft_fail() {
	let _ = ::env_logger::try_init();

	let mut params = ActionParams::default();
	params.gas = U256::from(100_000);
	params.code = Some(Arc::new(load_sample!("gasleft.wasm")));
	let mut ext = FakeExt::new().with_wasm();
	let interpreter = wasm_interpreter(params);
	match interpreter.exec(&mut ext).ok().unwrap() {
		Err(_) => {},
		Ok(_) => panic!("interpreter.exec should return Err if ext.schedule.wasm.have_gasleft = false")
	}
}

#[test]
fn embedded_keccak() {
	let _ = ::env_logger::try_init();
	let mut code = load_sample!("keccak.wasm");
	code.extend_from_slice(b"something");

	let mut params = ActionParams::default();
	params.gas = U256::from(100_000);
	params.code = Some(Arc::new(code));
	params.params_type = vm::ParamsType::Embedded;

	let mut ext = FakeExt::new().with_wasm();

	let (gas_left, result) = {
		let mut interpreter = wasm_interpreter(params);
		let result = interpreter.exec(&mut ext).ok().unwrap().expect("Interpreter to execute without any errors");
		match result {
			GasLeft::Known(_) => { panic!("keccak should return payload"); },
			GasLeft::NeedsReturn { gas_left: gas, data: result, apply_state: _apply } => (gas, result.to_vec()),
		}
	};

	assert_eq!(
		H256::from_slice(&result),
		H256::from_str("68371d7e884c168ae2022c82bd837d51837718a7f7dfb7aa3f753074a35e1d87").unwrap(),
	);
	assert_eq!(gas_left, U256::from(85_949));
}

/// This test checks the correctness of log extern
/// Target test puts one event with two topic [keccak(input), reverse(keccak(input))]
/// and reversed input as a data
#[test]
fn events() {
	let _ = ::env_logger::try_init();
	let code = load_sample!("events.wasm");

	let mut params = ActionParams::default();
	params.gas = U256::from(100_000);
	params.code = Some(Arc::new(code));
	params.data = Some(b"something".to_vec());

	let mut ext = FakeExt::new().with_wasm();

	let (gas_left, result) = {
		let mut interpreter = wasm_interpreter(params);
		let result = interpreter.exec(&mut ext).ok().unwrap().expect("Interpreter to execute without any errors");
		match result {
			GasLeft::Known(_) => { panic!("events should return payload"); },
			GasLeft::NeedsReturn { gas_left: gas, data: result, apply_state: _apply } => (gas, result.to_vec()),
		}
	};

	assert_eq!(ext.logs.len(), 1);
	let log_entry = &ext.logs[0];
	assert_eq!(log_entry.topics.len(), 2);
	assert_eq!(
		log_entry.topics[0],
		H256::from_str("68371d7e884c168ae2022c82bd837d51837718a7f7dfb7aa3f753074a35e1d87").unwrap(),
	);
	assert_eq!(
		log_entry.topics[1],
		H256::from_str("871d5ea37430753faab7dff7a7187783517d83bd822c02e28a164c887e1d3768").unwrap(),
	);
	assert_eq!(log_entry.data, b"gnihtemos");

	assert_eq!(&result, b"gnihtemos");
	assert_eq!(gas_left, U256::from(83_161));
}

#[test]
fn recursive() {
	let _ = ::env_logger::try_init();
	let code = load_sample!("recursive.wasm");

	let mut params = ActionParams::default();
	params.gas = U256::from(100_000_000);
	params.code = Some(Arc::new(code));
	params.data = Some({
		// `recursive` expects only one 32-bit word in LE that
		// represents an iteration count.
		//
		// We pick a relative big number to definitely hit stack overflow.
		use byteorder::WriteBytesExt;
		let mut data = vec![];
		data.write_u32::<LittleEndian>(100000).unwrap();
		data
	});

	let mut ext = FakeExt::new().with_wasm();

	let interpreter = wasm_interpreter(params);
	let result = interpreter.exec(&mut ext).ok().unwrap();

	// We expect that stack overflow will occur and it should be generated by
	// deterministic stack metering. Exceeding deterministic stack height limit
	// always ends with a trap generated by `unreachable` instruction.
	match result {
		Err(trap) => {
			let err_description = trap.to_string();
			assert!(
				err_description.contains("Unreachable"),
				"err_description: {} should contain 'Unreachable'",
				err_description
			);
		},
		_ => panic!("this test should trap"),
	}
}
