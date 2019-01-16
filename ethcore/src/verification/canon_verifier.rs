// Copyright 2015-2019 Parity Technologies (UK) Ltd.
// This file is part of Parity Ethereum.

// Parity Ethereum is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity Ethereum is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity Ethereum.  If not, see <http://www.gnu.org/licenses/>.

//! Canonical verifier.

use client::{BlockInfo, CallContract};
use engines::EthEngine;
use error::Error;
use types::header::Header;
use super::Verifier;
use super::verification;

/// A canonial verifier -- this does full verification.
pub struct CanonVerifier;

impl<C: BlockInfo + CallContract> Verifier<C> for CanonVerifier {
	fn verify_block_family(
		&self,
		header: &Header,
		parent: &Header,
		engine: &EthEngine,
		do_full: Option<verification::FullFamilyParams<C>>,
	) -> Result<(), Error> {
		verification::verify_block_family(header, parent, engine, do_full)
	}

	fn verify_block_final(&self, expected: &Header, got: &Header) -> Result<(), Error> {
		verification::verify_block_final(expected, got)
	}

	fn verify_block_external(&self, header: &Header, engine: &EthEngine) -> Result<(), Error> {
		engine.verify_block_external(header)
	}
}
