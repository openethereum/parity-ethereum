// Copyright 2015, 2016 Ethcore (UK) Ltd.
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

fn do_json_test(json_data: &[u8]) -> Vec<String> {
	let tests = ethjson::transaction::Test::load(json_data).unwrap();
	let mut failed = Vec::new();
	let old_schedule = evm::Schedule::new_frontier();
	let new_schedule = evm::Schedule::new_homestead();
	for (name, test) in tests.into_iter() {
		let mut fail = false;
		let mut fail_unless = |cond: bool| if !cond && !fail { failed.push(name.clone()); println!("Transaction failed: {:?}", name); fail = true };

		let number: Option<u64> = test.block_number.map(Into::into);
		let schedule = match number {
			None => &old_schedule,
			Some(x) if x < 1_150_000 => &old_schedule,
			Some(_) => &new_schedule
		};

		let rlp: Vec<u8> = test.rlp.into();
		let res = UntrustedRlp::new(&rlp)
			.as_val()
			.map_err(From::from)
			.and_then(|t: SignedTransaction| t.validate(schedule, schedule.have_delegate_call));

		fail_unless(test.transaction.is_none() == res.is_err());
		if let (Some(tx), Some(sender)) = (test.transaction, test.sender) {
			let t = res.unwrap();
			fail_unless(t.sender().unwrap() == sender.into());
			let data: Vec<u8> = tx.data.into();
			fail_unless(t.data == data);
			fail_unless(t.gas_price == tx.gas_price.into());
			fail_unless(t.nonce == tx.nonce.into());
			fail_unless(t.value == tx.value.into());
			let to: Option<_> = tx.to.into();
			let to: Option<Address> = to.map(Into::into);
			match t.action {
				Action::Call(dest) => fail_unless(Some(dest) == to),
				Action::Create => fail_unless(None == to),
			}
		}
	}

	for f in &failed {
		println!("FAILED: {:?}", f);
	}
	failed
}

declare_test!{TransactionTests_ttTransactionTest, "TransactionTests/ttTransactionTest"}
declare_test!{heavy => TransactionTests_tt10mbDataField, "TransactionTests/tt10mbDataField"}
declare_test!{TransactionTests_ttWrongRLPTransaction, "TransactionTests/ttWrongRLPTransaction"}
declare_test!{TransactionTests_Homestead_ttTransactionTest, "TransactionTests/Homestead/ttTransactionTest"}
declare_test!{heavy => TransactionTests_Homestead_tt10mbDataField, "TransactionTests/Homestead/tt10mbDataField"}
declare_test!{TransactionTests_Homestead_ttWrongRLPTransaction, "TransactionTests/Homestead/ttWrongRLPTransaction"}
declare_test!{TransactionTests_RandomTests_tr201506052141PYTHON, "TransactionTests/RandomTests/tr201506052141PYTHON"}
