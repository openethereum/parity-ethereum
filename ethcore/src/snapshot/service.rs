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

use std::path::PathBuf;
use std::sync::Arc;

use super::{take_snapshot, ManifestData, StateRebuilder};
use engine::Engine;
use error::Error;

use util::{Bytes, H256, Mutex};
use util::journaldb::{Algorithm, JournalDB};

/// Intermediate restoration state.
struct Restoration {
	manifest: ManifestData,
	state_rebuilder: StateRebuilder,
}

/// Service implementation.
pub struct Service {
	engine: Arc<Box<Engine>>,
	pruning: Algorithm,
	restoration_manifest: Arc<Mutex<Option<Restoration>>>,
	snapshot_root: PathBuf,
	db_path: PathBuf,
}