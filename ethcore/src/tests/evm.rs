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
use tests::helpers::get_temp_state_with_factory;
use trace::{NoopVMTracer, NoopTracer};
use transaction::SYSTEM_ADDRESS;

use rustc_hex::FromHex;

use ethereum_types::{H256, Address};
use bytes::BytesRef;

evm_test!{test_blockhash_eip210: test_blockhash_eip210_int}
fn test_blockhash_eip210(factory: Factory) {
	let get_prev_hash_code = Arc::new("600143034060205260206020f3".from_hex().unwrap()); // this returns previous block hash
	let get_prev_hash_code_hash = keccak(get_prev_hash_code.as_ref());
	// This is same as DEFAULT_BLOCKHASH_CONTRACT except for metropolis transition block check removed.
	let test_blockhash_contract = "73fffffffffffffffffffffffffffffffffffffffe33141561007a57600143036020526000356101006020510755600061010060205107141561005057600035610100610100602051050761010001555b6000620100006020510714156100755760003561010062010000602051050761020001555b61014a565b4360003512151561009057600060405260206040f35b610100600035430312156100b357610100600035075460605260206060f3610149565b62010000600035430312156100d157600061010060003507146100d4565b60005b156100f6576101006101006000350507610100015460805260206080f3610148565b630100000060003543031215610116576000620100006000350714610119565b60005b1561013c57610100620100006000350507610200015460a052602060a0f3610147565b600060c052602060c0f35b5b5b5b5b";
	let blockhash_contract_code = Arc::new(test_blockhash_contract.from_hex().unwrap());
	let blockhash_contract_code_hash = keccak(blockhash_contract_code.as_ref());
	let machine = ::ethereum::new_constantinople_test_machine();
	let mut env_info = EnvInfo::default();

	// populate state with 256 last hashes
	let mut state = get_temp_state_with_factory(factory);
	let contract_address: Address = 0xf0.into();
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
			data: Some(H256::from(i - 1).to_vec()),
			call_type: CallType::Call,
			params_type: ParamsType::Separate,
		};
		let mut ex = Executive::new(&mut state, &env_info, &machine);
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
		gas: 100000.into(),
		gas_price: 0.into(),
		value: ActionValue::Transfer(0.into()),
		code: Some(get_prev_hash_code),
		code_hash: Some(get_prev_hash_code_hash),
		data: None,
		call_type: CallType::Call,
		params_type: ParamsType::Separate,
	};
	let mut ex = Executive::new(&mut state, &env_info, &machine);
	let mut substate = Substate::new();
	let mut output = H256::new();
	if let Err(e) = ex.call(params, &mut substate, BytesRef::Fixed(&mut output), &mut NoopTracer, &mut NoopVMTracer) {
		panic!("Encountered error on getting last hash: {}", e);
	}
	assert_eq!(output, 255.into());
}
