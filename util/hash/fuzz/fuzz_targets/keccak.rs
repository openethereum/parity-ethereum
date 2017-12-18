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

#![no_main]
#[macro_use] extern crate libfuzzer_sys;
extern crate keccak_hash;
use keccak_hash::{keccak, keccak_256, keccak_512};

fuzz_target!(|data: &[u8]| {
    keccak(data);
    unsafe {
        let mut data_m: Vec<u8> = Vec::with_capacity(data.len());
        data_m.extend_from_slice(data);
        keccak_256(data_m.as_mut_slice().as_mut_ptr(), data_m.len(), data.as_ptr(), data.len());
        keccak_512(data_m.as_mut_slice().as_mut_ptr(), data_m.len(), data.as_ptr(), data.len());
    }
});
