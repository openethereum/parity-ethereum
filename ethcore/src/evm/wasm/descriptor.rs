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

//! Wasm evm call descriptor

use types::executed::CallType;
use util::{U256, H256, Uint};

pub struct CallDescriptor {
    // address of code executed
    // zero if it's a create
    pub address: [u8; 32],

    // sender of the transaction
    pub sender: [u8; 32],

    // transfer value
    pub value: [u8; 32],

    // reserved space / alignment to 256 bytes
    _reserved: [u8; 160],

    // call/create params
    pub data: Vec<u8>,
}

impl CallDescriptor {
    pub fn new(address: H256, sender: H256, value: U256, data: Vec<u8>) -> Self {
        let mut descriptor = CallDescriptor {
            address: [0u8; 32],
            sender: [0u8; 32],
            value: [0u8; 32],
            _reserved: [0u8; 160],
            data: data,
        };

        descriptor.address.copy_from_slice(&*address);
        descriptor.sender.copy_from_slice(&*sender);
        value.to_big_endian(&mut descriptor.value);

        descriptor
    }

    pub fn len(&self) -> u32 {
        self.data.len() as u32 + 256
    }
}