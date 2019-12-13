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

//! Tests of EVM integration with transaction execution.

use std::sync::Arc;
use hash::keccak;
use vm::{EnvInfo, ActionParams, ActionValue, ActionType, ParamsType};
use evm::Factory;
use machine::{
	executive::Executive,
	substate::Substate,
	test_helpers::new_eip210_test_machine,
};
use test_helpers::get_temp_state_with_factory;
use trace::{NoopVMTracer, NoopTracer};
use types::transaction::SYSTEM_ADDRESS;

use rustc_hex::FromHex;

use ethereum_types::{H256, Address};

evm_test!{test_blockhash_eip210: test_blockhash_eip210_int}
fn test_blockhash_eip210(factory: Factory) {
	let get_prev_hash_code = Arc::new("600143034060205260206020f3".from_hex::<Vec<_>>().unwrap()); // this returns previous block hash
	let get_prev_hash_code_hash = keccak(get_prev_hash_code.as_ref());
	// This is same as DEFAULT_BLOCKHASH_CONTRACT except for metropolis transition block check removed.
	let test_blockhash_contract = "73fffffffffffffffffffffffffffffffffffffffe33141561007a57600143036020526000356101006020510755600061010060205107141561005057600035610100610100602051050761010001555b6000620100006020510714156100755760003561010062010000602051050761020001555b61014a565b4360003512151561009057600060405260206040f35b610100600035430312156100b357610100600035075460605260206060f3610149565b62010000600035430312156100d157600061010060003507146100d4565b60005b156100f6576101006101006000350507610100015460805260206080f3610148565b630100000060003543031215610116576000620100006000350714610119565b60005b1561013c57610100620100006000350507610200015460a052602060a0f3610147565b600060c052602060c0f35b5b5b5b5b";
	let blockhash_contract_code = Arc::new(test_blockhash_contract.from_hex::<Vec<_>>().unwrap());
	let blockhash_contract_code_hash = keccak(blockhash_contract_code.as_ref());
	let machine = new_eip210_test_machine();
	let mut env_info = EnvInfo::default();

	// populate state with 256 last hashes
	let mut state = get_temp_state_with_factory(factory);
	let contract_address = Address::from_low_u64_be(0xf0);
	state.init_code(&contract_address, (*blockhash_contract_code).clone()).unwrap();
	for i in 1 .. 257 {
		env_info.number = i.into();
		let params = ActionParams {
			code_address: contract_address.clone(),
			address: contract_address,
			sender: SYSTEM_ADDRESS.clone(),
			origin: SYSTEM_ADDRESS.clone(),
			gas: 100000.into(),
			gas_price: 0.into(),
			value: ActionValue::Transfer(0.into()),
			code: Some(blockhash_contract_code.clone()),
			code_hash: Some(blockhash_contract_code_hash),
			code_version: 0.into(),
			data: Some(H256::from_low_u64_be(i - 1).as_bytes().to_vec()),
			action_type: ActionType::Call,
			params_type: ParamsType::Separate,
		};
		let schedule = machine.schedule(env_info.number);
		let mut ex = Executive::new(&mut state, &env_info, &machine, &schedule);
		let mut substate = Substate::new();
		if let Err(e) = ex.call(params, &mut substate, &mut NoopTracer, &mut NoopVMTracer) {
			panic!("Encountered error on updating last hashes: {}", e);
		}
	}

	env_info.number = 256;
	let params = ActionParams {
		code_address: Address::zero(),
		address: Address::zero(),
		sender: Address::zero(),
		origin: Address::zero(),
		gas: 100000.into(),
		gas_price: 0.into(),
		value: ActionValue::Transfer(0.into()),
		code: Some(get_prev_hash_code),
		code_hash: Some(get_prev_hash_code_hash),
		code_version: 0.into(),
		data: None,
		action_type: ActionType::Call,
		params_type: ParamsType::Separate,
	};
	let schedule = machine.schedule(env_info.number);
	let mut ex = Executive::new(&mut state, &env_info, &machine, &schedule);
	let mut substate = Substate::new();
	let res = ex.call(params, &mut substate, &mut NoopTracer, &mut NoopVMTracer);
	let output = match res {
		Ok(res) => H256::from_slice(&res.return_data[..32]),
		Err(e) => {
			panic!("Encountered error on getting last hash: {}", e);
		},
	};
	assert_eq!(output, H256::from_low_u64_be(255));
}
