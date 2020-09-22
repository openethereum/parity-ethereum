// Copyright 2015-2020 Parity Technologies (UK) Ltd.
// This file is part of OpenEthereum.

// OpenEthereum is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// OpenEthereum is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with OpenEthereum.  If not, see <http://www.gnu.org/licenses/>.

//! Web3 rpc implementation.
use ethereum_types::H256;
use hash::keccak;
use jsonrpc_core::Result;
use v1::{traits::Web3, types::Bytes};
use version::version;

/// Web3 rpc implementation.
#[derive(Default)]
pub struct Web3Client;

impl Web3 for Web3Client {
    fn client_version(&self) -> Result<String> {
        Ok(version().to_owned().replacen("/", "//", 1))
    }

    fn sha3(&self, data: Bytes) -> Result<H256> {
        Ok(keccak(&data.0))
    }
}
