use fixture::{Fixture, Assert, CallLocator, Source};
use wasm::WasmInterpreter;
use vm::{self, Vm, GasLeft, ActionParams, ActionValue, ParamsType};
use vm::tests::FakeExt;
use std::io::{self, Read};
use std::{fs, path, fmt};
use std::sync::Arc;
use ethereum_types::{U256, H256, H160};
use rustc_hex::ToHex;

fn load_code<P: AsRef<path::Path>>(p: P) -> io::Result<Vec<u8>> {
	let mut result = Vec::new();
	let mut f = fs::File::open(p)?;
	f.read_to_end(&mut result)?;
	Ok(result)
}

fn wasm_interpreter() -> WasmInterpreter {
	WasmInterpreter
}

#[derive(Debug)]
pub enum SpecNonconformity {
	Address,
}

#[derive(Debug)]
pub enum Fail {
	Return { expected: Vec<u8>, actual: Vec<u8> },
	UsedGas { expected: u64, actual: u64 },
	Runtime(String),
	Load(io::Error),
	NoCall(CallLocator),
	StorageMismatch { key: H256, expected: H256, actual: Option<H256> },
	Nonconformity(SpecNonconformity)
}

impl Fail {
	fn runtime(err: vm::Error) -> Vec<Fail> {
		vec![Fail::Runtime(format!("{}", err))]
	}

	fn load(err: io::Error) -> Vec<Fail> {
		vec![Fail::Load(err)]
	}

	fn nononformity(kind: SpecNonconformity) -> Vec<Fail> {
		vec![Fail::Nonconformity(kind)]
	}
}

impl fmt::Display for Fail {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		use self::Fail::*;
		match *self {
			Return { ref expected, ref actual } =>
				write!(
					f,
					"Expected to return result: 0x{} ({} bytes), but got 0x{} ({} bytes)",
					expected.to_hex(),
					expected.len(),
					actual.to_hex(),
					actual.len()
				),

			UsedGas { expected, actual } =>
				write!(f, "Expected to use gas: {}, but got actual gas used: {}", expected, actual),

			Runtime(ref s) =>
				write!(f, "WASM Runtime error: {}", s),

			Load(ref e) =>
				write!(f, "Load i/o error: {}", e),

			NoCall(ref call) =>
				write!(f, "Call not found: {:?}", call),

			StorageMismatch { ref key, ref expected, actual: Some(ref actual)} =>
				write!(
					f,
					"Storage key {} value mismatch, expected {}, got: {}",
					key.to_vec().to_hex(),
					expected.to_vec().to_hex(),
					actual.to_vec().to_hex(),
				),

			StorageMismatch { ref key, ref expected, actual: None} =>
				write!(
					f,
					"No expected storage value for key {} found, expected {}",
					key.to_vec().to_hex(),
					expected.to_vec().to_hex(),
				),

			Nonconformity(SpecNonconformity::Address) =>
				write!(f, "Cannot use address when constructor is specified!"),
		}
	}
}

pub fn construct(
	ext: &mut vm::Ext,
	source: Vec<u8>,
	arguments: Vec<u8>,
	sender: H160,
	at: H160,
) -> Result<Vec<u8>, vm::Error> {

	let mut params = ActionParams::default();
	params.sender = sender;
	params.address = at;
	params.gas = U256::from(100_000_000);
	params.data = Some(arguments);
	params.code = Some(Arc::new(source));
	params.params_type = ParamsType::Separate;

	Ok(
		match wasm_interpreter().exec(params, ext)? {
			GasLeft::Known(_) => Vec::new(),
			GasLeft::NeedsReturn { data, .. } => data.to_vec(),
		}
	)
}

pub fn run_fixture(fixture: &Fixture) -> Vec<Fail> {
	let mut params = ActionParams::default();

	let source = match load_code(fixture.source.as_ref()) {
		Ok(code) => code,
		Err(e) => { return Fail::load(e); },
	};

	let mut ext = FakeExt::new().with_wasm();
	params.code = Some(Arc::new(
		if let Source::Constructor { ref arguments, ref sender, ref at, .. } = fixture.source {
			match construct(&mut ext, source, arguments.clone().into(), sender.clone().into(), at.clone().into()) {
				Ok(code) => code,
				Err(e) => { return Fail::runtime(e); }
			}
		} else {
			source
		}
	));

	if let Some(ref sender) = fixture.sender {
		params.sender = sender.clone().into();
	}

	if let Some(ref address) = fixture.address {
		if let Source::Constructor { .. } = fixture.source {
			return Fail::nononformity(SpecNonconformity::Address);
		}

		params.address = address.clone().into();
	} else if let Source::Constructor { ref at, .. } = fixture.source {
		params.address = at.clone().into();
	}

	if let Some(gas_limit) = fixture.gas_limit {
		params.gas = U256::from(gas_limit);
	}

	if let Some(ref data) = fixture.payload {
		params.data = Some(data.clone().into())
	}

	if let Some(value) = fixture.value {
		params.value = ActionValue::Transfer(value.clone().into())
	}

	if let Some(ref storage) = fixture.storage {
		for storage_entry in storage.iter() {
			let key: U256 = storage_entry.key.into();
			let val: U256 = storage_entry.value.into();
			ext.store.insert(key.into(), val.into());
		}
	}

	let mut interpreter = wasm_interpreter();

	let interpreter_return = match interpreter.exec(params, &mut ext) {
		Ok(ret) => ret,
		Err(e) => { return Fail::runtime(e); }
	};
	let (gas_left, result) = match interpreter_return {
		GasLeft::Known(gas) => { (gas, Vec::new()) },
		GasLeft::NeedsReturn { gas_left: gas, data: result, apply_state: _apply } => (gas, result.to_vec()),
	};

	let mut fails = Vec::new();

	for assert in fixture.asserts.iter() {
		match *assert {
			Assert::Return(ref data) => {
				if &data[..] != &result[..] {
					fails.push(Fail::Return { expected: (&data[..]).to_vec(), actual: (&result[..]).to_vec() })
				}
			},
			Assert::UsedGas(gas) => {
				let used_gas = fixture.gas_limit.unwrap_or(0) - gas_left.low_u64();
				if gas != used_gas {
					fails.push(Fail::UsedGas { expected: gas, actual: used_gas });
				}
			},
			Assert::HasCall(ref locator) => {
				let mut found = false;

				for fake_call in ext.calls.iter() {
					let mut match_ = true;
					if let Some(ref data) = locator.data {
						if data.as_ref() != &fake_call.data[..] { match_ = false; }
					}

					if let Some(ref code_addr) = locator.code_address {
						if fake_call.code_address.unwrap_or(H160::zero()) != code_addr.clone().into() { match_ = false }
					}

					if let Some(ref sender) = locator.sender {
						if fake_call.sender_address.unwrap_or(H160::zero()) != sender.clone().into() { match_ = false }
					}

					if let Some(ref receiver) = locator.receiver {
						if fake_call.receive_address.unwrap_or(H160::zero()) != receiver.clone().into() { match_ = false }
					}

					if match_ {
						found = true;
						break;
					}
				}

				if !found {
					fails.push(Fail::NoCall(locator.clone()))
				}
			},
			Assert::HasStorage(ref storage_entry) => {
				let expected_storage_key: H256 = storage_entry.key.clone().into();
				let expected_storage_value: H256 = storage_entry.value.clone().into();
				let val = ext.store.get(&expected_storage_key);

				if let Some(val) = val {
					if val != &expected_storage_value {
						fails.push(Fail::StorageMismatch {
							key: expected_storage_key,
							expected: expected_storage_value,
							actual: Some(val.clone())
						})
					}
				} else {
					fails.push(Fail::StorageMismatch {
						key: expected_storage_key,
						expected: expected_storage_value,
						actual: None,
					})
				}

			},
		}
	}
	fails
}
