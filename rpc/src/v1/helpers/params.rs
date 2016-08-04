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

use jsonrpc_core::{Error, Params};
use v1::helpers::errors;

pub fn expect_no_params(params: Params) -> Result<(), Error> {
	match params {
		Params::None => Ok(()),
		p => Err(errors::invalid_params("No parameters were expected", p)),
	}
}
