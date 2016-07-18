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
use std::io::ErrorKind;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use super::{ManifestData, StateRebuilder, BlockRebuilder, SnapshotService, RestorationStatus};
use super::io::{SnapshotReader, LooseReader};

use blockchain::BlockChain;
use client::get_db_path;
use engine::Engine;
use error::Error;
use service::ClientIoMessage;
use spec::Spec;

use util::{Bytes, H256, Mutex};
use util::io::IoChannel;
use util::journaldb::{self, Algorithm};
use util::snappy;

/// State restoration manager.
struct StateRestoration {
	chunks_left: HashSet<H256>,
	rebuilder: StateRebuilder,
	snappy_buffer: Bytes,
}

impl StateRestoration {
	// make a new state restoration, building databases in the given path.
	fn new(manifest: &ManifestData, pruning: Algorithm, path: &Path) -> Self {
		let mut state_db_path = path.to_owned();
		state_db_path.push("state");

		let state_db = journaldb::new(&*state_db_path.to_string_lossy(), pruning, Default::default());

		StateRestoration {
			chunks_left: manifest.state_hashes.iter().cloned().collect(),
			rebuilder: StateRebuilder::new(state_db),
			snappy_buffer: Vec::new(),
		}
	}

	// feeds a state chunk, returning true if all state chunks have been fed.
	fn feed(&mut self, hash: H256, chunk: &[u8]) -> Result<bool, Error> {
		if self.chunks_left.remove(&hash) {
			let len = try!(snappy::decompress_into(&chunk, &mut self.snappy_buffer));
			try!(self.rebuilder.feed(&self.snappy_buffer[..len]));
		}

		Ok(self.chunks_left.is_empty())
	}
}

/// Block restoration manager.
struct BlockRestoration {
	chunks_left: HashSet<H256>,
	rebuilder: BlockRebuilder,
	snappy_buffer: Bytes,
}

impl BlockRestoration {
	// create a new block restoration manager in the given path
	fn new(manifest: &ManifestData, genesis: &[u8], path: &Path) -> Result<Self, Error> {
		Ok(BlockRestoration {
			chunks_left: manifest.block_hashes.iter().cloned().collect(),
			rebuilder: try!(BlockRebuilder::new(BlockChain::new(Default::default(), genesis, path))),
			snappy_buffer: Vec::new(),
		})
	}

	// feeds a block chunk, returning true if all block chunks have been fed.
	fn feed(&mut self, hash: H256, chunk: &[u8], engine: &Engine) -> Result<(bool, u64), Error> {
		let mut block_count = 0;

		if self.chunks_left.remove(&hash) {
			let len = try!(snappy::decompress_into(&chunk, &mut self.snappy_buffer));
			block_count += try!(self.rebuilder.feed(&self.snappy_buffer[..len], engine));
		}

		Ok((self.chunks_left.is_empty(), block_count))
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
	state_restoration: Mutex<Option<StateRestoration>>,
	block_restoration: Mutex<Option<BlockRestoration>>,
	db_path: PathBuf,
	io_channel: Channel,
	pruning: Algorithm,
	status: Mutex<RestorationStatus>,
	genesis: Bytes,
	reader: Option<LooseReader>,
}

impl Service {
	/// Create a new snapshot service.
	pub fn new(spec: &Spec, pruning: Algorithm, db_path: PathBuf, io_channel: Channel) -> Result<Self, Error> {
		let reader = {
			let mut snapshot_path = db_path.clone();
			snapshot_path.push("snapshot");

			LooseReader::new(snapshot_path).ok()
		};

		let service = Service {
			engine: spec.engine.clone(),
			state_restoration: Mutex::new(None),
			block_restoration: Mutex::new(None),
			db_path: db_path,
			io_channel: io_channel,
			pruning: pruning,
			status: Mutex::new(RestorationStatus::Inactive),
			genesis: spec.genesis_block(),
			reader: reader
		};

		// create the snapshot dir if it doesn't exist.
		match fs::create_dir_all(service.snapshot_dir()) {
			Err(e) => {
				if e.kind() != ErrorKind::AlreadyExists {
					return Err(e.into())
				}
			}
			_ => {}
		}

		// delete the temporary restoration dir if it does exist.
		match fs::remove_dir_all(service.restoration_dir()) {
			Err(e) => {
				if e.kind() != ErrorKind::NotFound {
					return Err(e.into())
				}
			}
			_ => {}
		}

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

	// replace one of the client's databases with our own.
	// the database handle must be closed before doing this.
	fn replace_client_db(&self, name: &str) -> Result<(), Error> {
		let mut client_db = self.client_db_root();
		client_db.push(name);

		let mut our_db = self.db_path.clone();
		our_db.push(name);

		let mut backup_db = self.db_path.clone();
		backup_db.push(format!("backup_{}", name));

		try!(fs::remove_dir_all(&backup_db));
		try!(fs::rename(&client_db, &backup_db));
		match fs::rename(&our_db, &client_db) {
			Ok(_) => {
				// clean up the backup.
				try!(fs::remove_dir_all(&backup_db));
				Ok(())
			}
			Err(e) => {
				// restore the backup.
				try!(fs::rename(&backup_db, client_db));
				Err(e.into())
			}
		}
	}

	// finalize the restoration.
	fn finalize_restoration(&self) {
		*self.state_restoration.lock() = None;
		*self.block_restoration.lock() = None;

		*self.status.lock() = RestorationStatus::Inactive;

		// TODO: take control of restored snapshot.
		let _ = fs::remove_dir_all(self.restoration_dir());
	}

	/// Feed a state chunk to be processed synchronously.
	pub fn feed_state_chunk(&self, hash: H256, chunk: &[u8]) {
		let mut restoration = self.state_restoration.lock();
		let mut finished = false;

		if let Some(ref mut rest) = *restoration {
			match rest.feed(hash, chunk) {
				Ok(status) => {
					finished = status;
					match *self.status.lock() {
						RestorationStatus::Ongoing(_, ref mut state_chunks) => *state_chunks += 1,
						_ => panic!("feed_state_chunk guarded for ongoing status only; qed")
					}
				},
				Err(e) => {
					warn!("state chunk {} restoration failed: {}", hash, e);
					*self.status.lock() = RestorationStatus::Failed;
				}
			}
		}

		if finished {
			// replace state db here. ensure database handle is closed.
			*restoration = None;
			if let Err(e) = self.replace_client_db("state") {
				warn!("failed to restore client state db: {}", e);
				*self.status.lock() = RestorationStatus::Failed;
				return;
			}

			if self.block_restoration.lock().as_ref().map(|r| r.chunks_left.is_empty()).unwrap_or(false) {
				self.finalize_restoration();
			}
		}
	}

	/// Feed a block chunk to be processed synchronously.
	pub fn feed_block_chunk(&self, hash: H256, chunk: &[u8]) {
		let mut restoration = self.block_restoration.lock();
		let mut finished = false;

		if let Some(ref mut rest) = *restoration {
			match rest.feed(hash, chunk, &*self.engine) {
				Ok((status, bc)) => {
					finished = status;
					match *self.status.lock() {
						RestorationStatus::Ongoing(ref mut block_count, _) => *block_count += bc,
						_ => panic!("feed_block_chunk guarded for ongoing status only; qed")
					}
				}
				Err(e) => {
					warn!("block chunk {} restoration failed: {}", hash, e);
					*self.status.lock() = RestorationStatus::Failed;
				}
			}
		}

		if finished {
			// replace blocks and extras dbs here. ensure database handles are closed.
			*restoration = None;
			if let Err(e) = self.replace_client_db("blocks").and_then(|_| self.replace_client_db("extras")) {
				warn!("failed to restore blocks and extras databases: {}", e);
				*self.status.lock() = RestorationStatus::Failed;

				return;
			}

			if self.state_restoration.lock().as_ref().map(|r| r.chunks_left.is_empty()).unwrap_or(false) {
				self.finalize_restoration();
			}
		}
	}
}

impl SnapshotService for Service {
	fn manifest(&self) -> Option<ManifestData> {
		self.reader.as_ref().map(|r| r.manifest().clone())
	}

	fn chunk(&self, hash: H256) -> Option<Bytes> {
		self.reader.as_ref().and_then(|r| r.chunk(hash).ok())
	}

	fn status(&self) -> RestorationStatus {
		*self.status.lock()
	}

	fn begin_restore(&self, manifest: ManifestData) -> bool {
		let rest_dir = self.restoration_dir();

		let mut state_res = self.state_restoration.lock();
		let mut block_res = self.block_restoration.lock();

		// tear down existing restorations.
		*state_res = None;
		*block_res = None;

		// delete and restore the restoration dir.
		if let Err(e) = fs::remove_dir_all(&rest_dir).and_then(|_| fs::create_dir_all(&rest_dir)) {
			match e.kind() {
				ErrorKind::NotFound => {},
				_ => {
					warn!("encountered error {} while beginning snapshot restoration.", e);
					return false;
				}
			}
		}

		// make new restorations.
		*block_res = match BlockRestoration::new(&manifest, &self.genesis, &rest_dir) {
				Ok(b) => Some(b),
				Err(e) => {
					warn!("encountered error {} while beginning snapshot restoration.", e);
					return false;
				}
		};
		*state_res = Some(StateRestoration::new(&manifest, self.pruning, &rest_dir));

		*self.status.lock() = RestorationStatus::Ongoing(0, 0);
		true
	}

	fn restore_state_chunk(&self, hash: H256, chunk: Bytes) {
		if let RestorationStatus::Ongoing(_, _) = self.status() {
			self.io_channel.send(ClientIoMessage::FeedStateChunk(hash, chunk))
				.expect("snapshot service and io service are kept alive by client service; qed");
		}
	}

	fn restore_block_chunk(&self, hash: H256, chunk: Bytes) {
		if let RestorationStatus::Ongoing(_, _) = self.status() {
			self.io_channel.send(ClientIoMessage::FeedBlockChunk(hash, chunk))
				.expect("snapshot service and io service are kept alive by client service; qed");
		}
	}
}