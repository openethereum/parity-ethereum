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

//! Web3 rpc implementation.
use jsonrpc_core::*;
use util::version;
use v1::traits::Web3;
use v1::types::{H256, Bytes};
use v1::helpers::params::expect_no_params;
use util::sha3::Hashable;

/// Web3 rpc implementation.
pub struct Web3Client;

impl Web3Client {
	/// Creates new Web3Client.
	pub fn new() -> Self { Web3Client }
}

impl Web3 for Web3Client {
	fn client_version(&self, params: Params) -> Result<Value, Error> {
		try!(expect_no_params(params));
		Ok(Value::String(version().to_owned().replace("Parity/", "Parity//")))
	}

	fn sha3(&self, params: Params) -> Result<Value, Error> {
		from_params::<(Bytes,)>(params).map(
			|(data,)| {
				let Bytes(ref vec) = data;
				let sha3 = vec.sha3();
				to_value(&H256::from(sha3))
			}
		)
	}
}
