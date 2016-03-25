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

fn do_json_test(json_data: &[u8]) -> Vec<String> {
	let json = Json::from_str(::std::str::from_utf8(json_data).unwrap()).expect("Json is invalid");
	let mut failed = Vec::new();
	let old_schedule = evm::Schedule::new_frontier();
	let new_schedule = evm::Schedule::new_homestead();
	let ot = RefCell::new(None);
	for (name, test) in json.as_object().unwrap() {
		let mut fail = false;
		let mut fail_unless = |cond: bool| {
			if !cond && !fail {
				failed.push(name.clone());
				println!("Transaction: {:?}", ot.borrow());
				fail = true
			}
		};
		let schedule = match test.find("blocknumber")
		                         .and_then(|j| j.as_string())
		                         .and_then(|s| BlockNumber::from_str(s).ok())
		                         .unwrap_or(0) {
			x if x < 1_000_000 => &old_schedule,
			_ => &new_schedule,
		};
		let rlp = Bytes::from_json(&test["rlp"]);
		let res = UntrustedRlp::new(&rlp)
			.as_val()
			.map_err(From::from)
			.and_then(|t: SignedTransaction| t.validate(schedule, schedule.have_delegate_call));
		fail_unless(test.find("transaction").is_none() == res.is_err());
		if let (Some(&Json::Object(ref tx)), Some(&Json::String(ref expect_sender))) = (test.find("transaction"), test.find("sender")) {
			let t = res.unwrap();
			fail_unless(t.sender().unwrap() == address_from_hex(clean(expect_sender)));
			fail_unless(t.data == Bytes::from_json(&tx["data"]));
			fail_unless(t.gas == xjson!(&tx["gasLimit"]));
			fail_unless(t.gas_price == xjson!(&tx["gasPrice"]));
			fail_unless(t.nonce == xjson!(&tx["nonce"]));
			fail_unless(t.value == xjson!(&tx["value"]));
			if let Action::Call(ref to) = t.action {
				*ot.borrow_mut() = Some(t.clone());
				fail_unless(to == &xjson!(&tx["to"]));
			} else {
				*ot.borrow_mut() = Some(t.clone());
				fail_unless(Bytes::from_json(&tx["to"]).is_empty());
			}
		}
	}
	for f in &failed {
		println!("FAILED: {:?}", f);
	}
	failed
}

// Once we have interpolate idents.
// macro_rules! declare_test {
// ($test_set_name: ident / $name: ident) => {
// #[test]
// #[allow(non_snake_case)]
// fn $name() {
// assert!(do_json_test(include_bytes!(concat!("../res/ethereum/tests/", stringify!($test_set_name), "/", stringify!($name), ".json"))).len() == 0);
// }
// };
// ($test_set_name: ident / $prename: ident / $name: ident) => {
// #[test]
// #[allow(non_snake_case)]
// interpolate_idents! { fn [$prename _ $name]()
// {
// let json = include_bytes!(concat!("../res/ethereum/tests/", stringify!($test_set_name), "/", stringify!($prename), "/", stringify!($name), ".json"));
// assert!(do_json_test(json).len() == 0);
// }
// }
// };
// }
//
// declare_test!{TransactionTests/ttTransactionTest}
// declare_test!{TransactionTests/tt10mbDataField}
// declare_test!{TransactionTests/ttWrongRLPTransaction}
// declare_test!{TransactionTests/Homestead/ttTransactionTest}
// declare_test!{heavy => TransactionTests/Homestead/tt10mbDataField}
// declare_test!{TransactionTests/Homestead/ttWrongRLPTransaction}
// declare_test!{TransactionTests/RandomTests/tr201506052141PYTHON}


declare_test!{TransactionTests_ttTransactionTest, "TransactionTests/ttTransactionTest"}
declare_test!{heavy => TransactionTests_tt10mbDataField, "TransactionTests/tt10mbDataField"}
declare_test!{TransactionTests_ttWrongRLPTransaction, "TransactionTests/ttWrongRLPTransaction"}
declare_test!{TransactionTests_Homestead_ttTransactionTest, "TransactionTests/Homestead/ttTransactionTest"}
declare_test!{heavy => TransactionTests_Homestead_tt10mbDataField, "TransactionTests/Homestead/tt10mbDataField"}
declare_test!{TransactionTests_Homestead_ttWrongRLPTransaction, "TransactionTests/Homestead/ttWrongRLPTransaction"}
declare_test!{TransactionTests_RandomTests_tr201506052141PYTHON, "TransactionTests/RandomTests/tr201506052141PYTHON"}
