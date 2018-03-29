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

//! Snapshot manifest type definition

use ethereum_types::H256;
use rlp::{Rlp, RlpStream, DecoderError};
use bytes::Bytes;

/// Manifest data.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ManifestData {
	/// Snapshot format version.
	pub version: u64,
	/// List of state chunk hashes.
	pub state_hashes: Vec<H256>,
	/// List of block chunk hashes.
	pub block_hashes: Vec<H256>,
	/// The final, expected state root.
	pub state_root: H256,
	/// Block number this snapshot was taken at.
	pub block_number: u64,
	/// Block hash this snapshot was taken at.
	pub block_hash: H256,
}

impl ManifestData {
	/// Encode the manifest data to rlp.
	pub fn into_rlp(self) -> Bytes {
		let mut stream = RlpStream::new_list(6);
		stream.append(&self.version);
		stream.append_list(&self.state_hashes);
		stream.append_list(&self.block_hashes);
		stream.append(&self.state_root);
		stream.append(&self.block_number);
		stream.append(&self.block_hash);

		stream.out()
	}

	/// Try to restore manifest data from raw bytes, interpreted as RLP.
	pub fn from_rlp(raw: &[u8]) -> Result<Self, DecoderError> {
		let decoder = Rlp::new(raw);
		let (start, version) = if decoder.item_count()? == 5 {
			(0, 1)
		} else {
			(1, decoder.val_at(0)?)
		};

		let state_hashes: Vec<H256> = decoder.list_at(start + 0)?;
		let block_hashes: Vec<H256> = decoder.list_at(start + 1)?;
		let state_root: H256 = decoder.val_at(start + 2)?;
		let block_number: u64 = decoder.val_at(start + 3)?;
		let block_hash: H256 = decoder.val_at(start + 4)?;

		Ok(ManifestData {
			version: version,
			state_hashes: state_hashes,
			block_hashes: block_hashes,
			state_root: state_root,
			block_number: block_number,
			block_hash: block_hash,
		})
	}
}

