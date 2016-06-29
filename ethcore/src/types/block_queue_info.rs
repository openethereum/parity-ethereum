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

/// Block queue status
#[derive(Debug, Binary)]
pub struct BlockQueueInfo {
	/// Number of queued blocks pending verification
	pub unverified_queue_size: usize,
	/// Number of verified queued blocks pending import
	pub verified_queue_size: usize,
	/// Number of blocks being verified
	pub verifying_queue_size: usize,
	/// Configured maximum number of blocks in the queue
	pub max_queue_size: usize,
	/// Configured maximum number of bytes to use
	pub max_mem_use: usize,
	/// Heap memory used in bytes
	pub mem_used: usize,
}
