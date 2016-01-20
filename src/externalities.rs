//! Transaction Execution environment.
use common::*;
use state::*;
use engine::*;
use executive::*;
use evm::{self, Schedule, Ext, ContractCreateResult, MessageCallResult};
use substate::*;

/// Policy for handling output data on `RETURN` opcode.
pub enum OutputPolicy<'a> {
	/// Return reference to fixed sized output.
	/// Used for message calls.
	Return(BytesRef<'a>),
	/// Init new contract as soon as `RETURN` is called.
	InitContract
}

/// Transaction properties that externalities need to know about.
pub struct OriginInfo {
	sender: Address,
	value: U256,
	address: Address,
	origin: Address,
	gas_price: U256
}

impl OriginInfo {
	/// Populates origin info from action params.
	pub fn from(params: &ActionParams) -> Self {
		OriginInfo {
			sender: params.sender.clone(),
			value: params.value.clone(),
			address: params.address.clone(),
			origin: params.origin.clone(),
			gas_price: params.gas_price.clone()
		}
	}
}

/// Implementation of evm Externalities.
pub struct Externalities<'a> {
	state: &'a mut State,
	env_info: &'a EnvInfo,
	engine: &'a Engine,
	depth: usize,
	origin_info: OriginInfo,
	substate: &'a mut Substate,
	schedule: Schedule,
	output: OutputPolicy<'a>
}

impl<'a> Externalities<'a> {
	/// Basic `Externalities` constructor.
	pub fn new(state: &'a mut State, 
			   env_info: &'a EnvInfo, 
			   engine: &'a Engine, 
			   depth: usize,
			   origin_info: OriginInfo,
			   substate: &'a mut Substate, 
			   output: OutputPolicy<'a>) -> Self {
		Externalities {
			state: state,
			env_info: env_info,
			engine: engine,
			depth: depth,
			origin_info: origin_info,
			substate: substate,
			schedule: engine.schedule(env_info),
			output: output
		}
	}
}

impl<'a> Ext for Externalities<'a> {
	fn storage_at(&self, key: &H256) -> H256 {
		self.state.storage_at(&self.origin_info.address, key)
	}

	fn set_storage(&mut self, key: H256, value: H256) {
		self.state.set_storage(&self.origin_info.address, key, value)
	}

	fn exists(&self, address: &Address) -> bool {
		self.state.exists(address)
	}

	fn balance(&self, address: &Address) -> U256 {
		self.state.balance(address)
	}

	fn blockhash(&self, number: &U256) -> H256 {
		match *number < U256::from(self.env_info.number) && number.low_u64() >= cmp::max(256, self.env_info.number) - 256 {
			true => {
				let index = self.env_info.number - number.low_u64() - 1;
				let r = self.env_info.last_hashes[index as usize].clone();
				trace!("ext: blockhash({}) -> {} self.env_info.number={}\n", number, r, self.env_info.number);
				r
			},
			false => {
				trace!("ext: blockhash({}) -> null self.env_info.number={}\n", number, self.env_info.number);
				H256::from(&U256::zero())
			},
		}
	}

	fn create(&mut self, gas: &U256, value: &U256, code: &[u8]) -> ContractCreateResult {
		// create new contract address
		let address = contract_address(&self.origin_info.address, &self.state.nonce(&self.origin_info.address));

		// prepare the params
		let params = ActionParams {
			code_address: address.clone(),
			address: address.clone(),
			sender: self.origin_info.address.clone(),
			origin: self.origin_info.origin.clone(),
			gas: *gas,
			gas_price: self.origin_info.gas_price.clone(),
			value: value.clone(),
			is_value_transfer: true,
			code: Some(code.to_vec()),
			data: None,
		};

		self.state.inc_nonce(&self.origin_info.address);
		let mut ex = Executive::from_parent(self.state, self.env_info, self.engine, self.depth);
		
		// TODO: handle internal error separately
		match ex.create(params, self.substate) {
			Ok(gas_left) => {
				self.substate.contracts_created.push(address.clone());
				ContractCreateResult::Created(address, gas_left)
			},
			_ => ContractCreateResult::Failed
		}
	}

	fn delegatecall(&mut self, 
			gas: &U256, 
			data: &[u8], 
			code_address: &Address, 
			output: &mut [u8]) -> MessageCallResult {

		let params = ActionParams {
			code_address: code_address.clone(),
			address: self.origin_info.address.clone(), 
			sender: self.origin_info.sender.clone(),
			origin: self.origin_info.origin.clone(),
			gas: *gas,
			gas_price: self.origin_info.gas_price.clone(),
			value: self.origin_info.value.clone(),
			is_value_transfer: false,
			code: self.state.code(code_address),
			data: Some(data.to_vec()),
		};

		let mut ex = Executive::from_parent(self.state, self.env_info, self.engine, self.depth);
    
		match ex.call(params, self.substate, BytesRef::Fixed(output)) {
			Ok(gas_left) => MessageCallResult::Success(gas_left),
			_ => MessageCallResult::Failed
		}
  }

	fn call(&mut self, 
			gas: &U256, 
			address: &Address, 
			value: &U256, 
			data: &[u8], 
			code_address: &Address, 
			output: &mut [u8]) -> MessageCallResult {

		let params = ActionParams {
			code_address: code_address.clone(),
			address: address.clone(), 
			sender: self.origin_info.address.clone(),
			origin: self.origin_info.origin.clone(),
			gas: *gas,
			gas_price: self.origin_info.gas_price.clone(),
			value: value.clone(),
			is_value_transfer: true,
			code: self.state.code(code_address),
			data: Some(data.to_vec()),
		};

		let mut ex = Executive::from_parent(self.state, self.env_info, self.engine, self.depth);

		match ex.call(params, self.substate, BytesRef::Fixed(output)) {
			Ok(gas_left) => MessageCallResult::Success(gas_left),
			_ => MessageCallResult::Failed
		}
	}

	fn extcode(&self, address: &Address) -> Bytes {
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
				let address = &self.origin_info.address;
				self.state.init_code(address, code);
				Ok(*gas - return_cost)
			}
		}
	}

	fn log(&mut self, topics: Vec<H256>, data: &[u8]) {
		let address = self.origin_info.address.clone();
		self.substate.logs.push(LogEntry::new(address, topics, data.to_vec()));
	}

	fn suicide(&mut self, refund_address: &Address) {
		let address = self.origin_info.address.clone();
		let balance = self.balance(&address);
		self.state.transfer_balance(&address, refund_address, &balance);
		self.substate.suicides.insert(address);
	}

	fn schedule(&self) -> &Schedule {
		&self.schedule
	}

	fn env_info(&self) -> &EnvInfo {
		&self.env_info
	}

	fn depth(&self) -> usize {
		self.depth
	}

	fn inc_sstore_clears(&mut self) {
		self.substate.sstore_clears_count = self.substate.sstore_clears_count + U256::one();
	}
}
