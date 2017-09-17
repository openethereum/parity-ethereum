// Copyright 2015-2017 Parity Technologies (UK) Ltd.
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

use super::{ManifestData, RestorationStatus};
use bigint::hash::H256;
use bytes::Bytes;
use ipc::IpcConfig;

/// The interface for a snapshot network service.
/// This handles:
///    - restoration of snapshots to temporary databases.
///    - responding to queries for snapshot manifests and chunks
#[ipc(client_ident="RemoteSnapshotService")]
pub trait SnapshotService : Sync + Send {
	/// Query the most recent manifest data.
	fn manifest(&self) -> Option<ManifestData>;

	/// Get the supported range of snapshot version numbers.
	/// `None` indicates warp sync isn't supported by the consensus engine.
	fn supported_versions(&self) -> Option<(u64, u64)>;

	/// Get raw chunk for a given hash.
	fn chunk(&self, hash: H256) -> Option<Bytes>;

	/// Ask the snapshot service for the restoration status.
	fn status(&self) -> RestorationStatus;

	/// Begin snapshot restoration.
	/// If restoration in-progress, this will reset it.
	/// From this point on, any previous snapshot may become unavailable.
	fn begin_restore(&self, manifest: ManifestData);

	/// Abort an in-progress restoration if there is one.
	fn abort_restore(&self);

	/// Feed a raw state chunk to the service to be processed asynchronously.
	/// no-op if not currently restoring.
	fn restore_state_chunk(&self, hash: H256, chunk: Bytes);

	/// Feed a raw block chunk to the service to be processed asynchronously.
	/// no-op if currently restoring.
	fn restore_block_chunk(&self, hash: H256, chunk: Bytes);
}

impl IpcConfig for SnapshotService { }
