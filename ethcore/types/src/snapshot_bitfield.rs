// Copyright 2015-2018 Parity Technologies (UK) Ltd.
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

use super::snapshot_manifest::ManifestData;
use bytes::Bytes;
use ethereum_types::H256;
use rlp::{Rlp, RlpStream, DecoderError};

use std::collections::HashSet;
use std::iter::FromIterator;

#[derive(Clone)]
struct BitfieldCompletion {
	/// Raw bits of completion (indexed hash, 1 if completed, 0 otherwise)
	bytes: Vec<u8>,
	/// Number of chunks available
	num_available: usize,
}

impl BitfieldCompletion {
	pub fn new() -> BitfieldCompletion {
		BitfieldCompletion {
			bytes: Vec::new(),
			num_available: 0,
		}
	}

	pub fn new_from_bytes(bytes: &Vec<u8>, length: usize) -> BitfieldCompletion {
		let mut num_available = 0;

		// Count the number of pieces available
		for index in 0..length {
			let byte_index = index / 8;
			let bit_index = index % 8;

			if bytes[byte_index] & (1 << (7 - bit_index)) != 0 {
				num_available += 1;
			}
		}

		BitfieldCompletion {
			bytes: bytes.clone(),
			num_available,
		}
	}

	pub fn reset(&mut self, length: usize) {
		self.bytes = vec![0; length];
		self.num_available = 0;
	}

	pub fn is_available(&self, index: usize) -> bool {
		let byte_index = index / 8;
		let bit_index = index % 8;

		self.bytes[byte_index] & (1 << (7 - bit_index)) != 0
	}

	/// Set the given hash at the given index as completed
	pub fn mark(&mut self, index: usize) {
		let byte_index = index / 8;
		let bit_index = index % 8;
		let mask = 1 << (7 - bit_index);

		// Update `bytes` and `completed chunks`
		self.bytes[byte_index] |= mask;
		self.num_available += 1;
	}

	pub fn bytes(&self) -> Vec<u8> {
		self.bytes.clone()
	}

	pub fn num_available(&self) -> usize {
		self.num_available
	}

	pub fn len(&self) -> usize {
		self.bytes.len()
	}
}

#[derive(Clone)]
pub struct Bitfield {
	completion: BitfieldCompletion,
	hashes: Vec<H256>,
}

impl Bitfield {
	pub fn new() -> Bitfield {
		Bitfield {
			completion: BitfieldCompletion::new(),
			hashes: Vec::new(),
		}
	}

	pub fn new_from_manifest(manifest: &ManifestData) -> Bitfield {
		let mut bitfield = Bitfield::new();
		bitfield.reset_to(manifest);

		bitfield
	}

	pub fn new_from_bytes(manifest: &ManifestData, bytes: &Vec<u8>) -> Bitfield {
		let mut bitfield = Bitfield::new();
		bitfield.reset_to(manifest);
		bitfield.completion = BitfieldCompletion::new_from_bytes(bytes, bitfield.len());

		bitfield
	}

	/// Encode the manifest bitfield to rlp.
	pub fn into_rlp(self) -> Bytes {
		let mut stream = RlpStream::new_list(1);
		stream.append_list(&self.completion.bytes());
		stream.out()
	}

	/// Try to restore bitfield data from raw bytes, interpreted as RLP.
	pub fn from_rlp(raw: &[u8], manifest: &ManifestData) -> Result<Self, DecoderError> {
		let decoder = Rlp::new(raw);
		let raw_bytes: Vec<u8> = decoder.list_at(0)?;

		Ok(Bitfield::new_from_bytes(manifest, &raw_bytes))
	}

	pub fn available_chunks(&self) -> Vec<H256> {
		self.hashes.iter().enumerate()
			.filter(|(i, _)| self.completion.is_available(*i))
			.map(|(_, h)| *h)
			.collect()
	}

	pub fn needed_chunks(&self) -> Vec<H256> {
		self.hashes.iter().enumerate()
			.filter(|(i, _)| !self.completion.is_available(*i))
			.map(|(_, h)| *h)
			.collect()
	}

	/// Returns the length of the bitfield
	pub fn len(&self) -> usize {
		self.completion.len()
	}

	pub fn num_available(&self) -> usize {
		self.completion.num_available()
	}

	/// Reset the current Bitfield
	pub fn reset(&mut self) {
		let length = self.completion.len();
		self.completion.reset(length);
	}

	pub fn reset_to(&mut self, manifest: &ManifestData) {
		self.hashes = manifest.block_hashes
			.iter()
			.chain(manifest.state_hashes.iter())
			.map(|h| *h)
			.collect::<Vec<H256>>();

		let length = (self.hashes.len() as f64 / 8 as f64).ceil() as usize;
		self.completion.reset(length);
	}

	/// Mark some hashes as completed
	pub fn mark_some(&mut self, completed_hashes: &Vec<H256>) {
		let iter = completed_hashes.iter().map(|&h| h).clone();
		let completed_hashes: HashSet<H256> = HashSet::from_iter(iter);

		for (index, hash) in self.hashes.iter().enumerate() {
			if completed_hashes.contains(&hash) {
				self.completion.mark(index);
			}
		}
	}

	/// Mark one hash as completed
	pub fn mark_one(&mut self, hash: &H256) {
		// Find the index of the completed hash
		if let Some(index) = self.hashes.iter().position(|&h| h == *hash) {
			self.completion.mark(index);
		}
	}

	/// Mark all chunks as available
	pub fn mark_all(&mut self) {
		for (index, _) in self.hashes.iter().enumerate() {
			self.completion.mark(index);
		}
	}

	pub fn as_raw(&self) -> Vec<u8> {
		self.completion.bytes()
	}
}
