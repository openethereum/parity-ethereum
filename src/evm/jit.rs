//! Just in time compiler execution environment.
use common::*;
use evmjit;
use evm;

/// Ethcore representation of evmjit runtime data.
struct RuntimeData {
	gas: U256,
	gas_price: U256,
	call_data: Vec<u8>,
	address: Address,
	caller: Address,
	origin: Address,
	call_value: U256,
	author: Address,
	difficulty: U256,
	gas_limit: U256,
	number: u64,
	timestamp: u64,
	code: Vec<u8>
}

impl RuntimeData {
	fn new() -> RuntimeData {
		RuntimeData {
			gas: U256::zero(),
			gas_price: U256::zero(),
			call_data: vec![],
			address: Address::new(),
			caller: Address::new(),
			origin: Address::new(),
			call_value: U256::zero(),
			author: Address::new(),
			difficulty: U256::zero(),
			gas_limit: U256::zero(),
			number: 0,
			timestamp: 0,
			code: vec![]
		}
	}
}

/// Should be used to convert jit types to ethcore
trait FromJit<T>: Sized {
	fn from_jit(input: T) -> Self;
}

/// Should be used to covert ethcore types to jit
trait IntoJit<T> {
	fn into_jit(self) -> T;
}

impl<'a> FromJit<&'a evmjit::I256> for U256 {
	fn from_jit(input: &'a evmjit::I256) -> Self {
		unsafe {
			let mut res: U256 = mem::uninitialized();
			ptr::copy(input.words.as_ptr(), res.0.as_mut_ptr(), 4);
			res
		}
	}
}

impl<'a> FromJit<&'a evmjit::I256> for H256 {
	fn from_jit(input: &'a evmjit::I256) -> Self {
		let u = U256::from_jit(input);
		H256::from(&u)
	}
}

impl<'a> FromJit<&'a evmjit::I256> for Address {
	fn from_jit(input: &'a evmjit::I256) -> Self {
		Address::from(H256::from_jit(input))
	}
}

impl<'a> FromJit<&'a evmjit::H256> for H256 {
	fn from_jit(input: &'a evmjit::H256) -> Self {
		H256::from_jit(&evmjit::I256::from(input.clone()))
	}
}

impl<'a> FromJit<&'a evmjit::H256> for Address {
	fn from_jit(input: &'a evmjit::H256) -> Self {
		Address::from(H256::from_jit(input))
	}
}

impl IntoJit<evmjit::I256> for U256 {
	fn into_jit(self) -> evmjit::I256 {
		unsafe {
			let mut res: evmjit::I256 = mem::uninitialized();
			ptr::copy(self.0.as_ptr(), res.words.as_mut_ptr(), 4);
			res
		}
	}
}

impl IntoJit<evmjit::I256> for H256 {
	fn into_jit(self) -> evmjit::I256 {
		let mut ret = [0; 4];
		for i in 0..self.bytes().len() {
			let rev = self.bytes().len() - 1 - i;
			let pos = rev / 8;
			ret[pos] += (self.bytes()[i] as u64) << (rev % 8) * 8;
		}
		evmjit::I256 { words: ret }
	}
}

impl IntoJit<evmjit::H256> for H256 {
	fn into_jit(self) -> evmjit::H256 {
		let i: evmjit::I256 = self.into_jit();
		From::from(i)
	}
}

impl IntoJit<evmjit::I256> for Address {
	fn into_jit(self) -> evmjit::I256 {
		H256::from(self).into_jit()
	}
}

impl IntoJit<evmjit::H256> for Address {
	fn into_jit(self) -> evmjit::H256 {
		H256::from(self).into_jit()
	}
}

impl IntoJit<evmjit::RuntimeDataHandle> for RuntimeData {
	fn into_jit(self) -> evmjit::RuntimeDataHandle {
		let mut data = evmjit::RuntimeDataHandle::new();
		assert!(self.gas <= U256::from(u64::max_value()), "evmjit gas must be lower than 2 ^ 64");
		assert!(self.gas_price <= U256::from(u64::max_value()), "evmjit gas_price must be lower than 2 ^ 64");
		data.gas = self.gas.low_u64() as i64;
		data.gas_price = self.gas_price.low_u64() as i64;
		data.call_data = self.call_data.as_ptr();
		data.call_data_size = self.call_data.len() as u64;
		mem::forget(self.call_data);
		data.address = self.address.into_jit();
		data.caller = self.caller.into_jit();
		data.origin = self.origin.into_jit();
		data.call_value = self.call_value.into_jit();
		data.author = self.author.into_jit();
		data.difficulty = self.difficulty.into_jit();
		data.gas_limit = self.gas_limit.into_jit();
		data.number = self.number;
		data.timestamp = self.timestamp as i64;
		data.code = self.code.as_ptr();
		data.code_size = self.code.len() as u64;
		data.code_hash = self.code.sha3().into_jit();
		mem::forget(self.code);
		data
	}
}

/// Externalities adapter. Maps callbacks from evmjit to externalities trait.
/// 
/// Evmjit doesn't have to know about children execution failures. 
/// This adapter 'catches' them and moves upstream.
struct ExtAdapter<'a> {
	ext: &'a mut evm::Ext,
	err: &'a mut Option<evm::Error>
}

impl<'a> ExtAdapter<'a> {
	fn new(ext: &'a mut evm::Ext, err: &'a mut Option<evm::Error>) -> Self {
		ExtAdapter {
			ext: ext,
			err: err
		}
	}
}

impl<'a> evmjit::Ext for ExtAdapter<'a> {
	fn sload(&self, index: *const evmjit::I256, out_value: *mut evmjit::I256) {
		unsafe {
			let i = H256::from_jit(&*index);
			let o = self.ext.sload(&i);
			*out_value = o.into_jit();
		}
	}

	fn sstore(&mut self, index: *const evmjit::I256, value: *const evmjit::I256) {
		unsafe {
			self.ext.sstore(H256::from_jit(&*index), H256::from_jit(&*value));
		}
	}

	fn balance(&self, address: *const evmjit::H256, out_value: *mut evmjit::I256) {
		unsafe {
			let a = Address::from_jit(&*address);
			let o = self.ext.balance(&a);
			*out_value = o.into_jit();
		}
	}

	fn blockhash(&self, number: *const evmjit::I256, out_hash: *mut evmjit::H256) {
		unsafe {
			let n = U256::from_jit(&*number);
			let o = self.ext.blockhash(&n);
			*out_hash = o.into_jit();
		}
	}

	fn create(&mut self,
			  io_gas: *mut u64,
			  endowment: *const evmjit::I256,
			  init_beg: *const u8,
			  init_size: u64,
			  address: *mut evmjit::H256) {
		unsafe {
			match self.ext.create(*io_gas, &U256::from_jit(&*endowment), slice::from_raw_parts(init_beg, init_size as usize)) {
				Ok((gas_left, opt)) => {
					*io_gas = gas_left;
					if let Some(addr) = opt {
						*address = addr.into_jit();
					}
				},
				Err(err @ evm::Error::OutOfGas) => {
					*self.err = Some(err);
					// hack to propagate `OutOfGas` to evmjit and stop
					// the execution immediately.
					// Works, cause evmjit uses i64, not u64
					*io_gas = -1i64 as u64
				},
				Err(err) => *self.err = Some(err)
			}
		}
	}

	fn call(&mut self,
				io_gas: *mut u64,
				call_gas: u64,
				receive_address: *const evmjit::H256,
				value: *const evmjit::I256,
				in_beg: *const u8,
				in_size: u64,
				out_beg: *mut u8,
				out_size: u64,
				code_address: *const evmjit::H256) -> bool {
		unsafe {
			let res = self.ext.call(*io_gas, 
									call_gas, 
									&Address::from_jit(&*receive_address),
									&U256::from_jit(&*value),
									slice::from_raw_parts(in_beg, in_size as usize),
									&Address::from_jit(&*code_address),
									slice::from_raw_parts_mut(out_beg, out_size as usize));

			match res {
				Ok(gas_left) => {
					*io_gas = gas_left;
					true
				},
				Err(err @ evm::Error::OutOfGas) => {
					*self.err = Some(err);
					// hack to propagate `OutOfGas` to evmjit and stop
					// the execution immediately.
					// Works, cause evmjit uses i64, not u64
					*io_gas = -1i64 as u64;
					false
				},
				Err(err) => {
					*self.err = Some(err);
					false
				}
			}
		}
	}

	fn log(&mut self,
		   beg: *const u8,
		   size: u64,
		   topic1: *const evmjit::H256,
		   topic2: *const evmjit::H256,
		   topic3: *const evmjit::H256,
		   topic4: *const evmjit::H256) {

		unsafe {
			let mut topics = vec![];
			if !topic1.is_null() {
				topics.push(H256::from_jit(&*topic1));
			}

			if !topic2.is_null() {
				topics.push(H256::from_jit(&*topic2));
			}

			if !topic3.is_null() {
				topics.push(H256::from_jit(&*topic3));
			}

			if !topic4.is_null() {
				topics.push(H256::from_jit(&*topic4));
			}
		
			let bytes_ref: &[u8] = slice::from_raw_parts(beg, size as usize);
			self.ext.log(topics, bytes_ref.to_vec());
		}
	}

	fn extcode(&self, address: *const evmjit::H256, size: *mut u64) -> *const u8 {
		unsafe {
			let code = self.ext.extcode(&Address::from_jit(&*address));
			*size = code.len() as u64;
			let ptr = code.as_ptr();
			mem::forget(code);
			ptr
		}
	}
}

pub struct JitEvm;

impl evm::Evm for JitEvm {
	fn exec(&self, params: &ActionParams, ext: &mut evm::Ext) -> evm::Result {
		let mut optional_err = None;
		// Dirty hack. This is unsafe, but we interact with ffi, so it's justified.
		let ext_adapter: ExtAdapter<'static> = unsafe { ::std::mem::transmute(ExtAdapter::new(ext, &mut optional_err)) };
		let mut ext_handle = evmjit::ExtHandle::new(ext_adapter);
		let mut data = RuntimeData::new();
		data.gas = params.gas;
		data.gas_price = params.gas_price;
		data.call_data = params.data.clone();
		data.address = params.address.clone();
		data.caller = params.sender.clone();
		data.origin = params.origin.clone();
		data.call_value = params.value;
		data.code = params.code.clone();

		// TODO:
		data.author = Address::new();
		data.difficulty = U256::zero();
		data.gas_limit = U256::zero();
		data.number = 0;
		data.timestamp = 0;
		
		let mut context = unsafe { evmjit::ContextHandle::new(data.into_jit(), &mut ext_handle) };
		let res = context.exec();
		
		// check in adapter if execution of children contracts failed.
		if let Some(err) = optional_err {
			return Err(err);
		}
		
		match res {
			evmjit::ReturnCode::Stop => Ok(U256::from(context.gas_left())),
			evmjit::ReturnCode::Return => ext.ret(context.gas_left(), context.output_data()).map(|gas_left| U256::from(gas_left)),
			evmjit::ReturnCode::Suicide => { 
				// what if there is a suicide and we run out of gas just after?
				ext.suicide();
				Ok(U256::from(context.gas_left()))
			},
			evmjit::ReturnCode::OutOfGas => Err(evm::Error::OutOfGas),
			_err => Err(evm::Error::Internal)
		}
	}
}

