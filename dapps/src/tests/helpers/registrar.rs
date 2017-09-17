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

use std::str;
use std::sync::Arc;
use std::collections::HashMap;
use rustc_hex::FromHex;

use hash_fetch::urlhint::ContractClient;
use bigint::hash::H256;
use util::Address;
use bytes::{Bytes, ToPretty};
use parking_lot::Mutex;

const REGISTRAR: &'static str = "8e4e9b13d4b45cb0befc93c3061b1408f67316b2";
const URLHINT: &'static str = "deadbeefcafe0000000000000000000000000000";
const URLHINT_RESOLVE: &'static str = "267b6922";
const DEFAULT_HASH: &'static str = "1472a9e190620cdf6b31f383373e45efcfe869a820c91f9ccd7eb9fb45e4985d";

pub struct FakeRegistrar {
	pub calls: Arc<Mutex<Vec<(String, String)>>>,
	pub responses: Mutex<HashMap<(String, String), Result<Bytes, String>>>,
}

impl FakeRegistrar {
	pub fn new() -> Self {
		FakeRegistrar {
			calls: Arc::new(Mutex::new(Vec::new())),
			responses: Mutex::new({
				let mut map = HashMap::new();
				map.insert(
					(REGISTRAR.into(), "6795dbcd058740ee9a5a3fb9f1cfa10752baec87e09cc45cd7027fd54708271aca300c75000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000000014100000000000000000000000000000000000000000000000000000000000000".into()),
					Ok(format!("000000000000000000000000{}", URLHINT).from_hex().unwrap()),
				);
				map.insert(
					(URLHINT.into(), format!("{}{}", URLHINT_RESOLVE, DEFAULT_HASH)),
					Ok(vec![])
				);
				map
			}),
		}
	}

	pub fn set_result(&self, hash: H256, result: Result<Bytes, String>) {
		self.responses.lock().insert(
			(URLHINT.into(), format!("{}{:?}", URLHINT_RESOLVE, hash)),
			result
		);
	}
}

impl ContractClient for FakeRegistrar {
	fn registrar(&self) -> Result<Address, String> {
		Ok(REGISTRAR.parse().unwrap())
	}

	fn call(&self, address: Address, data: Bytes) -> ::futures::BoxFuture<Bytes, String> {
		let call = (address.to_hex(), data.to_hex());
		self.calls.lock().push(call.clone());
		let res = self.responses.lock().get(&call).cloned().expect(&format!("No response for call: {:?}", call));
		Box::new(::futures::future::done(res))
	}
}
