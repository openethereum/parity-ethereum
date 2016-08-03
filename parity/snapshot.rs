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

use std::str::{FromStr, from_utf8};
use std::io::{BufReader, BufRead};
use std::time::Duration;
use std::thread::sleep;
use std::path::Path;
use std::sync::Arc;
use ethcore_logger::{setup_log, Config as LogConfig};
use util::panics::{PanicHandler, ForwardPanic};
use ethcore::snapshot::{RestorationStatus, SnapshotService};
use ethcore::snapshot::io::{SnapshotReader, PackedReader};
use ethcore::service::ClientService;
use ethcore::client::{Mode, DatabaseCompactionProfile, Switch, VMType, BlockImportError, BlockChainClient, BlockID};
use ethcore::error::ImportError;
use ethcore::miner::Miner;
use cache::CacheConfig;
use informant::Informant;
use params::{SpecType, Pruning};
use helpers::{to_client_config, execute_upgrades};
use dir::Directories;
use fdlimit;

/// Command for snapshot creation or restoration.
pub struct SnapshotCommand {
	pub dirs: Directories,
	pub spec: SpecType,
	pub pruning: Pruning,
	pub logger_config: LogConfig,
	pub miner_options: MinerOptions,
	pub mode: Mode,
	pub tracing: Switch,
	pub compaction: DatabaseCompactionProfile,
	pub filename: PathBuf,
}

impl SnapshotCommand {
	// shared portion of snapshot commands: start the client service
	fn start_service(self) -> Result<(Arc<ClientService>, Arc<PanicHandler>), String> {
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
		let client_config = to_client_config(&self.cache_config, &self.dirs, genesis_hash, self.mode, self.tracing, self.pruning, self.compaction, self.wal, VMType::default(), "".into(), spec.fork_name.as_ref());

		let service = try!(ClientService::start(
			client_config,
			spec,
			Path::new(&client_path),
			Arc::new(Miner::with_spec(try!(self.spec.spec())))
		).map_err(|e| format!("Client service error: {:?}", e)));

		(service, panic_handler)
	}

	/// restore from a snapshot
	pub fn restore(self) -> Result<(), String> {
		let filename = self.filename.clone();
		let (service, panic_handler) = try!(self.start_service());

		let snapshot = service.snapshot_service();
		let reader = PackedReader::new(&filename)
			.map_err(|e| format!("Couldn't open snapshot file: {}", e))
			.and_then(|x| x.ok_or("Snapshot file has invalid format.".into()));

		let reader = try!(reader);
		let manifest = reader.manifest();

		// drop the client so we don't restore while it has open DB handles.
		drop(service);

		if !snapshot.begin_restoration(manifest.clone()) {
			return Err("Failed to begin restoration.".into());
		}

		let (num_state, num_blocks) = (manifest.state_hashes.len(), manifest.block_hashes.len());

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
				.ok_or_else(|| format!("Failed to read chunk {:?} from snapshot file.", state_hash)));
 			snapshot.feed_state_chunk(state_hash, &chunk);
 		}

		info!("Restoring blocks");
		for &block_hash in &manifest.state_hashes {
			if snapshot.status() ==RestorationStatus::Failed {
				return Err("Restoration failed".into());

				let chunk = try!(reader.chunk(block_hash)
					.ok_or_else(|| format!("Failed to read chunk {:?} from snapshot file.", block_hash)));
				snapshot.feed_block_chunk(block_hash, &chunk);
			}
		}

		match snapshot.status() {
			RestorationStatus::Ongoing => Err("Snapshot file is incomplete and missing chunks.".into()),
			RestoratoinStatus::Failed => Err("Snapshot restoration failed.".into()),
			RestorationStatus::Inactive => {
				info!("Restoration complete.");
				Ok(())
			}
		}
	}

	/// Take a snapshot from the head of the chain.
	pub fn take_snapshot(self) -> Result<(), String> {
		let filename = self.filename.clone();
		let (service, panic_handler) = try!(self.start_service());

		let writer = try!(PackedWriter::new(&filename)
			.map_err(|e| format!("Failed to open snapshot writer: {}", e)));

		if let Err(e) = service.client().take_snapshot(writer) {
			let _ = ::std::fs::remove(&filename);
			return Err(format!("Encountered fatal error while creating snapshot: {}", e));
		}

		Ok(())
	}
}