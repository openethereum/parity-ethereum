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

use std::path::Path;
use super::test_common::*;
use ethjson;
use rlp::Rlp;
use transaction::{UnverifiedTransaction, SignedTransaction};
use client::{EvmTestClient};

/// Run transaction jsontests on a given folder.
pub fn run_test_path<H: FnMut(&str, HookType)>(p: &Path, skip: &[&'static str], h: &mut H) {
	::json_tests::test_common::run_test_path(p, skip, do_json_test, h)
}

/// Run transaction jsontests on a given file.
pub fn run_test_file<H: FnMut(&str, HookType)>(p: &Path, h: &mut H) {
	::json_tests::test_common::run_test_file(p, do_json_test, h)
}

fn do_json_test<H: FnMut(&str, HookType)>(json_data: &[u8], start_stop_hook: &mut H) -> Vec<String> {
	let tests = ethjson::transaction::Test::load(json_data).unwrap();
	let mut failed = Vec::new();
	for (name, test) in tests.into_iter() {
		start_stop_hook(&name, HookType::OnStart);

		let rlp: Vec<u8> = test.rlp.clone().into();

		for (spec_name, infos) in test.infos {
			let spec = match EvmTestClient::spec_from_json(&spec_name) {
				Some(spec) => spec,
				None => {
					println!("	 - {} | {:?} Ignoring tests because of missing spec", name, spec_name);
					continue;
				}
			};

			// using block 0 is safe with test conf (by convention enable feature are 0 and disable are Max)
			let block_number = 0;

			// using block 0 is safe with test conf (by convention enable feature are 0 and disable are Max
			let schedule = spec.params().schedule(block_number);

			let info = format!("{} | {:?}  ...", name, spec_name);

			let mut fail_unless = |cond: bool, title: &str| if !cond { failed.push(name.clone()); println!("Transaction failed: {:?}: {:?}", info, title); };

			let allow_chain_id_of_one = block_number >= spec.params().eip160_transition;
			let allow_unsigned = block_number >= spec.params().eip160_transition;

			let res = Rlp::new(&rlp)
				.as_val()
				.map_err(::error::Error::from)
				.and_then(|t: UnverifiedTransaction| {
					t.validate(&schedule, schedule.have_delegate_call, allow_chain_id_of_one, allow_unsigned).map_err(Into::into)
			});

			fail_unless(infos.hash.is_none() == res.is_err(), "Validity different");
			if let (Some(hash), Some(sender)) = (infos.hash, infos.sender) {
				let t = res.unwrap();
				fail_unless(t.hash() == hash, "Transaction hash mismatch");
				fail_unless(SignedTransaction::new(t.clone()).unwrap().sender() == sender.into(), "sender mismatch");
				let is_acceptable_chain_id = match t.chain_id() {
					None => true,
					Some(1) if allow_chain_id_of_one => true,
					_ => false,
				};
				fail_unless(is_acceptable_chain_id, "Network ID unacceptable");
			}

		}
		start_stop_hook(&name, HookType::OnStop);
	}

	for f in &failed {
		println!("FAILED: {:?}", f);
	}
	failed
}

declare_test!{TransactionTests_ttAddress, "TransactionTests/ttAddress"}
declare_test!{TransactionTests_ttData, "TransactionTests/ttData"}
declare_test!{TransactionTests_ttGasLimit, "TransactionTests/ttGasLimit"}
declare_test!{TransactionTests_ttGasPrice, "TransactionTests/ttGasPrice"}
declare_test!{TransactionTests_ttNonce, "TransactionTests/ttNonce"}
declare_test!{TransactionTests_ttRSValue, "TransactionTests/ttRSValue"}
declare_test!{TransactionTests_ttSignature, "TransactionTests/ttSignature"}
declare_test!{TransactionTests_ttValue, "TransactionTests/ttValue"}
declare_test!{TransactionTests_ttVValue, "TransactionTests/ttVValue"}
declare_test!{TransactionTests_ttWrongRLP, "TransactionTests/ttWrongRLP"}
