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

//! Snapshot and restoration commands.

use std::time::Duration;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use ethcore_logger::{setup_log, Config as LogConfig};
use ethcore::snapshot::{Progress, RestorationStatus, SnapshotService};
use ethcore::snapshot::io::{SnapshotReader, PackedReader, PackedWriter};
use ethcore::service::ClientService;
use ethcore::client::{Mode, DatabaseCompactionProfile, Switch, VMType};
use ethcore::miner::Miner;
use ethcore::ids::BlockID;

use cache::CacheConfig;
use params::{SpecType, Pruning};
use helpers::{to_client_config, execute_upgrades};
use dir::Directories;
use fdlimit;

use io::PanicHandler;

/// Kinds of snapshot commands.
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Kind {
	/// Take a snapshot.
	Take,
	/// Restore a snapshot.
	Restore
}

/// Command for snapshot creation or restoration.
#[derive(Debug, PartialEq)]
pub struct SnapshotCommand {
	pub cache_config: CacheConfig,
	pub dirs: Directories,
	pub spec: SpecType,
	pub pruning: Pruning,
	pub pruning_history: u64,
	pub logger_config: LogConfig,
	pub mode: Mode,
	pub tracing: Switch,
	pub compaction: DatabaseCompactionProfile,
	pub file_path: Option<String>,
	pub wal: bool,
	pub kind: Kind,
	pub block_at: BlockID,
}

impl SnapshotCommand {
	// shared portion of snapshot commands: start the client service
	fn start_service(self) -> Result<(ClientService, Arc<PanicHandler>), String> {
		// Setup panic handler
		let panic_handler = PanicHandler::new_in_arc();

		// load spec file
		let spec = try!(self.spec.spec());

		// load genesis hash
		let genesis_hash = spec.genesis_header().hash();

		// Setup logging
		let _logger = setup_log(&self.logger_config);

		fdlimit::raise_fd_limit();

		// select pruning algorithm
		let algorithm = self.pruning.to_algorithm(&self.dirs, genesis_hash, spec.fork_name.as_ref());

		// prepare client_path
		let client_path = self.dirs.client_path(genesis_hash, spec.fork_name.as_ref(), algorithm);

		// execute upgrades
		try!(execute_upgrades(&self.dirs, genesis_hash, spec.fork_name.as_ref(), algorithm, self.compaction.compaction_profile()));

		// prepare client config
		let client_config = to_client_config(&self.cache_config, &self.dirs, genesis_hash, self.mode, self.tracing, self.pruning, self.pruning_history, self.compaction, self.wal, VMType::default(), "".into(), spec.fork_name.as_ref());

		let service = try!(ClientService::start(
			client_config,
			&spec,
			Path::new(&client_path),
			Arc::new(Miner::with_spec(&spec))
		).map_err(|e| format!("Client service error: {:?}", e)));

		Ok((service, panic_handler))
	}

	/// restore from a snapshot
	pub fn restore(self) -> Result<(), String> {
		let file = try!(self.file_path.clone().ok_or("No file path provided.".to_owned()));
		let (service, _panic_handler) = try!(self.start_service());

		warn!("Snapshot restoration is experimental and the format may be subject to change.");

		let snapshot = service.snapshot_service();
		let reader = PackedReader::new(Path::new(&file))
			.map_err(|e| format!("Couldn't open snapshot file: {}", e))
			.and_then(|x| x.ok_or("Snapshot file has invalid format.".into()));

		let reader = try!(reader);
		let manifest = reader.manifest();

		// drop the client so we don't restore while it has open DB handles.
		drop(service);

		if !snapshot.begin_restore(manifest.clone()) {
			return Err("Failed to begin restoration.".into());
		}

		let (num_state, num_blocks) = (manifest.state_hashes.len(), manifest.block_hashes.len());

		let informant_handle = snapshot.clone();
		::std::thread::spawn(move || {
 			while let RestorationStatus::Ongoing = informant_handle.status() {
 				let (state_chunks, block_chunks) = informant_handle.chunks_done();
 				info!("Processed {}/{} state chunks and {}/{} block chunks.",
 					state_chunks, num_state, block_chunks, num_blocks);

 				::std::thread::sleep(Duration::from_secs(5));
 			}
 		});

 		info!("Restoring state");
 		for &state_hash in &manifest.state_hashes {
 			if snapshot.status() == RestorationStatus::Failed {
 				return Err("Restoration failed".into());
 			}

 			let chunk = try!(reader.chunk(state_hash)
				.map_err(|e| format!("Encountered error while reading chunk {:?}: {}", state_hash, e)));
 			snapshot.feed_state_chunk(state_hash, &chunk);
 		}

		info!("Restoring blocks");
		for &block_hash in &manifest.block_hashes {
			if snapshot.status() == RestorationStatus::Failed {
				return Err("Restoration failed".into());
			}

 			let chunk = try!(reader.chunk(block_hash)
				.map_err(|e| format!("Encountered error while reading chunk {:?}: {}", block_hash, e)));
			snapshot.feed_block_chunk(block_hash, &chunk);
		}

		match snapshot.status() {
			RestorationStatus::Ongoing => Err("Snapshot file is incomplete and missing chunks.".into()),
			RestorationStatus::Failed => Err("Snapshot restoration failed.".into()),
			RestorationStatus::Inactive => {
				info!("Restoration complete.");
				Ok(())
			}
		}
	}

	/// Take a snapshot from the head of the chain.
	pub fn take_snapshot(self) -> Result<(), String> {
		let file_path = try!(self.file_path.clone().ok_or("No file path provided.".to_owned()));
		let file_path: PathBuf = file_path.into();
		let block_at = self.block_at;
		let (service, _panic_handler) = try!(self.start_service());

		warn!("Snapshots are currently experimental. File formats may be subject to change.");

		let writer = try!(PackedWriter::new(&file_path)
			.map_err(|e| format!("Failed to open snapshot writer: {}", e)));

		let progress = Arc::new(Progress::default());
		let p = progress.clone();
		let informant_handle = ::std::thread::spawn(move || {
			::std::thread::sleep(Duration::from_secs(5));

			let mut last_size = 0;
			while !p.done() {
				let cur_size = p.size();
				if cur_size != last_size {
					last_size = cur_size;
					info!("Snapshot: {} accounts {} blocks {} bytes", p.accounts(), p.blocks(), p.size());
				} else {
					info!("Snapshot: No progress since last update.");
				}

				::std::thread::sleep(Duration::from_secs(5));
			}
 		});

		if let Err(e) = service.client().take_snapshot(writer, block_at, &*progress) {
			let _ = ::std::fs::remove_file(&file_path);
			return Err(format!("Encountered fatal error while creating snapshot: {}", e));
		}

		info!("snapshot creation complete");

		assert!(progress.done());
		try!(informant_handle.join().map_err(|_| "failed to join logger thread"));

		Ok(())
	}
}

/// Execute this snapshot command.
pub fn execute(cmd: SnapshotCommand) -> Result<String, String> {
	match cmd.kind {
		Kind::Take => try!(cmd.take_snapshot()),
		Kind::Restore => try!(cmd.restore()),
	}

	Ok(String::new())
}
