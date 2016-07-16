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

//! Snapshot i/o.
//! Ways of writing and reading snapshots. This module supports writing and reading
//! snapshots of two different formats: packed and loose.
//! Packed snapshots are written to a single file, and loose snapshots are
//! written to multiple files in one directory.

use std::io::{self, Write};
use std::fs::{self, File};
use std::path::{Path, PathBuf};

use util::hash::H256;
use util::rlp::{Encodable, RlpStream, Stream};

use super::ManifestData;

/// Something which can write snapshots.
/// Writing the same chunk multiple times will lead to implementation-defined
/// behavior, and is not advised.
pub trait SnapshotWriter {
	/// Write a compressed state chunk.
	fn write_state_chunk(&mut self, hash: H256, chunk: &[u8]) -> io::Result<()>;

	/// Write a compressed block chunk.
	fn write_block_chunk(&mut self, hash: H256, chunk: &[u8]) -> io::Result<()>;

	/// Complete writing. The manifest's chunk lists must be consistent
	/// with the chunks written.
	fn finish(self, manifest: ManifestData) -> io::Result<()> where Self: Sized;
}

// (hash, len, offset)
struct ChunkInfo(H256, u64, u64);

impl Encodable for ChunkInfo {
	fn rlp_append(&self, s: &mut RlpStream) {
		s.append(&self.0).append(&self.1).append(&self.2);
	}
}

/// A packed snapshot writer. This writes snapshots to a single concatenated file.
///
/// The file format is very simple and consists of three parts:
/// 	[Concatenated chunk data]
/// 	[manifest as RLP]
///     [manifest start offset]
///
/// The manifest contains all the same information as a standard `ManifestData`,
/// but also maps chunk hashes to their lengths and offsets in the file
/// for easy reading.
pub struct PackedWriter {
	file: File,
	state_hashes: Vec<ChunkInfo>,
	block_hashes: Vec<ChunkInfo>,
	cur_len: u64,
}

impl PackedWriter {
	/// Create a new "PackedWriter", to write into the file at the given path.
	pub fn new(path: &Path) -> io::Result<Self> {
		Ok(PackedWriter {
			file: try!(File::create(path)),
			state_hashes: Vec::new(),
			block_hashes: Vec::new(),
			cur_len: 0,
		})
	}
}

impl SnapshotWriter for PackedWriter {
	fn write_state_chunk(&mut self, hash: H256, chunk: &[u8]) -> io::Result<()> {
		try!(self.file.write_all(chunk));

		let len = chunk.len() as u64;
		self.state_hashes.push(ChunkInfo(hash, len, self.cur_len));

		self.cur_len += len;
		Ok(())
	}

	fn write_block_chunk(&mut self, hash: H256, chunk: &[u8]) -> io::Result<()> {
		try!(self.file.write_all(chunk));

		let len = chunk.len() as u64;
		self.block_hashes.push(ChunkInfo(hash, len, self.cur_len));

		self.cur_len += len;
		Ok(())
	}

	fn finish(mut self, manifest: ManifestData) -> io::Result<()> {
		// we ignore the hashes fields of the manifest under the assumption that
		// they are consistent with ours.
		let mut stream = RlpStream::new_list(5);
		stream
			.append(&self.state_hashes)
			.append(&self.block_hashes)
			.append(&manifest.state_root)
			.append(&manifest.block_number)
			.append(&manifest.block_hash);

		try!(self.file.write_all(&stream.drain()));
		let off = self.cur_len;
		let off_bytes: [u8; 8] =
			[
				off as u8,
				(off >> 8) as u8,
				(off >> 16) as u8,
				(off >> 24) as u8,
				(off >> 32) as u8,
				(off >> 40) as u8,
				(off >> 48) as u8,
				(off >> 56) as u8,
			];

		try!(self.file.write_all(&off_bytes[..]));

		Ok(())
	}
}

/// A "loose" writer writes chunk files into a directory.
pub struct LooseWriter {
	dir: PathBuf,
}

impl LooseWriter {
	/// Create a new LooseWriter which will write into the given directory,
	/// creating it if it doesn't exist.
	pub fn new(path: PathBuf) -> io::Result<Self> {
		try!(fs::create_dir_all(&path));

		Ok(LooseWriter {
			dir: path,
		})
	}

	// writing logic is the same for both kinds of chunks.
	fn write_chunk(&mut self, hash: H256, chunk: &[u8]) -> io::Result<()> {
		let mut file_path = self.dir.clone();
		file_path.push(hash.hex());

		let mut file = try!(File::create(file_path));
		try!(file.write_all(chunk));

		Ok(())
	}
}

impl SnapshotWriter for LooseWriter {
	fn write_state_chunk(&mut self, hash: H256, chunk: &[u8]) -> io::Result<()> {
		self.write_chunk(hash, chunk)
	}

	fn write_block_chunk(&mut self, hash: H256, chunk: &[u8]) -> io::Result<()> {
		self.write_chunk(hash, chunk)
	}

	fn finish(self, manifest: ManifestData) -> io::Result<()> {
		let rlp = manifest.to_rlp();
		let mut path = self.dir.clone();
		path.push("MANIFEST");

		let mut file = try!(File::create(path));
		try!(file.write_all(&rlp[..]));

		Ok(())
	}
}