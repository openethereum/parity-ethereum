// Copyright 2015-2019 Parity Technologies (UK) Ltd.
// This file is part of Parity Ethereum.

// Parity Ethereum is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity Ethereum is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity Ethereum.  If not, see <http://www.gnu.org/licenses/>.

//! State machine types

use ethereum_types::Address;

use crate::receipt;

/// Type alias for a function we can make calls through synchronously.
/// Returns the call result and state proof for each call.
pub type Call<'a> = dyn Fn(Address, Vec<u8>) -> Result<(Vec<u8>, Vec<Vec<u8>>), String> + 'a;

/// Request for auxiliary data of a block.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AuxiliaryRequest {
	/// Needs the body.
	Body,
	/// Needs the receipts.
	Receipts,
	/// Needs both body and receipts.
	Both,
}

/// Auxiliary data fetcher for an Ethereum machine. In Ethereum-like machines
/// there are two kinds of auxiliary data: bodies and receipts.
#[derive(Default, Clone)]
pub struct AuxiliaryData<'a> {
	/// The full block bytes, including the header.
	pub bytes: Option<&'a [u8]>,
	/// The block receipts.
	pub receipts: Option<&'a [receipt::Receipt]>,
}
