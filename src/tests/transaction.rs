use super::test_common::*;
use evm;

fn do_json_test(json_data: &[u8]) -> Vec<String> {
	let json = Json::from_str(::std::str::from_utf8(json_data).unwrap()).expect("Json is invalid");
	let mut failed = Vec::new();
	let old_schedule = evm::Schedule::new_frontier();
	let new_schedule = evm::Schedule::new_homestead();
	let ot = RefCell::new(Transaction::new());
	for (name, test) in json.as_object().unwrap() {
		let mut fail = false;
		let mut fail_unless = |cond: bool| if !cond && !fail { failed.push(name.to_string()); println!("Transaction: {:?}", ot.borrow()); fail = true };
		let schedule = match test.find("blocknumber")
			.and_then(|j| j.as_string())
			.and_then(|s| BlockNumber::from_str(s).ok())
			.unwrap_or(0) { x if x < 900000 => &old_schedule, _ => &new_schedule };
		let rlp = Bytes::from_json(&test["rlp"]);
		let res = UntrustedRlp::new(&rlp).as_val().map_err(|e| From::from(e)).and_then(|t: Transaction| t.validate(schedule, schedule.have_delegate_call));
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
				*ot.borrow_mut() = t.clone();
				fail_unless(to == &xjson!(&tx["to"]));
			} else {
				*ot.borrow_mut() = t.clone();
				fail_unless(Bytes::from_json(&tx["to"]).len() == 0);
			}
		}
	}
	for f in failed.iter() {
		println!("FAILED: {:?}", f);
	}
	failed
}

// Once we have interpolate idents.
/*macro_rules! declare_test {
	($test_set_name: ident / $name: ident) => {
		#[test]
		#[allow(non_snake_case)]
		fn $name() {
			assert!(do_json_test(include_bytes!(concat!("../res/ethereum/tests/", stringify!($test_set_name), "/", stringify!($name), ".json"))).len() == 0);
		}
	};
	($test_set_name: ident / $prename: ident / $name: ident) => {
		#[test]
		#[allow(non_snake_case)]
		interpolate_idents! { fn [$prename _ $name]()
			{
				let json = include_bytes!(concat!("../res/ethereum/tests/", stringify!($test_set_name), "/", stringify!($prename), "/", stringify!($name), ".json"));
				assert!(do_json_test(json).len() == 0);
			}
		}
	};
}

declare_test!{TransactionTests/ttTransactionTest}
declare_test!{TransactionTests/tt10mbDataField}
declare_test!{TransactionTests/ttWrongRLPTransaction}
declare_test!{TransactionTests/Homestead/ttTransactionTest}
declare_test!{heavy => TransactionTests/Homestead/tt10mbDataField}
declare_test!{TransactionTests/Homestead/ttWrongRLPTransaction}
declare_test!{TransactionTests/RandomTests/tr201506052141PYTHON}*/

declare_test!{TransactionTests_ttTransactionTest, "TransactionTests/ttTransactionTest"}
declare_test!{TransactionTests_tt10mbDataField, "TransactionTests/tt10mbDataField"}
declare_test!{TransactionTests_ttWrongRLPTransaction, "TransactionTests/ttWrongRLPTransaction"}
declare_test!{TransactionTests_Homestead_ttTransactionTest, "TransactionTests/Homestead/ttTransactionTest"}
declare_test!{heavy => TransactionTests_Homestead_tt10mbDataField, "TransactionTests/Homestead/tt10mbDataField"}
declare_test!{TransactionTests_Homestead_ttWrongRLPTransaction, "TransactionTests/Homestead/ttWrongRLPTransaction"}
declare_test!{TransactionTests_RandomTests_tr201506052141PYTHON, "TransactionTests/RandomTests/tr201506052141PYTHON"}
