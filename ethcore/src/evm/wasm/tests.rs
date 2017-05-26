use std::path::PathBuf;
use std::fs::File;
use std::io::Read;
use std::sync::Arc;

use ethcore_logger::init_log;
use super::super::tests::FakeExt;
use super::WasmInterpreter;
use evm::{self, Evm, GasLeft};
use action_params::{ActionParams, ActionValue};
use util::{U256, H256, Address};

fn load_sample(name: &str) -> Vec<u8> {
	let mut path = PathBuf::from("./res/wasm-tests/compiled");
	path.push(name);
	let mut file = File::open(path).expect(&format!("File {} for test to exist", name));
	let mut data = vec![];
	file.read_to_end(&mut data).expect(&format!("Test {} to load ok", name));
	data
}

fn test_finalize(res: Result<GasLeft, evm::Error>) -> Result<U256, evm::Error> {
	match res {
		Ok(GasLeft::Known(gas)) => Ok(gas),
		Ok(GasLeft::NeedsReturn{..}) => unimplemented!(), // since ret is unimplemented.
		Err(e) => Err(e),
	}
}

fn wasm_interpreter() -> WasmInterpreter {
	WasmInterpreter::new().expect("wasm interpreter to create without errors")
}

#[test]
fn empty() {
	init_log();

	let code = load_sample("empty.wasm");
	let address: Address = "0f572e5295c57f15886f9b263e2f6d2d6c7b5ec6".parse().unwrap();

	let mut params = ActionParams::default();
	params.address = address.clone();
	params.gas = U256::from(100_000);
	params.code = Some(Arc::new(code));
	let mut ext = FakeExt::new();

	let gas_left = {
		let mut interpreter = wasm_interpreter();
		test_finalize(interpreter.exec(params, &mut ext)).unwrap()
	};

	assert_eq!(gas_left, U256::from(99_996));
}

#[test]
fn logger() {
	let code = load_sample("logger.wasm");
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
	let mut ext = FakeExt::new();

	let gas_left = {
		let mut interpreter = wasm_interpreter();
		test_finalize(interpreter.exec(params, &mut ext)).unwrap()
	};

	assert_eq!(gas_left, U256::from(99846));
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
		U256::from(ext.store.get(&"0400000000000000000000000000000000000000000000000000000000000000".parse().unwrap()).expect("storage key to exist")),
		U256::from(1_000_000_000),
		"Logger sets 0x04 key to the trasferred value"
	);
}

#[test]
fn identity() {
	init_log();

	let code = load_sample("identity.wasm");
	let sender: Address = "01030507090b0d0f11131517191b1d1f21232527".parse().unwrap();

	let mut params = ActionParams::default();
	params.sender = sender.clone();
	params.gas = U256::from(100_000);
	params.code = Some(Arc::new(code));
	let mut ext = FakeExt::new();

	let (gas_left, result) = {
		let mut interpreter = wasm_interpreter();
		let result = interpreter.exec(params, &mut ext).expect("Interpreter to execute without any errors");
		match result {
			GasLeft::Known(_) => { panic!("Identity contract should return payload"); },
			GasLeft::NeedsReturn { gas_left: gas, data: result, apply_state: _apply } => (gas, result.to_vec()),
		}
	};

	assert_eq!(gas_left, U256::from(99_753));

	assert_eq!(
		Address::from_slice(&result),
		sender,
		"Idenity test contract does not return the sender passed"
	);
}