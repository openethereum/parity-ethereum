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

//! Snapshot network service implementation.

use std::collections::HashSet;
use std::fs::{self, File};
use std::io::Read;
use std::path::PathBuf;
use std::sync::Arc;

use super::{take_snapshot, ManifestData, StateRebuilder, BlockRebuilder, SnapshotService};

use client::get_db_path;
use engine::Engine;
use error::Error;
use service::ClientIoMessage;
use spec::Spec;

use util::{Bytes, H256, Mutex};
use util::io::IoChannel;
use util::journaldb::{Algorithm, JournalDB};
use util::snappy;

/// Intermediate restoration state.
struct Restoration {
	manifest: ManifestData,
	state_chunks_left: HashSet<H256>,
	block_chunks_left: HashSet<H256>,

	state_rebuilder: StateRebuilder,
	block_rebuilder: BlockRebuilder,

	snappy_buffer: Bytes,
}

impl Restoration {
	// whether state restoration is complete
	fn is_state_complete(&self) -> bool {
		self.state_chunks_left.is_empty()
	}

	// whether block restoration is complete
	fn is_block_complete(&self) -> bool {
		self.block_chunks_left.is_empty()
	}

	// feeds a state chunk, returning true if all state chunks have been fed.
	fn feed_state_chunk(&mut self, hash: H256, chunk: &[u8]) -> Result<bool, Error> {
		if self.state_chunks_left.remove(&hash) {
			let len = try!(snappy::decompress_into(&chunk, &mut self.snappy_buffer));
			try!(self.state_rebuilder.feed(&self.snappy_buffer[..len]));
		}

		Ok(self.is_state_complete())
	}

	// feeds a block chunk, returning true if all block chunks have been fed.
	fn feed_block_chunk(&mut self, hash: H256, chunk: &[u8], engine: &Engine) -> Result<bool, Error> {
		if self.block_chunks_left.remove(&hash) {
			let len = try!(snappy::decompress_into(&chunk, &mut self.snappy_buffer));
			try!(self.block_rebuilder.feed(&self.snappy_buffer[..len], engine));
		}

		Ok(self.is_block_complete())
	}
}

/// Type alias for client io channel.
pub type Channel = IoChannel<ClientIoMessage>;

/// Service implementation.
///
/// This will replace the client's state DB as soon as the last state chunk
/// is fed, and will replace the client's blocks DB when the last block chunk
/// is fed.
pub struct Service {
	engine: Arc<Engine>,
	restoration: Mutex<Option<Restoration>>,
	db_path: PathBuf,
	last_error: Mutex<Result<(), Error>>,
	io_channel: Channel,
	pruning: Algorithm
}

impl Service {
	/// Create a new snapshot service.
	pub fn new(spec: &Spec, pruning: Algorithm, mut db_path: PathBuf, io_channel: Channel) -> Result<Self, Error> {
		let service = Service {
			engine: spec.engine.clone(),
			restoration: Mutex::new(None),
			db_path: db_path,
			last_error: Mutex::new(Ok(())),
			io_channel: io_channel,
			pruning: pruning,
		};

		// create the snapshot dir if it doesn't exist.
		try!(fs::create_dir_all(service.snapshot_dir()));

		// delete the temporary restoration dir if it does exist.
		try!(fs::remove_dir_all(service.restoration_dir()));

		Ok(service)
	}

	// Get the client db root.
	fn client_db_root(&self) -> PathBuf {
		get_db_path(&self.db_path, self.pruning)
	}

	// Get the snapshot directory path.
	fn snapshot_dir(&self) -> PathBuf {
		let mut path = self.db_path.clone();
		path.push("snapshot");
		path
	}

	// Get the restoration directory path.
	fn restoration_dir(&self) -> PathBuf {
		let mut path = self.snapshot_dir();
		path.push("restoration");
		path
	}

	fn read_local_file(&self, name: &str) -> Result<Bytes, ::std::io::Error> {
		let mut filename = self.snapshot_dir();
		filename.push(name);

		let mut file = try!(File::open(&filename));

		// TODO [rob] handle big files
		let len = file.metadata().map(|m| m.len()).unwrap_or(0);
		let mut bytes = Vec::with_capacity(len as usize);

		try!(file.read_to_end(&mut bytes));

		Ok(bytes)
	}

	// replace one of the client's databases with our own.
	fn replace_client_db(&self, name: &str) -> Result<(), Error> {
		let mut client_db = self.client_db_root();
		client_db.push(name);

		let mut our_db = self.db_path.clone();
		our_db.push(name);

		// TODO [rob] backup?
		try!(fs::remove_dir_all(&client_db));
		try!(fs::rename(&our_db, &client_db));

		Ok(())
	}

	/// Feed a state chunk to be processed synchronously.
	pub fn feed_state_chunk(&self, hash: H256, chunk: &[u8]) {
		let mut restoration = self.restoration.lock();

		if let Some(ref mut rest) = *restoration {
			match rest.feed_state_chunk(hash, chunk) {
				Ok(true) => {
					// replace state db here.
					let _ = self.replace_client_db("state");

					// if blocks are also done, destroy the restoration context.
				}
				Err(e) => *self.last_error.lock() = Err(e),
				_ => {}
			}
		}
	}

	/// Feed a block chunk to be processed synchronously.
	pub fn feed_block_chunk(&self, hash: H256, chunk: &[u8]) {
		let mut restoration = self.restoration.lock();

		if let Some(ref mut rest) = *restoration {
			match rest.feed_block_chunk(hash, chunk, &*self.engine) {
				Ok(true) => {
					// replace blocks and extras dbs here.
					unimplemented!()

					// if state is also done, destroy the restoration context.
				}
				Err(e) => *self.last_error.lock() = Err(e),
				_ => {}
			}
		}
	}
}

impl SnapshotService for Service {
	fn manifest(&self) -> Option<ManifestData> {
		match self.read_local_file("MANIFEST") {
			Ok(raw) => ManifestData::from_rlp(&raw).ok(),
			Err(_) => None,
		}
	}

	fn chunk(&self, hash: H256) -> Result<Bytes, Error> {
		self.read_local_file(&hash.hex()).map_err(Into::into)
	}

	fn last_error(&self) -> Result<(), Error> {
		::std::mem::replace(&mut *self.last_error.lock(), Ok(()))
	}

	fn begin_restore(&self, manifest: ManifestData) {
		// delete the existing restoration directory and create a new
		// `Restoration`
		unimplemented!()
	}

	fn restore_state_chunk(&self, hash: H256, chunk: Bytes) {
		// TODO [rob] return an error here.
		let _ = self.io_channel.send(ClientIoMessage::FeedStateChunk(hash, chunk));
	}

	fn restore_block_chunk(&self, hash: H256, chunk: Bytes) {
		let _ = self.io_channel.send(ClientIoMessage::FeedBlockChunk(hash, chunk));
	}
}