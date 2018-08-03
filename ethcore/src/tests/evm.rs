// Copyright 2015-2018 Parity Technologies (UK) Ltd.
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

//! Tests of EVM integration with transaction execution.

use std::sync::Arc;
use hash::keccak;
use vm::{EnvInfo, ActionParams, ActionValue, CallType, ParamsType};
use evm::{Factory, VMType};
use executive::Executive;
use state::Substate;
use test_helpers::get_temp_state_with_factory;
use trace::{NoopVMTracer, NoopTracer};
use transaction::SYSTEM_ADDRESS;

use rustc_hex::FromHex;

use ethereum_types::{H256, Address};
use bytes::BytesRef;

evm_test!{test_blockhash_eip210: test_blockhash_eip210_int}
fn test_blockhash_eip210(factory: Factory) {
	let get_prev_hash_code = Arc::new("600143034060205260206020f3".from_hex().unwrap()); // this returns previous block hash
	let get_prev_hash_code_hash = keccak(get_prev_hash_code.as_ref());
	// This is same as DEFAULT_BLOCKHASH_CONTRACT
	let test_blockhash_contract = "600073fffffffffffffffffffffffffffffffffffffffe33141561005957600143035b60011561005357600035610100820683015561010081061561004057005b6101008104905061010082019150610022565b506100e0565b4360003512156100d4576000356001814303035b61010081121515610085576000610100830614610088565b60005b156100a75761010083019250610100820491506101008104905061006d565b610100811215156100bd57600060a052602060a0f35b610100820683015460c052602060c0f350506100df565b600060e052602060e0f35b5b50";
	let blockhash_contract_code = Arc::new(test_blockhash_contract.from_hex().unwrap());
	let blockhash_contract_code_hash = keccak(blockhash_contract_code.as_ref());
	let machine = ::ethereum::new_constantinople_test_machine();
	let mut env_info = EnvInfo::default();

	// populate state with 256 last hashes
	let mut state = get_temp_state_with_factory(factory);
	let contract_address: Address = 0xf0.into();
	state.init_code(&contract_address, (*blockhash_contract_code).clone()).unwrap();
	for i in 2 .. 257 {
		env_info.number = i.into();
		let params = ActionParams {
			code_address: contract_address.clone(),
			address: contract_address,
			sender: SYSTEM_ADDRESS.clone(),
			origin: SYSTEM_ADDRESS.clone(),
			gas: 1000000.into(),
			gas_price: 0.into(),
			value: ActionValue::Transfer(0.into()),
			code: Some(blockhash_contract_code.clone()),
			code_hash: Some(blockhash_contract_code_hash),
			data: Some(H256::from(i - 1).to_vec()),
			call_type: CallType::Call,
			params_type: ParamsType::Separate,
		};
		let schedule = machine.schedule(env_info.number);
		let mut ex = Executive::new(&mut state, &env_info, &machine, &schedule);
		let mut substate = Substate::new();
		let mut output = [];
		if let Err(e) = ex.call(params, &mut substate, BytesRef::Fixed(&mut output), &mut NoopTracer, &mut NoopVMTracer) {
			panic!("Encountered error on updating last hashes: {}", e);
		}
	}

	env_info.number = 256;
	let params = ActionParams {
		code_address: Address::new(),
		address: Address::new(),
		sender: Address::new(),
		origin: Address::new(),
		gas: 1000000.into(),
		gas_price: 0.into(),
		value: ActionValue::Transfer(0.into()),
		code: Some(get_prev_hash_code),
		code_hash: Some(get_prev_hash_code_hash),
		data: None,
		call_type: CallType::Call,
		params_type: ParamsType::Separate,
	};
	let schedule = machine.schedule(env_info.number);
	let mut ex = Executive::new(&mut state, &env_info, &machine, &schedule);
	let mut substate = Substate::new();
	let mut output = H256::new();
	if let Err(e) = ex.call(params, &mut substate, BytesRef::Fixed(&mut output), &mut NoopTracer, &mut NoopVMTracer) {
		panic!("Encountered error on getting last hash: {}", e);
	}
	assert_eq!(output, 255.into());
}
