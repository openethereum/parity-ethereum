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

//! Snapshot network service implementation for light client.

use std::path::PathBuf;
use std::sync::Arc;


use ethcore::snapshot::{
	ManifestData,
	Error as SnapshotError,
	io::{LooseWriter},
	service::{ChainRestorationParams, Restoration},
};

use cache::Cache;
use client::HeaderChain;
use client::header_chain::HardcodedSync;
use ethcore::BlockChainDBHandler;
use ethcore::engines::EthEngine;
use ethcore::spec::Spec;
use ethcore::error::Error;
use client::snapshot::chain::LightChain;

use parking_lot::Mutex;

/// Light client specific snapshot restoration params.
pub struct LightClientRestorationParams {
	pub(crate) spec: Spec,
	pub(crate) allow_hs: HardcodedSync,
	pub(crate) col: Option<u32>,
	pub(crate) cache: Arc<Mutex<Cache>>,
}

impl ChainRestorationParams for LightClientRestorationParams {
	fn restoration(
		&self,
		manifest: ManifestData,
		rest_db: PathBuf,
		restoration_db_handler: &BlockChainDBHandler,
		writer: Option<LooseWriter>,
		engine: &EthEngine,
	) -> Result<Restoration, Error> {
		let db = restoration_db_handler.open(&rest_db)?;
		let chain = HeaderChain::new(
			db.key_value().clone(),
			self.col,
			&self.spec,
			self.cache.clone(),
			self.allow_hs
		)?;
		let boxed = Box::new(LightChain::new(chain));
		let restoration = engine.snapshot_components()
			.ok_or_else(|| SnapshotError::SnapshotsUnsupported)?;
		let rebuilder = restoration.rebuilder(boxed, db.clone(), &manifest)?;

		Ok(Restoration::new_light(
			manifest,
			rest_db,
			writer,
			db,
			rebuilder,
		))
	}

	fn is_light(&self) -> bool {
		true
	}
}
