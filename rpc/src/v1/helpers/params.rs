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

//! Parameters parsing helpers

use serde;
use jsonrpc_core::{Error, Params, from_params};
use v1::types::BlockNumber;
use v1::helpers::errors;

pub fn expect_no_params(params: Params) -> Result<(), Error> {
	match params {
		Params::None => Ok(()),
		p => Err(errors::invalid_params("No parameters were expected", p)),
	}
}

/// Returns number of different parameters in given `Params` object.
pub fn params_len(params: &Params) -> usize {
	match params {
		&Params::Array(ref vec) => vec.len(),
		_ => 0,
	}
}

/// Deserialize request parameters with optional second parameter `BlockNumber` defaulting to `BlockNumber::Latest`.
pub fn from_params_default_second<F>(params: Params) -> Result<(F, BlockNumber, ), Error> where F: serde::de::Deserialize {
	match params_len(&params) {
		1 => from_params::<(F, )>(params).map(|(f,)| (f, BlockNumber::Latest)),
		_ => from_params::<(F, BlockNumber)>(params),
	}
}

/// Deserialize request parameters with optional third parameter `BlockNumber` defaulting to `BlockNumber::Latest`.
pub fn from_params_default_third<F1, F2>(params: Params) -> Result<(F1, F2, BlockNumber, ), Error> where F1: serde::de::Deserialize, F2: serde::de::Deserialize {
	match params_len(&params) {
		2 => from_params::<(F1, F2, )>(params).map(|(f1, f2)| (f1, f2, BlockNumber::Latest)),
		_ => from_params::<(F1, F2, BlockNumber)>(params)
	}
}

