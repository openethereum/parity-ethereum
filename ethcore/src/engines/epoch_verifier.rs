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

// Epoch verifiers.

use error::Error;
use header::Header;

/// Verifier for all blocks within an epoch with self-contained state.
///
/// See docs on `Engine` relating to proving functions for more details.
pub trait EpochVerifier: Send + Sync {
	/// Get the epoch number.
	fn epoch_number(&self) -> u64;

	/// Lightly verify the next block header.
	/// This may not be a header belonging to a different epoch.
	fn verify_light(&self, header: &Header) -> Result<(), Error>;

	/// Perform potentially heavier checks on the next block header.
	fn verify_heavy(&self, header: &Header) -> Result<(), Error> {
		self.verify_light(header)
	}
}

/// Special "no-op" verifier for stateless, epoch-less engines.
pub struct NoOp;

impl EpochVerifier for NoOp {
	fn epoch_number(&self) -> u64 { 0 }
	fn verify_light(&self, _header: &Header) -> Result<(), Error> { Ok(()) }
}
