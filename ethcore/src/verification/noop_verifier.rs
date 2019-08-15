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

//! No-op verifier.

use call_contract::CallContract;
use client_traits::BlockInfo;
use engine::Engine;
use types::{
	header::Header,
	errors::EthcoreError as Error
};
use super::{verification, Verifier};

/// A no-op verifier -- this will verify everything it's given immediately.
#[allow(dead_code)]
pub struct NoopVerifier;

impl<C: BlockInfo + CallContract> Verifier<C> for NoopVerifier {
	fn verify_block_family(
		&self,
		_: &Header,
		_t: &Header,
		_: &dyn Engine,
		_: Option<verification::FullFamilyParams<C>>
	) -> Result<(), Error> {
		Ok(())
	}

	fn verify_block_final(&self, _expected: &Header, _got: &Header) -> Result<(), Error> {
		Ok(())
	}

	fn verify_block_external(&self, _header: &Header, _engine: &dyn Engine) -> Result<(), Error> {
		Ok(())
	}
}
