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

use ethcore::transaction::{Transaction, SignedTransaction, Action};
use bigint::prelude::U256;

use jsonrpc_core::Error;
use v1::helpers::CallRequest;

pub fn sign_call(request: CallRequest, gas_cap: bool) -> Result<SignedTransaction, Error> {
	let max_gas = 50_000_000.into();
	let gas = match request.gas {
		Some(gas) if gas_cap && gas > max_gas => {
			warn!("Gas limit capped to {} (from {})", max_gas, gas);
			max_gas
		}
		Some(gas) => gas,
		None if gas_cap => max_gas,
		None => U256::from(2) << 50,
	};
	let from = request.from.unwrap_or(0.into());

	Ok(Transaction {
		nonce: request.nonce.unwrap_or_else(|| 0.into()),
		action: request.to.map_or(Action::Create, Action::Call),
		gas,
		gas_price: request.gas_price.unwrap_or_else(|| 0.into()),
		value: request.value.unwrap_or(0.into()),
		data: request.data.unwrap_or_default(),
	}.fake_sign(from))
}
