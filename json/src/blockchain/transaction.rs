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

//! Blockchain test transaction deserialization.

use uint::Uint;
use bytes::Bytes;

/// Blockchain test transaction deserialization.
#[derive(Debug, PartialEq, Deserialize)]
pub struct Transaction {
	data: Bytes,
	#[serde(rename="gasLimit")]
	gas_limit: Uint,
	#[serde(rename="gasPrice")]
	gas_price: Uint,
	nonce: Uint,
	r: Uint,
	s: Uint,
	v: Uint,
	value: Uint
}
