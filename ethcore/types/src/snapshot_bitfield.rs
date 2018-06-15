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
	pub fn new(length: usize) -> BitfieldCompletion {
		BitfieldCompletion {
			bytes: vec![0; length],
			num_available: 0,
		}
	}

	pub fn is_available(index: usize, bytes: &Vec<u8>) -> bool {
		let byte_index = index / 8;
		let bit_index = index % 8;
		let mask = 1 << (7 - bit_index);

		bytes[byte_index] & mask != 0
	}

	/// Set the given hash at the given index as completed
	pub fn mark(&mut self, index: usize) {
		let byte_index = index / 8;
		let bit_index = index % 8;
		let mask = 1 << (7 - bit_index);

		// Update `bytes` and `completed chunks` only if not set yet
		if self.bytes[byte_index] & mask == 0 {
			self.bytes[byte_index] |= mask;
			self.num_available += 1;
		}
	}

	pub fn bytes(&self) -> Vec<u8> {
		self.bytes.clone()
	}

	pub fn num_available(&self) -> usize {
		self.num_available
	}
}

#[derive(Clone)]
pub struct Bitfield {
	completion: BitfieldCompletion,
	hashes: Vec<H256>,
}

impl Bitfield {
	pub fn new(manifest: &ManifestData) -> Bitfield {
		let (hashes, len) = Bitfield::read_manifest(manifest);

		Bitfield {
			hashes,
			completion: BitfieldCompletion::new(len),
		}
	}

	/// Encode the manifest bitfield to rlp.
	pub fn into_rlp(self) -> Bytes {
		let mut stream = RlpStream::new_list(1);
		stream.append_list(&self.completion.bytes());
		stream.out()
	}

	/// Converts the given bitfield (RLP encoded) to a hashset of available chunks
	pub fn read_rlp(manifest: &ManifestData, raw: &[u8]) -> Result<HashSet<H256>, DecoderError> {
		let decoder = Rlp::new(raw);
		let raw_bytes: Vec<u8> = decoder.list_at(0)?;

		let (hashes, len) = Bitfield::read_manifest(manifest);
		let mut available_chunks = HashSet::new();

		if raw_bytes.len() != len {
			return Err(DecoderError::RlpIncorrectListLen);
		}

		for (index, hash) in hashes.iter().enumerate() {
			if BitfieldCompletion::is_available(index, &raw_bytes) {
				available_chunks.insert(*hash);
			}
		}

		Ok(available_chunks)
	}

	/// Read the given Manifest file to collect the bytes length
	/// and the corresponding hashes
	pub fn read_manifest(manifest: &ManifestData) -> (Vec<H256>, usize) {
		let hashes = manifest.block_hashes
			.iter()
			.chain(manifest.state_hashes.iter())
			.map(|h| *h)
			.collect::<Vec<H256>>();

		let length = (hashes.len() as f64 / 8 as f64).ceil() as usize;
		(hashes, length)
	}

	/// Returns a HashSet of available chunks
	pub fn available_chunks(&self) -> HashSet<H256> {
		let iter = self.hashes.iter().enumerate()
			.filter(|(i, _)| BitfieldCompletion::is_available(*i, &self.completion.bytes))
			.map(|(_, h)| *h);
		HashSet::from_iter(iter)
	}

	/// Returns a HashSet of needed chunks
	pub fn needed_chunks(&self) -> HashSet<H256> {
		let iter = self.hashes.iter().enumerate()
			.filter(|(i, _)| !BitfieldCompletion::is_available(*i, &self.completion.bytes))
			.map(|(_, h)| *h);
		HashSet::from_iter(iter)
	}

	/// Returns the number of available chunks
	pub fn num_available(&self) -> usize {
		self.completion.num_available()
	}

	/// Mark one hash as completed
	pub fn mark_one(&mut self, hash: &H256) {
		// Find the index of the completed hash
		if let Some(index) = self.hashes.iter().position(|&h| h == *hash) {
			self.completion.mark(index);
		}
	}

	/// Mark some hashes as completed
	pub fn mark_some(&mut self, completed_hashes: &HashSet<H256>) {
		for (index, hash) in self.hashes.iter().enumerate() {
			if completed_hashes.contains(&hash) {
				self.completion.mark(index);
			}
		}
	}

	/// Mark all chunks as available
	pub fn mark_all(&mut self) {
		for (index, _) in self.hashes.iter().enumerate() {
			self.completion.mark(index);
		}
	}
}
