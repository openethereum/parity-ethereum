use std::path::PathBuf;
use std::fs::File;
use std::io::Read;
use std::sync::Arc;

use ethcore_logger::init_log;
use super::super::tests::FakeExt;
use super::WasmInterpreter;
use evm::{self, Evm, GasLeft};
use action_params::ActionParams;
use util::{U256, Address};

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