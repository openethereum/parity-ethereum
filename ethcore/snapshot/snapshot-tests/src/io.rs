// Copyright 2015-2020 Parity Technologies (UK) Ltd.
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

//! Tests for snapshot i/o.

use tempdir::TempDir;
use keccak_hash::keccak;

use common_types::snapshot::ManifestData;
use snapshot::io::{
	SnapshotWriter,SnapshotReader,
	PackedWriter, PackedReader, LooseWriter, LooseReader,
	SNAPSHOT_VERSION,
};

const STATE_CHUNKS: &'static [&'static [u8]] = &[b"dog", b"cat", b"hello world", b"hi", b"notarealchunk"];
const BLOCK_CHUNKS: &'static [&'static [u8]] = &[b"hello!", b"goodbye!", b"abcdefg", b"hijklmnop", b"qrstuvwxy", b"and", b"z"];

#[test]
fn packed_write_and_read() {
	let tempdir = TempDir::new("").unwrap();
	let path = tempdir.path().join("packed");
	let mut writer = PackedWriter::new(&path).unwrap();

	let mut state_hashes = Vec::new();
	let mut block_hashes = Vec::new();

	for chunk in STATE_CHUNKS {
		let hash = keccak(&chunk);
		state_hashes.push(hash.clone());
		writer.write_state_chunk(hash, chunk).unwrap();
	}

	for chunk in BLOCK_CHUNKS {
		let hash = keccak(&chunk);
		block_hashes.push(hash.clone());
		writer.write_block_chunk(keccak(&chunk), chunk).unwrap();
	}

	let manifest = ManifestData {
		version: SNAPSHOT_VERSION,
		state_hashes,
		block_hashes,
		state_root: keccak(b"notarealroot"),
		block_number: 12345678987654321,
		block_hash: keccak(b"notarealblock"),
	};

	writer.finish(manifest.clone()).unwrap();

	let reader = PackedReader::new(&path).unwrap().unwrap();
	assert_eq!(reader.manifest(), &manifest);

	for hash in manifest.state_hashes.iter().chain(&manifest.block_hashes) {
		reader.chunk(hash.clone()).unwrap();
	}
}

#[test]
fn loose_write_and_read() {
	let tempdir = TempDir::new("").unwrap();
	let mut writer = LooseWriter::new(tempdir.path().into()).unwrap();

	let mut state_hashes = Vec::new();
	let mut block_hashes = Vec::new();

	for chunk in STATE_CHUNKS {
		let hash = keccak(&chunk);
		state_hashes.push(hash.clone());
		writer.write_state_chunk(hash, chunk).unwrap();
	}

	for chunk in BLOCK_CHUNKS {
		let hash = keccak(&chunk);
		block_hashes.push(hash.clone());
		writer.write_block_chunk(keccak(&chunk), chunk).unwrap();
	}

	let manifest = ManifestData {
		version: SNAPSHOT_VERSION,
		state_hashes,
		block_hashes,
		state_root: keccak(b"notarealroot"),
		block_number: 12345678987654321,
		block_hash: keccak(b"notarealblock)"),
	};

	writer.finish(manifest.clone()).unwrap();

	let reader = LooseReader::new(tempdir.path().into()).unwrap();
	assert_eq!(reader.manifest(), &manifest);

	for hash in manifest.state_hashes.iter().chain(&manifest.block_hashes) {
		reader.chunk(hash.clone()).unwrap();
	}
}
