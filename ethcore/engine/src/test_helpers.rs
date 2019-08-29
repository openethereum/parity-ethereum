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

//! Test helpers for engine related tests

use std::sync::Arc;

use ethereum_types::{Address, H256};
use ethkey::{Password, Signature};
use log::warn;
use accounts::{self, AccountProvider, SignError};

use crate::signer::EngineSigner;

impl EngineSigner for (Arc<AccountProvider>, Address, Password) {
	fn sign(&self, hash: H256) -> Result<Signature, ethkey::Error> {
		match self.0.sign(self.1, Some(self.2.clone()), hash) {
			Err(SignError::NotUnlocked) => unreachable!(),
			Err(SignError::NotFound) => Err(ethkey::Error::InvalidAddress),
			Err(SignError::SStore(accounts::Error::EthKey(err))) => Err(err),
			Err(SignError::SStore(accounts::Error::EthKeyCrypto(err))) => {
				warn!("Low level crypto error: {:?}", err);
				Err(ethkey::Error::InvalidSecret)
			},
			Err(SignError::SStore(err)) => {
				warn!("Error signing for engine: {:?}", err);
				Err(ethkey::Error::InvalidSignature)
			},
			Ok(ok) => Ok(ok),
		}
	}

	fn address(&self) -> Address {
		self.1
	}
}
