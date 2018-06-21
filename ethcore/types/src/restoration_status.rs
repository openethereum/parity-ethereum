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

//! Restoration status type definition

/// Statuses for restorations.
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum RestorationStatus {
	///	No restoration.
	Inactive,
	/// Restoration is initalizing
	Initializing {
		/// Number of chunks done/imported
		chunks_done: u32,
	},
	/// Ongoing restoration.
	Ongoing {
		/// Total number of state chunks.
		state_chunks: u32,
		/// Total number of block chunks.
		block_chunks: u32,
		/// Number of state chunks completed.
		state_chunks_done: u32,
		/// Number of block chunks completed.
		block_chunks_done: u32,
	},
	/// Failed restoration.
	Failed,
}
