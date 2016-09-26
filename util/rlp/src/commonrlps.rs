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

//! Contains RLPs used for compression.

use rlpcompression::InvalidRlpSwapper;

lazy_static! {
	/// Swapper for snapshot compression.
	pub static ref SNAPSHOT_RLP_SWAPPER: InvalidRlpSwapper<'static> = InvalidRlpSwapper::new(EMPTY_RLPS, INVALID_RLPS);
}

lazy_static! {
	/// Swapper with common long RLPs, up to 127 can be added.
	pub static ref BLOCKS_RLP_SWAPPER: InvalidRlpSwapper<'static> = InvalidRlpSwapper::new(COMMON_RLPS, INVALID_RLPS);
}

static EMPTY_RLPS: &'static [&'static [u8]] = &[
	// RLP of SHA3_NULL_RLP
	&[160, 86, 232, 31, 23, 27, 204, 85, 166, 255, 131, 69, 230, 146, 192, 248, 110, 91, 72, 224, 27, 153, 108, 173, 192, 1, 98, 47, 181, 227, 99, 180, 33],
	// RLP of SHA3_EMPTY
	&[160, 197, 210, 70, 1, 134, 247, 35, 60, 146, 126, 125, 178, 220, 199, 3, 192, 229, 0, 182, 83, 202, 130, 39, 59, 123, 250, 216, 4, 93, 133, 164, 112]
];

static COMMON_RLPS: &'static [&'static [u8]] = &[
	// RLP of SHA3_NULL_RLP
	&[160, 86, 232, 31, 23, 27, 204, 85, 166, 255, 131, 69, 230, 146, 192, 248, 110, 91, 72, 224, 27, 153, 108, 173, 192, 1, 98, 47, 181, 227, 99, 180, 33],
	// RLP of SHA3_EMPTY
	&[160, 197, 210, 70, 1, 134, 247, 35, 60, 146, 126, 125, 178, 220, 199, 3, 192, 229, 0, 182, 83, 202, 130, 39, 59, 123, 250, 216, 4, 93, 133, 164, 112],
	// Other RLPs found in blocks DB using the test below.
	&[160, 29, 204, 77, 232, 222, 199, 93, 122, 171, 133, 181, 103, 182, 204, 212, 26, 211, 18, 69, 27, 148, 138, 116, 19, 240, 161, 66, 253, 64, 212, 147, 71],
	&[148, 50, 190, 52, 59, 148, 248, 96, 18, 77, 196, 254, 226, 120, 253, 203, 211, 140, 16, 45, 136],
	&[148, 82, 188, 68, 213, 55, 131, 9, 238, 42, 191, 21, 57, 191, 113, 222, 27, 125, 123, 227, 181],
	&[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]
];

static INVALID_RLPS: &'static [&'static [u8]] = &[&[0x81, 0x0], &[0x81, 0x1], &[0x81, 0x2], &[0x81, 0x3], &[0x81, 0x4], &[0x81, 0x5], &[0x81, 0x6], &[0x81, 0x7], &[0x81, 0x8], &[0x81, 0x9], &[0x81, 0xa], &[0x81, 0xb], &[0x81, 0xc], &[0x81, 0xd], &[0x81, 0xe], &[0x81, 0xf], &[0x81, 0x10], &[0x81, 0x11], &[0x81, 0x12], &[0x81, 0x13], &[0x81, 0x14], &[0x81, 0x15], &[0x81, 0x16], &[0x81, 0x17], &[0x81, 0x18], &[0x81, 0x19], &[0x81, 0x1a], &[0x81, 0x1b], &[0x81, 0x1c], &[0x81, 0x1d], &[0x81, 0x1e], &[0x81, 0x1f], &[0x81, 0x20], &[0x81, 0x21], &[0x81, 0x22], &[0x81, 0x23], &[0x81, 0x24], &[0x81, 0x25], &[0x81, 0x26], &[0x81, 0x27], &[0x81, 0x28], &[0x81, 0x29], &[0x81, 0x2a], &[0x81, 0x2b], &[0x81, 0x2c], &[0x81, 0x2d], &[0x81, 0x2e], &[0x81, 0x2f], &[0x81, 0x30], &[0x81, 0x31], &[0x81, 0x32], &[0x81, 0x33], &[0x81, 0x34], &[0x81, 0x35], &[0x81, 0x36], &[0x81, 0x37], &[0x81, 0x38], &[0x81, 0x39], &[0x81, 0x3a], &[0x81, 0x3b], &[0x81, 0x3c], &[0x81, 0x3d], &[0x81, 0x3e], &[0x81, 0x3f], &[0x81, 0x40], &[0x81, 0x41], &[0x81, 0x42], &[0x81, 0x43], &[0x81, 0x44], &[0x81, 0x45], &[0x81, 0x46], &[0x81, 0x47], &[0x81, 0x48], &[0x81, 0x49], &[0x81, 0x4a], &[0x81, 0x4b], &[0x81, 0x4c], &[0x81, 0x4d], &[0x81, 0x4e], &[0x81, 0x4f], &[0x81, 0x50], &[0x81, 0x51], &[0x81, 0x52], &[0x81, 0x53], &[0x81, 0x54], &[0x81, 0x55], &[0x81, 0x56], &[0x81, 0x57], &[0x81, 0x58], &[0x81, 0x59], &[0x81, 0x5a], &[0x81, 0x5b], &[0x81, 0x5c], &[0x81, 0x5d], &[0x81, 0x5e], &[0x81, 0x5f], &[0x81, 0x60], &[0x81, 0x61], &[0x81, 0x62], &[0x81, 0x63], &[0x81, 0x64], &[0x81, 0x65], &[0x81, 0x66], &[0x81, 0x67], &[0x81, 0x68], &[0x81, 0x69], &[0x81, 0x6a], &[0x81, 0x6b], &[0x81, 0x6c], &[0x81, 0x6d], &[0x81, 0x6e], &[0x81, 0x6f], &[0x81, 0x70], &[0x81, 0x71], &[0x81, 0x72], &[0x81, 0x73], &[0x81, 0x74], &[0x81, 0x75], &[0x81, 0x76], &[0x81, 0x77], &[0x81, 0x78], &[0x81, 0x79], &[0x81, 0x7a], &[0x81, 0x7b], &[0x81, 0x7c], &[0x81, 0x7d], &[0x81, 0x7e]];