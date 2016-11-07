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

//! Tendermint specific parameters.

use ethjson;
use super::timeout::DefaultTimeouts;
use util::{Address, U256};

/// `Tendermint` params.
#[derive(Debug, Clone)]
pub struct TendermintParams {
	/// Gas limit divisor.
	pub gas_limit_bound_divisor: U256,
	/// List of authorities.
	pub authorities: Vec<Address>,
	/// Number of authorities.
	pub authority_n: usize,
	/// Timeout durations for different steps.
	pub timeouts: DefaultTimeouts,
}

impl Default for TendermintParams {
	fn default() -> Self {
		let authorities = vec!["0x7d577a597b2742b498cb5cf0c26cdcd726d39e6e".into(), "0x82a978b3f5962a5b0957d9ee9eef472ee55b42f1".into()];
		let val_n = authorities.len();
		TendermintParams {
			gas_limit_bound_divisor: 0x0400.into(),
			authorities: authorities,
			authority_n: val_n,
			timeouts:  DefaultTimeouts::default()
		}
	}
}

impl From<ethjson::spec::TendermintParams> for TendermintParams {
	fn from(p: ethjson::spec::TendermintParams) -> Self {
		let val: Vec<_> = p.authorities.into_iter().map(Into::into).collect();
		let val_n = val.len();
		TendermintParams {
			gas_limit_bound_divisor: p.gas_limit_bound_divisor.into(),
			authorities: val,
			authority_n: val_n,
			timeouts: DefaultTimeouts::default()
		}
	}
}
