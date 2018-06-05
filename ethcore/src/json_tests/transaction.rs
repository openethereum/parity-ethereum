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

use super::test_common::*;
use evm;
use ethjson;
use rlp::Rlp;
use transaction::{Action, UnverifiedTransaction, SignedTransaction};

fn do_json_test(json_data: &[u8]) -> Vec<String> {
	let tests = ethjson::transaction::Test::load(json_data).unwrap();
	let mut failed = Vec::new();
	let frontier_schedule = evm::Schedule::new_frontier();
	let homestead_schedule = evm::Schedule::new_homestead();
	let byzantium_schedule = evm::Schedule::new_byzantium();
	for (name, test) in tests.into_iter() {
		let mut fail_unless = |cond: bool, title: &str| if !cond { failed.push(name.clone()); println!("Transaction failed: {:?}: {:?}", name, title); };

		let number: Option<u64> = test.block_number.map(Into::into);
		let schedule = match number {
			None => &frontier_schedule,
			Some(x) if x < 1_150_000 => &frontier_schedule,
			Some(x) if x < 3_000_000 => &homestead_schedule,
			Some(_) => &byzantium_schedule
		};
		let allow_chain_id_of_one = number.map_or(false, |n| n >= 2_675_000);
		let allow_unsigned = number.map_or(false, |n| n >= 3_000_000);

		let rlp: Vec<u8> = test.rlp.into();
		let res = Rlp::new(&rlp)
			.as_val()
			.map_err(::error::Error::from)
			.and_then(|t: UnverifiedTransaction| {
				t.validate(schedule, schedule.have_delegate_call, allow_chain_id_of_one, allow_unsigned).map_err(Into::into)
			});

		fail_unless(test.transaction.is_none() == res.is_err(), "Validity different");
		if let (Some(tx), Some(sender)) = (test.transaction, test.sender) {
			let t = res.unwrap();
			fail_unless(SignedTransaction::new(t.clone()).unwrap().sender() == sender.into(), "sender mismatch");
			let is_acceptable_chain_id = match t.chain_id() {
				None => true,
				Some(1) if allow_chain_id_of_one => true,
				_ => false,
			};
			fail_unless(is_acceptable_chain_id, "Network ID unacceptable");
			let data: Vec<u8> = tx.data.into();
			fail_unless(t.data == data, "data mismatch");
			fail_unless(t.gas_price == tx.gas_price.into(), "gas_price mismatch");
			fail_unless(t.nonce == tx.nonce.into(), "nonce mismatch");
			fail_unless(t.value == tx.value.into(), "value mismatch");
			let to: Option<ethjson::hash::Address> = tx.to.into();
			let to: Option<Address> = to.map(Into::into);
			match t.action {
				Action::Call(dest) => fail_unless(Some(dest) == to, "call/destination mismatch"),
				Action::Create => fail_unless(None == to, "create mismatch"),
			}
		}
	}

	for f in &failed {
		println!("FAILED: {:?}", f);
	}
	failed
}

declare_test!{TransactionTests_ttEip155VitaliksHomesead, "TransactionTests/ttEip155VitaliksHomesead"}
declare_test!{TransactionTests_ttEip155VitaliksEip158, "TransactionTests/ttEip155VitaliksEip158"}
declare_test!{TransactionTests_ttEip158, "TransactionTests/ttEip158"}
declare_test!{TransactionTests_ttFrontier, "TransactionTests/ttFrontier"}
declare_test!{TransactionTests_ttHomestead, "TransactionTests/ttHomestead"}
declare_test!{TransactionTests_ttVRuleEip158, "TransactionTests/ttVRuleEip158"}
declare_test!{TransactionTests_ttWrongRLPFrontier, "TransactionTests/ttWrongRLPFrontier"}
declare_test!{TransactionTests_ttWrongRLPHomestead, "TransactionTests/ttWrongRLPHomestead"}
declare_test!{TransactionTests_ttConstantinople, "TransactionTests/ttConstantinople"}
declare_test!{TransactionTests_ttSpecConstantinople, "TransactionTests/ttSpecConstantinople"}
