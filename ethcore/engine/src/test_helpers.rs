// Copyright 2015-2020 Parity Technologies (UK) Ltd.
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
use ethkey::Password;
use parity_crypto::publickey::{Public, Signature, Error};
use log::warn;
use accounts::{self, AccountProvider, SignError};

use crate::signer::EngineSigner;

impl EngineSigner for (Arc<AccountProvider>, Address, Password) {
	fn sign(&self, hash: H256) -> Result<Signature, Error> {
		match self.0.sign(self.1, Some(self.2.clone()), hash) {
			Err(SignError::NotUnlocked) => unreachable!(),
			Err(SignError::NotFound) => Err(Error::InvalidAddress),
			Err(SignError::SStore(accounts::Error::EthCrypto(err))) => Err(Error::Custom(err.to_string())),
			Err(SignError::SStore(accounts::Error::EthPublicKeyCrypto(err))) => {
				warn!("Low level crypto error: {:?}", err);
				Err(Error::InvalidSecretKey)
			},
			Err(SignError::SStore(err)) => {
				warn!("Error signing for engine: {:?}", err);
				Err(Error::InvalidSignature)
			},
			Ok(ok) => Ok(ok),
		}
	}

	fn decrypt(&self, auth_data: &[u8], cipher: &[u8]) -> Result<Vec<u8>, Error> {
		self.0.decrypt(self.1, None, auth_data, cipher).map_err(|e| {
			warn!("Unable to decrypt message: {:?}", e);
			Error::InvalidMessage
		})
	}

	fn address(&self) -> Address {
		self.1
	}

	fn public(&self) -> Option<Public> {
		self.0.account_public(self.1, &self.2).ok()
	}
}
