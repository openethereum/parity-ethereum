// Copyright 2015-2017 Parity Technologies (UK) Ltd.
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

//! Transaction Execution environment.
use util::*;
use action_params::{ActionParams, ActionValue};
use state::{Backend as StateBackend, State, Substate, CleanupMode};
use engines::Engine;
use types::executed::CallType;
use env_info::EnvInfo;
use error::ExecutionError;
use evm::{self, Ext, Factory, Finalize};
use externalities::*;
use trace::{FlatTrace, Tracer, NoopTracer, ExecutiveTracer, VMTrace, VMTracer, ExecutiveVMTracer, NoopVMTracer};
use transaction::{Action, SignedTransaction};
use crossbeam;
pub use types::executed::{Executed, ExecutionResult};

/// Roughly estimate what stack size each level of evm depth will use
/// TODO [todr] We probably need some more sophisticated calculations here (limit on my machine 132)
/// Maybe something like here: `https://github.com/ethereum/libethereum/blob/4db169b8504f2b87f7d5a481819cfb959fc65f6c/libethereum/ExtVM.cpp`
const STACK_SIZE_PER_DEPTH: usize = 24*1024;

/// Returns new address created from address and given nonce.
pub fn contract_address(address: &Address, nonce: &U256) -> Address {
	use rlp::{RlpStream, Stream};

	let mut stream = RlpStream::new_list(2);
	stream.append(address);
	stream.append(nonce);
	From::from(stream.out().sha3())
}

/// Transaction execution options.
#[derive(Default, Copy, Clone, PartialEq)]
pub struct TransactOptions {
	/// Enable call tracing.
	pub tracing: bool,
	/// Enable VM tracing.
	pub vm_tracing: bool,
	/// Check transaction nonce before execution.
	pub check_nonce: bool,
}

/// Transaction executor.
pub struct Executive<'a, B: 'a + StateBackend> {
	state: &'a mut State<B>,
	info: &'a EnvInfo,
	engine: &'a Engine,
	vm_factory: &'a Factory,
	depth: usize,
}

impl<'a, B: 'a + StateBackend> Executive<'a, B> {
	/// Basic constructor.
	pub fn new(state: &'a mut State<B>, info: &'a EnvInfo, engine: &'a Engine, vm_factory: &'a Factory) -> Self {
		Executive {
			state: state,
			info: info,
			engine: engine,
			vm_factory: vm_factory,
			depth: 0,
		}
	}

	/// Populates executive from parent properties. Increments executive depth.
	pub fn from_parent(state: &'a mut State<B>, info: &'a EnvInfo, engine: &'a Engine, vm_factory: &'a Factory, parent_depth: usize) -> Self {
		Executive {
			state: state,
			info: info,
			engine: engine,
			vm_factory: vm_factory,
			depth: parent_depth + 1,
		}
	}

	/// Creates `Externalities` from `Executive`.
	pub fn as_externalities<'any, T, V>(
		&'any mut self,
		origin_info: OriginInfo,
		substate: &'any mut Substate,
		output: OutputPolicy<'any, 'any>,
		tracer: &'any mut T,
		vm_tracer: &'any mut V
	) -> Externalities<'any, T, V, B> where T: Tracer, V: VMTracer {
		Externalities::new(self.state, self.info, self.engine, self.vm_factory, self.depth, origin_info, substate, output, tracer, vm_tracer)
	}

	/// This function should be used to execute transaction.
	pub fn transact(&'a mut self, t: &SignedTransaction, options: TransactOptions) -> Result<Executed, ExecutionError> {
		let check = options.check_nonce;
		match options.tracing {
			true => match options.vm_tracing {
				true => self.transact_with_tracer(t, check, ExecutiveTracer::default(), ExecutiveVMTracer::toplevel()),
				false => self.transact_with_tracer(t, check, ExecutiveTracer::default(), NoopVMTracer),
			},
			false => match options.vm_tracing {
				true => self.transact_with_tracer(t, check, NoopTracer, ExecutiveVMTracer::toplevel()),
				false => self.transact_with_tracer(t, check, NoopTracer, NoopVMTracer),
			},
		}
	}

	/// Execute transaction/call with tracing enabled
	pub fn transact_with_tracer<T, V>(
		&'a mut self,
		t: &SignedTransaction,
		check_nonce: bool,
		mut tracer: T,
		mut vm_tracer: V
	) -> Result<Executed, ExecutionError> where T: Tracer, V: VMTracer {
		let sender = t.sender();
		let nonce = self.state.nonce(&sender)?;

		let schedule = self.engine.schedule(self.info);
		let base_gas_required = U256::from(t.gas_required(&schedule));

		if t.gas < base_gas_required {
			return Err(From::from(ExecutionError::NotEnoughBaseGas { required: base_gas_required, got: t.gas }));
		}

		let init_gas = t.gas - base_gas_required;

		// validate transaction nonce
		if check_nonce && t.nonce != nonce {
			return Err(From::from(ExecutionError::InvalidNonce { expected: nonce, got: t.nonce }));
		}

		// validate if transaction fits into given block
		if self.info.gas_used + t.gas > self.info.gas_limit {
			return Err(From::from(ExecutionError::BlockGasLimitReached {
				gas_limit: self.info.gas_limit,
				gas_used: self.info.gas_used,
				gas: t.gas
			}));
		}

		// TODO: we might need bigints here, or at least check overflows.
		let balance = self.state.balance(&sender)?;
		let gas_cost = t.gas.full_mul(t.gas_price);
		let total_cost = U512::from(t.value) + gas_cost;

		// avoid unaffordable transactions
		let balance512 = U512::from(balance);
		if balance512 < total_cost {
			return Err(From::from(ExecutionError::NotEnoughCash { required: total_cost, got: balance512 }));
		}

		// NOTE: there can be no invalid transactions from this point.
		self.state.inc_nonce(&sender)?;
		self.state.sub_balance(&sender, &U256::from(gas_cost))?;

		let mut substate = Substate::new();

		let (gas_left, output) = match t.action {
			Action::Create => {
				let new_address = contract_address(&sender, &nonce);
				let params = ActionParams {
					code_address: new_address.clone(),
					code_hash: t.data.sha3(),
					address: new_address,
					sender: sender.clone(),
					origin: sender.clone(),
					gas: init_gas,
					gas_price: t.gas_price,
					value: ActionValue::Transfer(t.value),
					code: Some(Arc::new(t.data.clone())),
					data: None,
					call_type: CallType::None,
				};
				(self.create(params, &mut substate, &mut tracer, &mut vm_tracer), vec![])
			},
			Action::Call(ref address) => {
				let params = ActionParams {
					code_address: address.clone(),
					address: address.clone(),
					sender: sender.clone(),
					origin: sender.clone(),
					gas: init_gas,
					gas_price: t.gas_price,
					value: ActionValue::Transfer(t.value),
					code: self.state.code(address)?,
					code_hash: self.state.code_hash(address)?,
					data: Some(t.data.clone()),
					call_type: CallType::Call,
				};
				let mut out = vec![];
				(self.call(params, &mut substate, BytesRef::Flexible(&mut out), &mut tracer, &mut vm_tracer), out)
			}
		};

		// finalize here!
		Ok(self.finalize(t, substate, gas_left, output, tracer.traces(), vm_tracer.drain())?)
	}

	fn exec_vm<T, V>(
		&mut self,
		params: ActionParams,
		unconfirmed_substate: &mut Substate,
		output_policy: OutputPolicy,
		tracer: &mut T,
		vm_tracer: &mut V
	) -> evm::Result<U256> where T: Tracer, V: VMTracer {

		let depth_threshold = ::io::LOCAL_STACK_SIZE.with(|sz| sz.get() / STACK_SIZE_PER_DEPTH);

		// Ordinary execution - keep VM in same thread
		if (self.depth + 1) % depth_threshold != 0 {
			let vm_factory = self.vm_factory;
			let mut ext = self.as_externalities(OriginInfo::from(&params), unconfirmed_substate, output_policy, tracer, vm_tracer);
			trace!(target: "executive", "ext.schedule.have_delegate_call: {}", ext.schedule().have_delegate_call);
			return vm_factory.create(params.gas).exec(params, &mut ext).finalize(ext);
		}

		// Start in new thread to reset stack
		// TODO [todr] No thread builder yet, so we need to reset once for a while
		// https://github.com/aturon/crossbeam/issues/16
		crossbeam::scope(|scope| {
			let vm_factory = self.vm_factory;
			let mut ext = self.as_externalities(OriginInfo::from(&params), unconfirmed_substate, output_policy, tracer, vm_tracer);

			scope.spawn(move || {
				vm_factory.create(params.gas).exec(params, &mut ext).finalize(ext)
			})
		}).join()
	}

	/// Calls contract function with given contract params.
	/// NOTE. It does not finalize the transaction (doesn't do refunds, nor suicides).
	/// Modifies the substate and the output.
	/// Returns either gas_left or `evm::Error`.
	pub fn call<T, V>(
		&mut self,
		params: ActionParams,
		substate: &mut Substate,
		mut output: BytesRef,
		tracer: &mut T,
		vm_tracer: &mut V
	) -> evm::Result<U256> where T: Tracer, V: VMTracer {
		// backup used in case of running out of gas
		self.state.checkpoint();

		let schedule = self.engine.schedule(self.info);

		// at first, transfer value to destination
		if let ActionValue::Transfer(val) = params.value {
			self.state.transfer_balance(&params.sender, &params.address, &val, substate.to_cleanup_mode(&schedule))?;
		}
		trace!("Executive::call(params={:?}) self.env_info={:?}", params, self.info);

		if self.engine.is_builtin(&params.code_address) {
			// if destination is builtin, try to execute it

			let default = [];
			let data = if let Some(ref d) = params.data { d as &[u8] } else { &default as &[u8] };

			let trace_info = tracer.prepare_trace_call(&params);

			let cost = self.engine.cost_of_builtin(&params.code_address, data);
			if cost <= params.gas {
				self.engine.execute_builtin(&params.code_address, data, &mut output);
				self.state.discard_checkpoint();

				// trace only top level calls to builtins to avoid DDoS attacks
				if self.depth == 0 {
					let mut trace_output = tracer.prepare_trace_output();
					if let Some(mut out) = trace_output.as_mut() {
						*out = output.to_owned();
					}

					tracer.trace_call(
						trace_info,
						cost,
						trace_output,
						vec![]
					);
				}

				Ok(params.gas - cost)
			} else {
				// just drain the whole gas
				self.state.revert_to_checkpoint();

				tracer.trace_failed_call(trace_info, vec![], evm::Error::OutOfGas.into());

				Err(evm::Error::OutOfGas)
			}
		} else {
			let trace_info = tracer.prepare_trace_call(&params);
			let mut trace_output = tracer.prepare_trace_output();
			let mut subtracer = tracer.subtracer();

			let gas = params.gas;

			if params.code.is_some() {
				// part of substate that may be reverted
				let mut unconfirmed_substate = Substate::new();

				// TODO: make ActionParams pass by ref then avoid copy altogether.
				let mut subvmtracer = vm_tracer.prepare_subtrace(params.code.as_ref().expect("scope is conditional on params.code.is_some(); qed"));

				let res = {
					self.exec_vm(params, &mut unconfirmed_substate, OutputPolicy::Return(output, trace_output.as_mut()), &mut subtracer, &mut subvmtracer)
				};

				vm_tracer.done_subtrace(subvmtracer);

				trace!(target: "executive", "res={:?}", res);

				let traces = subtracer.traces();
				match res {
					Ok(ref gas_left) => tracer.trace_call(
						trace_info,
						gas - *gas_left,
						trace_output,
						traces
					),
					Err(ref e) => tracer.trace_failed_call(trace_info, traces, e.into()),
				};

				trace!(target: "executive", "substate={:?}; unconfirmed_substate={:?}\n", substate, unconfirmed_substate);

				self.enact_result(&res, substate, unconfirmed_substate);
				trace!(target: "executive", "enacted: substate={:?}\n", substate);
				res
			} else {
				// otherwise it's just a basic transaction, only do tracing, if necessary.
				self.state.discard_checkpoint();

				tracer.trace_call(trace_info, U256::zero(), trace_output, vec![]);
				Ok(params.gas)
			}
		}
	}

	/// Creates contract with given contract params.
	/// NOTE. It does not finalize the transaction (doesn't do refunds, nor suicides).
	/// Modifies the substate.
	pub fn create<T, V>(
		&mut self,
		params: ActionParams,
		substate: &mut Substate,
		tracer: &mut T,
		vm_tracer: &mut V
	) -> evm::Result<U256> where T: Tracer, V: VMTracer {
		// backup used in case of running out of gas
		self.state.checkpoint();

		// part of substate that may be reverted
		let mut unconfirmed_substate = Substate::new();

		// create contract and transfer value to it if necessary
		let schedule = self.engine.schedule(self.info);
		let nonce_offset = if schedule.no_empty {1} else {0}.into();
		let prev_bal = self.state.balance(&params.address)?;
		if let ActionValue::Transfer(val) = params.value {
			self.state.sub_balance(&params.sender, &val)?;
			self.state.new_contract(&params.address, val + prev_bal, nonce_offset);
		} else {
			self.state.new_contract(&params.address, prev_bal, nonce_offset);
		}

		let trace_info = tracer.prepare_trace_create(&params);
		let mut trace_output = tracer.prepare_trace_output();
		let mut subtracer = tracer.subtracer();
		let gas = params.gas;
		let created = params.address.clone();

		let mut subvmtracer = vm_tracer.prepare_subtrace(params.code.as_ref().expect("two ways into create (Externalities::create and Executive::transact_with_tracer); both place `Some(...)` `code` in `params`; qed"));

		let res = {
			self.exec_vm(params, &mut unconfirmed_substate, OutputPolicy::InitContract(trace_output.as_mut()), &mut subtracer, &mut subvmtracer)
		};

		vm_tracer.done_subtrace(subvmtracer);

		match res {
			Ok(ref gas_left) => tracer.trace_create(
				trace_info,
				gas - *gas_left,
				trace_output,
				created,
				subtracer.traces()
			),
			Err(ref e) => tracer.trace_failed_create(trace_info, subtracer.traces(), e.into())
		};

		self.enact_result(&res, substate, unconfirmed_substate);
		res
	}

	/// Finalizes the transaction (does refunds and suicides).
	fn finalize(
		&mut self,
		t: &SignedTransaction,
		mut substate: Substate,
		result: evm::Result<U256>,
		output: Bytes,
		trace: Vec<FlatTrace>,
		vm_trace: Option<VMTrace>
	) -> ExecutionResult {
		let schedule = self.engine.schedule(self.info);

		// refunds from SSTORE nonzero -> zero
		let sstore_refunds = U256::from(schedule.sstore_refund_gas) * substate.sstore_clears_count;
		// refunds from contract suicides
		let suicide_refunds = U256::from(schedule.suicide_refund_gas) * U256::from(substate.suicides.len());
		let refunds_bound = sstore_refunds + suicide_refunds;

		// real ammount to refund
		let gas_left_prerefund = match result { Ok(x) => x, _ => 0.into() };
		let refunded = cmp::min(refunds_bound, (t.gas - gas_left_prerefund) >> 1);
		let gas_left = gas_left_prerefund + refunded;

		let gas_used = t.gas - gas_left;
		let refund_value = gas_left * t.gas_price;
		let fees_value = gas_used * t.gas_price;

		trace!("exec::finalize: t.gas={}, sstore_refunds={}, suicide_refunds={}, refunds_bound={}, gas_left_prerefund={}, refunded={}, gas_left={}, gas_used={}, refund_value={}, fees_value={}\n",
			t.gas, sstore_refunds, suicide_refunds, refunds_bound, gas_left_prerefund, refunded, gas_left, gas_used, refund_value, fees_value);

		let sender = t.sender();
		trace!("exec::finalize: Refunding refund_value={}, sender={}\n", refund_value, sender);
		// Below: NoEmpty is safe since the sender must already be non-null to have sent this transaction
		self.state.add_balance(&sender, &refund_value, CleanupMode::NoEmpty)?;
		trace!("exec::finalize: Compensating author: fees_value={}, author={}\n", fees_value, &self.info.author);
		self.state.add_balance(&self.info.author, &fees_value, substate.to_cleanup_mode(&schedule))?;

		// perform suicides
		for address in &substate.suicides {
			self.state.kill_account(address);
		}

		// perform garbage-collection
		for address in &substate.garbage {
			if self.state.exists(address)? && !self.state.exists_and_not_null(address)? {
				self.state.kill_account(address);
			}
		}

		match result {
			Err(evm::Error::Internal(msg)) => Err(ExecutionError::Internal(msg)),
			Err(exception) => {
				Ok(Executed {
					exception: Some(exception),
					gas: t.gas,
					gas_used: t.gas,
					refunded: U256::zero(),
					cumulative_gas_used: self.info.gas_used + t.gas,
					logs: vec![],
					contracts_created: vec![],
					output: output,
					trace: trace,
					vm_trace: vm_trace,
					state_diff: None,
				})
			},
			_ => {
				Ok(Executed {
					exception: None,
					gas: t.gas,
					gas_used: gas_used,
					refunded: refunded,
					cumulative_gas_used: self.info.gas_used + gas_used,
					logs: substate.logs,
					contracts_created: substate.contracts_created,
					output: output,
					trace: trace,
					vm_trace: vm_trace,
					state_diff: None,
				})
			},
		}
	}

	fn enact_result(&mut self, result: &evm::Result<U256>, substate: &mut Substate, un_substate: Substate) {
		match *result {
			Err(evm::Error::OutOfGas)
				| Err(evm::Error::BadJumpDestination {..})
				| Err(evm::Error::BadInstruction {.. })
				| Err(evm::Error::StackUnderflow {..})
				| Err(evm::Error::OutOfStack {..}) => {
					self.state.revert_to_checkpoint();
			},
			Ok(_) | Err(evm::Error::Internal(_)) => {
				self.state.discard_checkpoint();
				substate.accrue(un_substate);
			}
		}
	}
}

#[cfg(test)]
#[allow(dead_code)]
mod tests {
	use std::sync::Arc;
	use ethkey::{Generator, Random};
	use super::*;
	use util::{H256, U256, U512, Address, Uint, FixedHash, FromHex, FromStr};
	use util::bytes::BytesRef;
	use action_params::{ActionParams, ActionValue};
	use env_info::EnvInfo;
	use evm::{Factory, VMType};
	use error::ExecutionError;
	use state::{Substate, CleanupMode};
	use tests::helpers::*;
	use trace::trace;
	use trace::{FlatTrace, Tracer, NoopTracer, ExecutiveTracer};
	use trace::{VMTrace, VMOperation, VMExecutedOperation, MemoryDiff, StorageDiff, VMTracer, NoopVMTracer, ExecutiveVMTracer};
	use transaction::{Action, Transaction};

	use types::executed::CallType;

	#[test]
	fn test_contract_address() {
		let address = Address::from_str("0f572e5295c57f15886f9b263e2f6d2d6c7b5ec6").unwrap();
		let expected_address = Address::from_str("3f09c73a5ed19289fb9bdc72f1742566df146f56").unwrap();
		assert_eq!(expected_address, contract_address(&address, &U256::from(88)));
	}

	// TODO: replace params with transactions!
	evm_test!{test_sender_balance: test_sender_balance_jit, test_sender_balance_int}
	fn test_sender_balance(factory: Factory) {
		let sender = Address::from_str("0f572e5295c57f15886f9b263e2f6d2d6c7b5ec6").unwrap();
		let address = contract_address(&sender, &U256::zero());
		let mut params = ActionParams::default();
		params.address = address.clone();
		params.sender = sender.clone();
		params.gas = U256::from(100_000);
		params.code = Some(Arc::new("3331600055".from_hex().unwrap()));
		params.value = ActionValue::Transfer(U256::from(0x7));
		let mut state_result = get_temp_state();
		let mut state = state_result.reference_mut();
		state.add_balance(&sender, &U256::from(0x100u64), CleanupMode::NoEmpty).unwrap();
		let info = EnvInfo::default();
		let engine = TestEngine::new(0);
		let mut substate = Substate::new();

		let gas_left = {
			let mut ex = Executive::new(&mut state, &info, &engine, &factory);
			ex.create(params, &mut substate, &mut NoopTracer, &mut NoopVMTracer).unwrap()
		};

		assert_eq!(gas_left, U256::from(79_975));
		assert_eq!(state.storage_at(&address, &H256::new()).unwrap(), H256::from(&U256::from(0xf9u64)));
		assert_eq!(state.balance(&sender).unwrap(), U256::from(0xf9));
		assert_eq!(state.balance(&address).unwrap(), U256::from(0x7));
		// 0 cause contract hasn't returned
		assert_eq!(substate.contracts_created.len(), 0);

		// TODO: just test state root.
	}

	evm_test!{test_create_contract_out_of_depth: test_create_contract_out_of_depth_jit, test_create_contract_out_of_depth_int}
	fn test_create_contract_out_of_depth(factory: Factory) {
		// code:
		//
		// 7c 601080600c6000396000f3006000355415600957005b60203560003555 - push 29 bytes?
		// 60 00 - push 0
		// 52
		// 60 1d - push 29
		// 60 03 - push 3
		// 60 17 - push 17
		// f0 - create
		// 60 00 - push 0
		// 55 sstore
		//
		// other code:
		//
		// 60 10 - push 16
		// 80 - duplicate first stack item
		// 60 0c - push 12
		// 60 00 - push 0
		// 39 - copy current code to memory
		// 60 00 - push 0
		// f3 - return

		let code = "7c601080600c6000396000f3006000355415600957005b60203560003555600052601d60036017f0600055".from_hex().unwrap();

		let sender = Address::from_str("cd1722f3947def4cf144679da39c4c32bdc35681").unwrap();
		let address = contract_address(&sender, &U256::zero());
		// TODO: add tests for 'callcreate'
		//let next_address = contract_address(&address, &U256::zero());
		let mut params = ActionParams::default();
		params.address = address.clone();
		params.sender = sender.clone();
		params.origin = sender.clone();
		params.gas = U256::from(100_000);
		params.code = Some(Arc::new(code));
		params.value = ActionValue::Transfer(U256::from(100));
		let mut state_result = get_temp_state();
		let mut state = state_result.reference_mut();
		state.add_balance(&sender, &U256::from(100), CleanupMode::NoEmpty).unwrap();
		let info = EnvInfo::default();
		let engine = TestEngine::new(0);
		let mut substate = Substate::new();

		let gas_left = {
			let mut ex = Executive::new(&mut state, &info, &engine, &factory);
			ex.create(params, &mut substate, &mut NoopTracer, &mut NoopVMTracer).unwrap()
		};

		assert_eq!(gas_left, U256::from(62_976));
		// ended with max depth
		assert_eq!(substate.contracts_created.len(), 0);
	}

	#[test]
	// Tracing is not suported in JIT
	fn test_call_to_create() {
		let factory = Factory::new(VMType::Interpreter, 1024 * 32);

		// code:
		//
		// 7c 601080600c6000396000f3006000355415600957005b60203560003555 - push 29 bytes?
		// 60 00 - push 0
		// 52
		// 60 1d - push 29
		// 60 03 - push 3
		// 60 17 - push 23
		// f0 - create
		// 60 00 - push 0
		// 55 sstore
		//
		// other code:
		//
		// 60 10 - push 16
		// 80 - duplicate first stack item
		// 60 0c - push 12
		// 60 00 - push 0
		// 39 - copy current code to memory
		// 60 00 - push 0
		// f3 - return

		let code = "7c601080600c6000396000f3006000355415600957005b60203560003555600052601d60036017f0600055".from_hex().unwrap();

		let sender = Address::from_str("cd1722f3947def4cf144679da39c4c32bdc35681").unwrap();
		let address = contract_address(&sender, &U256::zero());
		// TODO: add tests for 'callcreate'
		//let next_address = contract_address(&address, &U256::zero());
		let mut params = ActionParams::default();
		params.address = address.clone();
		params.code_address = address.clone();
		params.sender = sender.clone();
		params.origin = sender.clone();
		params.gas = U256::from(100_000);
		params.code = Some(Arc::new(code));
		params.value = ActionValue::Transfer(U256::from(100));
		params.call_type = CallType::Call;
		let mut state_result = get_temp_state();
		let mut state = state_result.reference_mut();
		state.add_balance(&sender, &U256::from(100), CleanupMode::NoEmpty).unwrap();
		let info = EnvInfo::default();
		let engine = TestEngine::new(5);
		let mut substate = Substate::new();
		let mut tracer = ExecutiveTracer::default();
		let mut vm_tracer = ExecutiveVMTracer::toplevel();

		let gas_left = {
			let mut ex = Executive::new(&mut state, &info, &engine, &factory);
			let output = BytesRef::Fixed(&mut[0u8;0]);
			ex.call(params, &mut substate, output, &mut tracer, &mut vm_tracer).unwrap()
		};

		assert_eq!(gas_left, U256::from(44_752));

		let expected_trace = vec![FlatTrace {
			trace_address: Default::default(),
			subtraces: 1,
			action: trace::Action::Call(trace::Call {
				from: "cd1722f3947def4cf144679da39c4c32bdc35681".into(),
				to: "b010143a42d5980c7e5ef0e4a4416dc098a4fed3".into(),
				value: 100.into(),
				gas: 100000.into(),
				input: vec![],
				call_type: CallType::Call,
			}),
			result: trace::Res::Call(trace::CallResult {
				gas_used: U256::from(55_248),
				output: vec![],
			}),
		}, FlatTrace {
			trace_address: vec![0].into_iter().collect(),
			subtraces: 0,
			action: trace::Action::Create(trace::Create {
				from: "b010143a42d5980c7e5ef0e4a4416dc098a4fed3".into(),
				value: 23.into(),
				gas: 67979.into(),
				init: vec![96, 16, 128, 96, 12, 96, 0, 57, 96, 0, 243, 0, 96, 0, 53, 84, 21, 96, 9, 87, 0, 91, 96, 32, 53, 96, 0, 53, 85]
			}),
			result: trace::Res::Create(trace::CreateResult {
				gas_used: U256::from(3224),
				address: Address::from_str("c6d80f262ae5e0f164e5fde365044d7ada2bfa34").unwrap(),
				code: vec![96, 0, 53, 84, 21, 96, 9, 87, 0, 91, 96, 32, 53, 96, 0, 53]
			}),
		}];

		assert_eq!(tracer.traces(), expected_trace);

		let expected_vm_trace = VMTrace {
			parent_step: 0,
			code: vec![124, 96, 16, 128, 96, 12, 96, 0, 57, 96, 0, 243, 0, 96, 0, 53, 84, 21, 96, 9, 87, 0, 91, 96, 32, 53, 96, 0, 53, 85, 96, 0, 82, 96, 29, 96, 3, 96, 23, 240, 96, 0, 85],
			operations: vec![
				VMOperation { pc: 0, instruction: 124, gas_cost: 3.into(), executed: Some(VMExecutedOperation { gas_used: 99997.into(), stack_push: vec_into![U256::from_dec_str("2589892687202724018173567190521546555304938078595079151649957320078677").unwrap()], mem_diff: None, store_diff: None }) },
				VMOperation { pc: 30, instruction: 96, gas_cost: 3.into(), executed: Some(VMExecutedOperation { gas_used: 99994.into(), stack_push: vec_into![0], mem_diff: None, store_diff: None }) },
				VMOperation { pc: 32, instruction: 82, gas_cost: 6.into(), executed: Some(VMExecutedOperation { gas_used: 99988.into(), stack_push: vec_into![], mem_diff: Some(MemoryDiff { offset: 0, data: vec![0, 0, 0, 96, 16, 128, 96, 12, 96, 0, 57, 96, 0, 243, 0, 96, 0, 53, 84, 21, 96, 9, 87, 0, 91, 96, 32, 53, 96, 0, 53, 85] }), store_diff: None }) },
				VMOperation { pc: 33, instruction: 96, gas_cost: 3.into(), executed: Some(VMExecutedOperation { gas_used: 99985.into(), stack_push: vec_into![29], mem_diff: None, store_diff: None }) },
				VMOperation { pc: 35, instruction: 96, gas_cost: 3.into(), executed: Some(VMExecutedOperation { gas_used: 99982.into(), stack_push: vec_into![3], mem_diff: None, store_diff: None }) },
				VMOperation { pc: 37, instruction: 96, gas_cost: 3.into(), executed: Some(VMExecutedOperation { gas_used: 99979.into(), stack_push: vec_into![23], mem_diff: None, store_diff: None }) },
				VMOperation { pc: 39, instruction: 240, gas_cost: 99979.into(), executed: Some(VMExecutedOperation { gas_used: 64755.into(), stack_push: vec_into![U256::from_dec_str("1135198453258042933984631383966629874710669425204").unwrap()], mem_diff: None, store_diff: None }) },
				VMOperation { pc: 40, instruction: 96, gas_cost: 3.into(), executed: Some(VMExecutedOperation { gas_used: 64752.into(), stack_push: vec_into![0], mem_diff: None, store_diff: None }) },
				VMOperation { pc: 42, instruction: 85, gas_cost: 20000.into(), executed: Some(VMExecutedOperation { gas_used: 44752.into(), stack_push: vec_into![], mem_diff: None, store_diff: Some(StorageDiff { location: 0.into(), value: U256::from_dec_str("1135198453258042933984631383966629874710669425204").unwrap() }) }) }
			],
			subs: vec![
				VMTrace {
					parent_step: 6,
					code: vec![96, 16, 128, 96, 12, 96, 0, 57, 96, 0, 243, 0, 96, 0, 53, 84, 21, 96, 9, 87, 0, 91, 96, 32, 53, 96, 0, 53, 85],
					operations: vec![
						VMOperation { pc: 0, instruction: 96, gas_cost: 3.into(), executed: Some(VMExecutedOperation { gas_used: 67976.into(), stack_push: vec_into![16], mem_diff: None, store_diff: None }) },
						VMOperation { pc: 2, instruction: 128, gas_cost: 3.into(), executed: Some(VMExecutedOperation { gas_used: 67973.into(), stack_push: vec_into![16, 16], mem_diff: None, store_diff: None }) },
						VMOperation { pc: 3, instruction: 96, gas_cost: 3.into(), executed: Some(VMExecutedOperation { gas_used: 67970.into(), stack_push: vec_into![12], mem_diff: None, store_diff: None }) },
						VMOperation { pc: 5, instruction: 96, gas_cost: 3.into(), executed: Some(VMExecutedOperation { gas_used: 67967.into(), stack_push: vec_into![0], mem_diff: None, store_diff: None }) },
						VMOperation { pc: 7, instruction: 57, gas_cost: 9.into(), executed: Some(VMExecutedOperation { gas_used: 67958.into(), stack_push: vec_into![], mem_diff: Some(MemoryDiff { offset: 0, data: vec![96, 0, 53, 84, 21, 96, 9, 87, 0, 91, 96, 32, 53, 96, 0, 53] }), store_diff: None }) },
						VMOperation { pc: 8, instruction: 96, gas_cost: 3.into(), executed: Some(VMExecutedOperation { gas_used: 67955.into(), stack_push: vec_into![0], mem_diff: None, store_diff: None }) },
						VMOperation { pc: 10, instruction: 243, gas_cost: 0.into(), executed: Some(VMExecutedOperation { gas_used: 67955.into(), stack_push: vec_into![], mem_diff: None, store_diff: None }) }
					],
					subs: vec![]
				}
			]
		};
		assert_eq!(vm_tracer.drain().unwrap(), expected_vm_trace);
	}

	#[test]
	fn test_create_contract() {
		// Tracing is not supported in JIT
		let factory = Factory::new(VMType::Interpreter, 1024 * 32);
		// code:
		//
		// 60 10 - push 16
		// 80 - duplicate first stack item
		// 60 0c - push 12
		// 60 00 - push 0
		// 39 - copy current code to memory
		// 60 00 - push 0
		// f3 - return

		let code = "601080600c6000396000f3006000355415600957005b60203560003555".from_hex().unwrap();

		let sender = Address::from_str("cd1722f3947def4cf144679da39c4c32bdc35681").unwrap();
		let address = contract_address(&sender, &U256::zero());
		// TODO: add tests for 'callcreate'
		//let next_address = contract_address(&address, &U256::zero());
		let mut params = ActionParams::default();
		params.address = address.clone();
		params.sender = sender.clone();
		params.origin = sender.clone();
		params.gas = U256::from(100_000);
		params.code = Some(Arc::new(code));
		params.value = ActionValue::Transfer(100.into());
		let mut state_result = get_temp_state();
		let mut state = state_result.reference_mut();
		state.add_balance(&sender, &U256::from(100), CleanupMode::NoEmpty).unwrap();
		let info = EnvInfo::default();
		let engine = TestEngine::new(5);
		let mut substate = Substate::new();
		let mut tracer = ExecutiveTracer::default();
		let mut vm_tracer = ExecutiveVMTracer::toplevel();

		let gas_left = {
			let mut ex = Executive::new(&mut state, &info, &engine, &factory);
			ex.create(params.clone(), &mut substate, &mut tracer, &mut vm_tracer).unwrap()
		};

		assert_eq!(gas_left, U256::from(96_776));

		let expected_trace = vec![FlatTrace {
			trace_address: Default::default(),
			subtraces: 0,
			action: trace::Action::Create(trace::Create {
				from: params.sender,
				value: 100.into(),
				gas: params.gas,
				init: vec![96, 16, 128, 96, 12, 96, 0, 57, 96, 0, 243, 0, 96, 0, 53, 84, 21, 96, 9, 87, 0, 91, 96, 32, 53, 96, 0, 53, 85],
			}),
			result: trace::Res::Create(trace::CreateResult {
				gas_used: U256::from(3224),
				address: params.address,
				code: vec![96, 0, 53, 84, 21, 96, 9, 87, 0, 91, 96, 32, 53, 96, 0, 53]
			}),
		}];

		assert_eq!(tracer.traces(), expected_trace);

		let expected_vm_trace = VMTrace {
			parent_step: 0,
			code: vec![96, 16, 128, 96, 12, 96, 0, 57, 96, 0, 243, 0, 96, 0, 53, 84, 21, 96, 9, 87, 0, 91, 96, 32, 53, 96, 0, 53, 85],
			operations: vec![
				VMOperation { pc: 0, instruction: 96, gas_cost: 3.into(), executed: Some(VMExecutedOperation { gas_used: 99997.into(), stack_push: vec_into![16], mem_diff: None, store_diff: None }) },
				VMOperation { pc: 2, instruction: 128, gas_cost: 3.into(), executed: Some(VMExecutedOperation { gas_used: 99994.into(), stack_push: vec_into![16, 16], mem_diff: None, store_diff: None }) },
				VMOperation { pc: 3, instruction: 96, gas_cost: 3.into(), executed: Some(VMExecutedOperation { gas_used: 99991.into(), stack_push: vec_into![12], mem_diff: None, store_diff: None }) },
				VMOperation { pc: 5, instruction: 96, gas_cost: 3.into(), executed: Some(VMExecutedOperation { gas_used: 99988.into(), stack_push: vec_into![0], mem_diff: None, store_diff: None }) },
				VMOperation { pc: 7, instruction: 57, gas_cost: 9.into(), executed: Some(VMExecutedOperation { gas_used: 99979.into(), stack_push: vec_into![], mem_diff: Some(MemoryDiff { offset: 0, data: vec![96, 0, 53, 84, 21, 96, 9, 87, 0, 91, 96, 32, 53, 96, 0, 53] }), store_diff: None }) },
				VMOperation { pc: 8, instruction: 96, gas_cost: 3.into(), executed: Some(VMExecutedOperation { gas_used: 99976.into(), stack_push: vec_into![0], mem_diff: None, store_diff: None }) },
				VMOperation { pc: 10, instruction: 243, gas_cost: 0.into(), executed: Some(VMExecutedOperation { gas_used: 99976.into(), stack_push: vec_into![], mem_diff: None, store_diff: None }) }
			],
			subs: vec![]
		};
		assert_eq!(vm_tracer.drain().unwrap(), expected_vm_trace);
	}

	evm_test!{test_create_contract_value_too_high: test_create_contract_value_too_high_jit, test_create_contract_value_too_high_int}
	fn test_create_contract_value_too_high(factory: Factory) {
		// code:
		//
		// 7c 601080600c6000396000f3006000355415600957005b60203560003555 - push 29 bytes?
		// 60 00 - push 0
		// 52
		// 60 1d - push 29
		// 60 03 - push 3
		// 60 e6 - push 230
		// f0 - create a contract trying to send 230.
		// 60 00 - push 0
		// 55 sstore
		//
		// other code:
		//
		// 60 10 - push 16
		// 80 - duplicate first stack item
		// 60 0c - push 12
		// 60 00 - push 0
		// 39 - copy current code to memory
		// 60 00 - push 0
		// f3 - return

		let code = "7c601080600c6000396000f3006000355415600957005b60203560003555600052601d600360e6f0600055".from_hex().unwrap();

		let sender = Address::from_str("cd1722f3947def4cf144679da39c4c32bdc35681").unwrap();
		let address = contract_address(&sender, &U256::zero());
		// TODO: add tests for 'callcreate'
		//let next_address = contract_address(&address, &U256::zero());
		let mut params = ActionParams::default();
		params.address = address.clone();
		params.sender = sender.clone();
		params.origin = sender.clone();
		params.gas = U256::from(100_000);
		params.code = Some(Arc::new(code));
		params.value = ActionValue::Transfer(U256::from(100));
		let mut state_result = get_temp_state();
		let mut state = state_result.reference_mut();
		state.add_balance(&sender, &U256::from(100), CleanupMode::NoEmpty).unwrap();
		let info = EnvInfo::default();
		let engine = TestEngine::new(0);
		let mut substate = Substate::new();

		let gas_left = {
			let mut ex = Executive::new(&mut state, &info, &engine, &factory);
			ex.create(params, &mut substate, &mut NoopTracer, &mut NoopVMTracer).unwrap()
		};

		assert_eq!(gas_left, U256::from(62_976));
		assert_eq!(substate.contracts_created.len(), 0);
	}

	evm_test!{test_create_contract_without_max_depth: test_create_contract_without_max_depth_jit, test_create_contract_without_max_depth_int}
	fn test_create_contract_without_max_depth(factory: Factory) {
		// code:
		//
		// 7c 601080600c6000396000f3006000355415600957005b60203560003555 - push 29 bytes?
		// 60 00 - push 0
		// 52
		// 60 1d - push 29
		// 60 03 - push 3
		// 60 17 - push 17
		// f0 - create
		// 60 00 - push 0
		// 55 sstore
		//
		// other code:
		//
		// 60 10 - push 16
		// 80 - duplicate first stack item
		// 60 0c - push 12
		// 60 00 - push 0
		// 39 - copy current code to memory
		// 60 00 - push 0
		// f3 - return

		let code = "7c601080600c6000396000f3006000355415600957005b60203560003555600052601d60036017f0".from_hex().unwrap();

		let sender = Address::from_str("cd1722f3947def4cf144679da39c4c32bdc35681").unwrap();
		let address = contract_address(&sender, &U256::zero());
		let next_address = contract_address(&address, &U256::zero());
		let mut params = ActionParams::default();
		params.address = address.clone();
		params.sender = sender.clone();
		params.origin = sender.clone();
		params.gas = U256::from(100_000);
		params.code = Some(Arc::new(code));
		params.value = ActionValue::Transfer(U256::from(100));
		let mut state_result = get_temp_state();
		let mut state = state_result.reference_mut();
		state.add_balance(&sender, &U256::from(100), CleanupMode::NoEmpty).unwrap();
		let info = EnvInfo::default();
		let engine = TestEngine::new(1024);
		let mut substate = Substate::new();

		{
			let mut ex = Executive::new(&mut state, &info, &engine, &factory);
			ex.create(params, &mut substate, &mut NoopTracer, &mut NoopVMTracer).unwrap();
		}

		assert_eq!(substate.contracts_created.len(), 1);
		assert_eq!(substate.contracts_created[0], next_address);
	}

	// test is incorrect, mk
	// TODO: fix (preferred) or remove
	evm_test_ignore!{test_aba_calls: test_aba_calls_jit, test_aba_calls_int}
	fn test_aba_calls(factory: Factory) {
		// 60 00 - push 0
		// 60 00 - push 0
		// 60 00 - push 0
		// 60 00 - push 0
		// 60 18 - push 18
		// 73 945304eb96065b2a98b57a48a06ae28d285a71b5 - push this address
		// 61 03e8 - push 1000
		// f1 - message call
		// 58 - get PC
		// 55 - sstore

		let code_a = "6000600060006000601873945304eb96065b2a98b57a48a06ae28d285a71b56103e8f15855".from_hex().unwrap();

		// 60 00 - push 0
		// 60 00 - push 0
		// 60 00 - push 0
		// 60 00 - push 0
		// 60 17 - push 17
		// 73 0f572e5295c57f15886f9b263e2f6d2d6c7b5ec6 - push this address
		// 61 0x01f4 - push 500
		// f1 - message call
		// 60 01 - push 1
		// 01 - add
		// 58 - get PC
		// 55 - sstore
		let code_b = "60006000600060006017730f572e5295c57f15886f9b263e2f6d2d6c7b5ec66101f4f16001015855".from_hex().unwrap();

		let address_a = Address::from_str("0f572e5295c57f15886f9b263e2f6d2d6c7b5ec6").unwrap();
		let address_b = Address::from_str("945304eb96065b2a98b57a48a06ae28d285a71b5" ).unwrap();
		let sender = Address::from_str("cd1722f3947def4cf144679da39c4c32bdc35681").unwrap();

		let mut params = ActionParams::default();
		params.address = address_a.clone();
		params.sender = sender.clone();
		params.gas = U256::from(100_000);
		params.code = Some(Arc::new(code_a.clone()));
		params.value = ActionValue::Transfer(U256::from(100_000));

		let mut state_result = get_temp_state();
		let mut state = state_result.reference_mut();
		state.init_code(&address_a, code_a.clone()).unwrap();
		state.init_code(&address_b, code_b.clone()).unwrap();
		state.add_balance(&sender, &U256::from(100_000), CleanupMode::NoEmpty).unwrap();

		let info = EnvInfo::default();
		let engine = TestEngine::new(0);
		let mut substate = Substate::new();

		let gas_left = {
			let mut ex = Executive::new(&mut state, &info, &engine, &factory);
			ex.call(params, &mut substate, BytesRef::Fixed(&mut []), &mut NoopTracer, &mut NoopVMTracer).unwrap()
		};

		assert_eq!(gas_left, U256::from(73_237));
		assert_eq!(state.storage_at(&address_a, &H256::from(&U256::from(0x23))).unwrap(), H256::from(&U256::from(1)));
	}

	// test is incorrect, mk
	// TODO: fix (preferred) or remove
	evm_test_ignore!{test_recursive_bomb1: test_recursive_bomb1_jit, test_recursive_bomb1_int}
	fn test_recursive_bomb1(factory: Factory) {
		// 60 01 - push 1
		// 60 00 - push 0
		// 54 - sload
		// 01 - add
		// 60 00 - push 0
		// 55 - sstore
		// 60 00 - push 0
		// 60 00 - push 0
		// 60 00 - push 0
		// 60 00 - push 0
		// 60 00 - push 0
		// 30 - load address
		// 60 e0 - push e0
		// 5a - get gas
		// 03 - sub
		// f1 - message call (self in this case)
		// 60 01 - push 1
		// 55 - sstore
		let sender = Address::from_str("cd1722f3947def4cf144679da39c4c32bdc35681").unwrap();
		let code = "600160005401600055600060006000600060003060e05a03f1600155".from_hex().unwrap();
		let address = contract_address(&sender, &U256::zero());
		let mut params = ActionParams::default();
		params.address = address.clone();
		params.gas = U256::from(100_000);
		params.code = Some(Arc::new(code.clone()));
		let mut state_result = get_temp_state();
		let mut state = state_result.reference_mut();
		state.init_code(&address, code).unwrap();
		let info = EnvInfo::default();
		let engine = TestEngine::new(0);
		let mut substate = Substate::new();

		let gas_left = {
			let mut ex = Executive::new(&mut state, &info, &engine, &factory);
			ex.call(params, &mut substate, BytesRef::Fixed(&mut []), &mut NoopTracer, &mut NoopVMTracer).unwrap()
		};

		assert_eq!(gas_left, U256::from(59_870));
		assert_eq!(state.storage_at(&address, &H256::from(&U256::zero())).unwrap(), H256::from(&U256::from(1)));
		assert_eq!(state.storage_at(&address, &H256::from(&U256::one())).unwrap(), H256::from(&U256::from(1)));
	}

	// test is incorrect, mk
	// TODO: fix (preferred) or remove
	evm_test_ignore!{test_transact_simple: test_transact_simple_jit, test_transact_simple_int}
	fn test_transact_simple(factory: Factory) {
		let keypair = Random.generate().unwrap();
		let t = Transaction {
			action: Action::Create,
			value: U256::from(17),
			data: "3331600055".from_hex().unwrap(),
			gas: U256::from(100_000),
			gas_price: U256::zero(),
			nonce: U256::zero()
		}.sign(keypair.secret(), None);
		let sender = t.sender();
		let contract = contract_address(&sender, &U256::zero());

		let mut state_result = get_temp_state();
		let mut state = state_result.reference_mut();
		state.add_balance(&sender, &U256::from(18), CleanupMode::NoEmpty).unwrap();
		let mut info = EnvInfo::default();
		info.gas_limit = U256::from(100_000);
		let engine = TestEngine::new(0);

		let executed = {
			let mut ex = Executive::new(&mut state, &info, &engine, &factory);
			let opts = TransactOptions { check_nonce: true, tracing: false, vm_tracing: false };
			ex.transact(&t, opts).unwrap()
		};

		assert_eq!(executed.gas, U256::from(100_000));
		assert_eq!(executed.gas_used, U256::from(41_301));
		assert_eq!(executed.refunded, U256::from(58_699));
		assert_eq!(executed.cumulative_gas_used, U256::from(41_301));
		assert_eq!(executed.logs.len(), 0);
		assert_eq!(executed.contracts_created.len(), 0);
		assert_eq!(state.balance(&sender).unwrap(), U256::from(1));
		assert_eq!(state.balance(&contract).unwrap(), U256::from(17));
		assert_eq!(state.nonce(&sender).unwrap(), U256::from(1));
		assert_eq!(state.storage_at(&contract, &H256::new()).unwrap(), H256::from(&U256::from(1)));
	}

	evm_test!{test_transact_invalid_nonce: test_transact_invalid_nonce_jit, test_transact_invalid_nonce_int}
	fn test_transact_invalid_nonce(factory: Factory) {
		let keypair = Random.generate().unwrap();
		let t = Transaction {
			action: Action::Create,
			value: U256::from(17),
			data: "3331600055".from_hex().unwrap(),
			gas: U256::from(100_000),
			gas_price: U256::zero(),
			nonce: U256::one()
		}.sign(keypair.secret(), None);
		let sender = t.sender();

		let mut state_result = get_temp_state();
		let mut state = state_result.reference_mut();
		state.add_balance(&sender, &U256::from(17), CleanupMode::NoEmpty).unwrap();
		let mut info = EnvInfo::default();
		info.gas_limit = U256::from(100_000);
		let engine = TestEngine::new(0);

		let res = {
			let mut ex = Executive::new(&mut state, &info, &engine, &factory);
			let opts = TransactOptions { check_nonce: true, tracing: false, vm_tracing: false };
			ex.transact(&t, opts)
		};

		match res {
			Err(ExecutionError::InvalidNonce { expected, got })
				if expected == U256::zero() && got == U256::one() => (),
			_ => assert!(false, "Expected invalid nonce error.")
		}
	}

	evm_test!{test_transact_gas_limit_reached: test_transact_gas_limit_reached_jit, test_transact_gas_limit_reached_int}
	fn test_transact_gas_limit_reached(factory: Factory) {
		let keypair = Random.generate().unwrap();
		let t = Transaction {
			action: Action::Create,
			value: U256::from(17),
			data: "3331600055".from_hex().unwrap(),
			gas: U256::from(80_001),
			gas_price: U256::zero(),
			nonce: U256::zero()
		}.sign(keypair.secret(), None);
		let sender = t.sender();

		let mut state_result = get_temp_state();
		let mut state = state_result.reference_mut();
		state.add_balance(&sender, &U256::from(17), CleanupMode::NoEmpty).unwrap();
		let mut info = EnvInfo::default();
		info.gas_used = U256::from(20_000);
		info.gas_limit = U256::from(100_000);
		let engine = TestEngine::new(0);

		let res = {
			let mut ex = Executive::new(&mut state, &info, &engine, &factory);
			let opts = TransactOptions { check_nonce: true, tracing: false, vm_tracing: false };
			ex.transact(&t, opts)
		};

		match res {
			Err(ExecutionError::BlockGasLimitReached { gas_limit, gas_used, gas })
				if gas_limit == U256::from(100_000) && gas_used == U256::from(20_000) && gas == U256::from(80_001) => (),
			_ => assert!(false, "Expected block gas limit error.")
		}
	}

	evm_test!{test_not_enough_cash: test_not_enough_cash_jit, test_not_enough_cash_int}
	fn test_not_enough_cash(factory: Factory) {

		let keypair = Random.generate().unwrap();
		let t = Transaction {
			action: Action::Create,
			value: U256::from(18),
			data: "3331600055".from_hex().unwrap(),
			gas: U256::from(100_000),
			gas_price: U256::one(),
			nonce: U256::zero()
		}.sign(keypair.secret(), None);
		let sender = t.sender();

		let mut state_result = get_temp_state();
		let mut state = state_result.reference_mut();
		state.add_balance(&sender, &U256::from(100_017), CleanupMode::NoEmpty).unwrap();
		let mut info = EnvInfo::default();
		info.gas_limit = U256::from(100_000);
		let engine = TestEngine::new(0);

		let res = {
			let mut ex = Executive::new(&mut state, &info, &engine, &factory);
			let opts = TransactOptions { check_nonce: true, tracing: false, vm_tracing: false };
			ex.transact(&t, opts)
		};

		match res {
			Err(ExecutionError::NotEnoughCash { required , got })
				if required == U512::from(100_018) && got == U512::from(100_017) => (),
			_ => assert!(false, "Expected not enough cash error. {:?}", res)
		}
	}

	evm_test!{test_sha3: test_sha3_jit, test_sha3_int}
	fn test_sha3(factory: Factory) {
		let code = "6064640fffffffff20600055".from_hex().unwrap();

		let sender = Address::from_str("0f572e5295c57f15886f9b263e2f6d2d6c7b5ec6").unwrap();
		let address = contract_address(&sender, &U256::zero());
		// TODO: add tests for 'callcreate'
		//let next_address = contract_address(&address, &U256::zero());
		let mut params = ActionParams::default();
		params.address = address.clone();
		params.sender = sender.clone();
		params.origin = sender.clone();
		params.gas = U256::from(0x0186a0);
		params.code = Some(Arc::new(code));
		params.value = ActionValue::Transfer(U256::from_str("0de0b6b3a7640000").unwrap());
		let mut state_result = get_temp_state();
		let mut state = state_result.reference_mut();
		state.add_balance(&sender, &U256::from_str("152d02c7e14af6800000").unwrap(), CleanupMode::NoEmpty).unwrap();
		let info = EnvInfo::default();
		let engine = TestEngine::new(0);
		let mut substate = Substate::new();

		let result = {
			let mut ex = Executive::new(&mut state, &info, &engine, &factory);
			ex.create(params, &mut substate, &mut NoopTracer, &mut NoopVMTracer)
		};

		match result {
			Err(_) => {
			},
			_ => {
				panic!("Expected OutOfGas");
			}
		}
	}
}
