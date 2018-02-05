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

use {VerifiedTransaction};

/// Transaction verification.
///
/// Verifier is responsible to decide if the transaction should even be considered for pool inclusion.
pub trait Verifier<U> {
	/// Verification error.
	type Error;

	/// Verified transaction.
	type VerifiedTransaction: VerifiedTransaction;

	/// Verifies a `UnverifiedTransaction` and produces `VerifiedTransaction` instance.
	fn verify_transaction(&self, tx: U) -> Result<Self::VerifiedTransaction, Self::Error>;
}
