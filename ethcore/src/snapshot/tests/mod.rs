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

//! Snapshot tests.

mod blocks;
mod state;
mod service;

pub mod helpers;

use super::ManifestData;

#[test]
fn manifest_rlp() {
	let manifest = ManifestData {
		block_hashes: Vec::new(),
		state_hashes: Vec::new(),
		block_number: 1234567,
		state_root: Default::default(),
		block_hash: Default::default(),
	};
	let raw = manifest.clone().into_rlp();
	assert_eq!(ManifestData::from_rlp(&raw).unwrap(), manifest);
}