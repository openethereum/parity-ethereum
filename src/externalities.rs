//! Transaction Execution environment.
use common::*;
use state::*;
use engine::*;
use executive::*;
use evm::{self, Schedule, Ext};
use substate::*;

/// Policy for handling output data on `RETURN` opcode.
pub enum OutputPolicy<'a> {
	/// Return reference to fixed sized output.
	/// Used for message calls.
	Return(BytesRef<'a>),
	/// Init new contract as soon as `RETURN` is called.
	InitContract
}

/// Implementation of evm Externalities.
pub struct Externalities<'a> {
	
	#[cfg(test)]
	pub state: &'a mut State,
	#[cfg(not(test))]
	state: &'a mut State,

	info: &'a EnvInfo,
	engine: &'a Engine,
	depth: usize,
	
	#[cfg(test)]
	pub params: &'a ActionParams,
	#[cfg(not(test))]
	params: &'a ActionParams,
	
	substate: &'a mut Substate,
	schedule: Schedule,
	output: OutputPolicy<'a>
}

impl<'a> Externalities<'a> {
	/// Basic `Externalities` constructor.
	pub fn new(state: &'a mut State, 
			   info: &'a EnvInfo, 
			   engine: &'a Engine, 
			   depth: usize,
			   params: &'a ActionParams, 
			   substate: &'a mut Substate, 
			   output: OutputPolicy<'a>) -> Self {
		Externalities {
			state: state,
			info: info,
			engine: engine,
			depth: depth,
			params: params,
			substate: substate,
			schedule: engine.schedule(info),
			output: output
		}
	}
}

impl<'a> Ext for Externalities<'a> {
	fn storage_at(&self, key: &H256) -> H256 {
		trace!("ext: storage_at({}, {}) == {}\n", self.params.address, key, U256::from(self.state.storage_at(&self.params.address, key).as_slice()));
		self.state.storage_at(&self.params.address, key)
	}

	fn set_storage_at(&mut self, key: H256, value: H256) {
		let old = self.state.storage_at(&self.params.address, &key);
		// if SSTORE nonzero -> zero, increment refund count
		if value.is_zero() && !old.is_zero() {
			trace!("ext: additional refund. {} -> {}\n", self.substate.refunds_count, self.substate.refunds_count + x!(1));
			self.substate.refunds_count = self.substate.refunds_count + U256::one();
		}
		trace!("ext: set_storage_at({}, {}): {} -> {}\n", self.params.address, key, U256::from(old.as_slice()), U256::from(value.as_slice()));
		self.state.set_storage(&self.params.address, key, value)
	}

	fn balance(&self, address: &Address) -> U256 {
		self.state.balance(address)
	}

	fn blockhash(&self, number: &U256) -> H256 {
		match *number < U256::from(self.info.number) && number.low_u64() >= cmp::max(256, self.info.number) - 256 {
			true => {
				let index = self.info.number - number.low_u64() - 1;
				let r = self.info.last_hashes[index as usize].clone();
				trace!("ext: blockhash({}) -> {} self.info.number={}\n", number, r, self.info.number);
				r
			},
			false => {
				trace!("ext: blockhash({}) -> null self.info.number={}\n", number, self.info.number);
				H256::from(&U256::zero())
			},
		}
	}

	fn create(&mut self, gas: &U256, value: &U256, code: &[u8]) -> (U256, Option<Address>) {
		// if balance is insufficient or we are to deep, return
		if self.state.balance(&self.params.address) < *value || self.depth >= self.schedule.max_depth {
			return (*gas, None);
		}

		// create new contract address
		let address = contract_address(&self.params.address, &self.state.nonce(&self.params.address));

		// prepare the params
		let params = ActionParams {
			code_address: address.clone(),
			address: address.clone(),
			sender: self.params.address.clone(),
			origin: self.params.origin.clone(),
			gas: *gas,
			gas_price: self.params.gas_price.clone(),
			value: value.clone(),
			code: Some(code.to_vec()),
			data: None,
		};

		self.state.inc_nonce(&self.params.address);
		let mut ex = Executive::from_parent(self.state, self.info, self.engine, self.depth);
		match ex.create(&params, self.substate) {
			Ok(gas_left) => (gas_left, Some(address)),
			_ => (U256::zero(), None)
		}
	}

	fn call(&mut self, 
			gas: &U256, 
			call_gas: &U256, 
			receive_address: &Address, 
			value: &U256, 
			data: &[u8], 
			code_address: &Address, 
			output: &mut [u8]) -> Result<(U256, bool), evm::Error> {

		let mut gas_cost = *call_gas;
		let mut call_gas = *call_gas;

		let is_call = receive_address == code_address;
		if is_call && !self.state.exists(&code_address) {
			gas_cost = gas_cost + U256::from(self.schedule.call_new_account_gas);
		}

		if *value > U256::zero() {
			assert!(self.schedule.call_value_transfer_gas > self.schedule.call_stipend, "overflow possible");
			gas_cost = gas_cost + U256::from(self.schedule.call_value_transfer_gas);
			call_gas = call_gas + U256::from(self.schedule.call_stipend);
		}

		debug!("Externalities::call(gas={}, call_gas={}, recv={}, value={}, data={}, code={})\n", gas, call_gas, receive_address, value, data.pretty(), code_address);

		if gas_cost > *gas {
			debug!("Externalities::call: OutOfGas gas_cost={}, gas={}", gas_cost, gas);
			return Err(evm::Error::OutOfGas);
		}

		let gas = *gas - gas_cost;

		// if balance is insufficient or we are too deep, return
		if self.state.balance(&self.params.address) < *value || self.depth >= self.schedule.max_depth {
			debug!("Externalities::call: OutOfCash bal({})={}, value={}", self.params.address, self.state.balance(&self.params.address), value);
			return Ok((gas + call_gas, false));
		}

		let params = ActionParams {
			code_address: code_address.clone(),
			address: receive_address.clone(), 
			sender: self.params.address.clone(),
			origin: self.params.origin.clone(),
			gas: call_gas,
			gas_price: self.params.gas_price.clone(),
			value: value.clone(),
			code: self.state.code(code_address),
			data: Some(data.to_vec()),
		};


		trace!("Externalities::call: BEFORE: bal({})={}, bal({})={}\n", params.sender, self.state.balance(&params.sender), params.address, self.state.balance(&params.address));
		trace!("Externalities::call: CALLING: params={:?}\n", params);
		let r = Executive::from_parent(self.state, self.info, self.engine, self.depth).call(&params, self.substate, BytesRef::Fixed(output));
		trace!("Externalities::call: AFTER: bal({})={}, bal({})={}\n", params.sender, self.state.balance(&params.sender), params.address, self.state.balance(&params.address));

		match r {
			Ok(gas_left) => Ok((gas + gas_left, true)),
			_ => Ok((gas, false))
		}
	}

	fn extcode(&self, address: &Address) -> Vec<u8> {
		self.state.code(address).unwrap_or(vec![])
	}

	fn ret(&mut self, gas: &U256, data: &[u8]) -> Result<U256, evm::Error> {
		match &mut self.output {
			&mut OutputPolicy::Return(BytesRef::Fixed(ref mut slice)) => unsafe {
				let len = cmp::min(slice.len(), data.len());
				ptr::copy(data.as_ptr(), slice.as_mut_ptr(), len);
				Ok(*gas)
			},
			&mut OutputPolicy::Return(BytesRef::Flexible(ref mut vec)) => unsafe {
				vec.clear();
				vec.reserve(data.len());
				ptr::copy(data.as_ptr(), vec.as_mut_ptr(), data.len());
				vec.set_len(data.len());
				Ok(*gas)
			},
			&mut OutputPolicy::InitContract => {
				let return_cost = U256::from(data.len()) * U256::from(self.schedule.create_data_gas);
				if return_cost > *gas {
					return match self.schedule.exceptional_failed_code_deposit {
						true => Err(evm::Error::OutOfGas),
						false => Ok(*gas)
					}
				}
				let mut code = vec![];
				code.reserve(data.len());
				unsafe {
					ptr::copy(data.as_ptr(), code.as_mut_ptr(), data.len());
					code.set_len(data.len());
				}
				let address = &self.params.address;
				self.state.init_code(address, code);
				self.substate.contracts_created.push(address.clone());
				Ok(*gas - return_cost)
			}
		}
	}

	fn log(&mut self, topics: Vec<H256>, data: Bytes) {
		let address = self.params.address.clone();
		self.substate.logs.push(LogEntry::new(address, topics, data));
	}

	fn suicide(&mut self, refund_address: &Address) {
		let address = self.params.address.clone();
		let balance = self.balance(&address);
		self.state.transfer_balance(&address, refund_address, &balance);
		self.substate.suicides.insert(address);
	}

	fn schedule(&self) -> &Schedule {
		&self.schedule
	}

	fn env_info(&self) -> &EnvInfo {
		&self.info
	}
}
