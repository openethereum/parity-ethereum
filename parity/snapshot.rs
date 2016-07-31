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

/// Command for snapshot creation or restoration.
pub enum SnapshotCmd {
	Create(CreateSnapshot),
	Restore(RestoreSnapshot),
}

/// Execute a snapshot command.
pub fn execute(cmd: SnapshotCmd) {
	match cmd {
		SnapshotCmd::Create(create) => create.execute(),
		SnapshotCmd::Restore(restore) => restore.execute(),
	}
}

/// The snapshot command.
///
/// This creates a snapshot file at the given path.
pub struct CreateSnapshot {

}

impl CreateSnapshot {
	pub fn execute(self) {
		unimplemented!()
	}
}

/// The snapshot restoration command: reads from the provided file and
/// restores the given state and blocks into the database.
pub struct RestoreSnapshot {

}

impl RestoreSnapshot {
	pub fn execute(self) {
		unimplemented!()
	}
}