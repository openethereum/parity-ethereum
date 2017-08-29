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

//! Wasm evm call arguments helper

use util::{U256, H160};

/// Input part of the wasm call descriptor
pub struct CallArgs {
	/// Receiver of the transaction
	pub address: [u8; 20],

	/// Sender of the transaction
	pub sender: [u8; 20],

	/// Original transaction initiator
	pub origin: [u8; 20],

	/// Transfer value
	pub value: [u8; 32],

	/// call/create params
	pub data: Vec<u8>,
}

impl CallArgs {
	/// New contract call payload with known parameters
	pub fn new(address: H160, sender: H160, origin: H160, value: U256, data: Vec<u8>) -> Self {
		let mut descriptor = CallArgs {
			address: [0u8; 20],
			sender: [0u8; 20],
			origin: [0u8; 20],
			value: [0u8; 32],
			data: data,
		};

		descriptor.address.copy_from_slice(&*address);
		descriptor.sender.copy_from_slice(&*sender);
		descriptor.origin.copy_from_slice(&*origin);
		value.to_big_endian(&mut descriptor.value);

		descriptor
	}

	/// Total call payload length in linear memory
	pub fn len(&self) -> u32 {
		self.data.len() as u32 + 92
	}
}