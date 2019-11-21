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

use crate::fork_id::ID;
use blockchain::BlockChain;

/// EthEntry is the "eth" db entry for advertising eth protocol on the discovery network.
pub struct EthEntry {
    fork_id: ID,    // Fork identifier defined in EIP-2124
    rest: String    // For forward compatibility ignore additional fields
}

impl EthEntry {
    /// Get current ENR entry object.
    pub fn get_current(chain: &BlockChain) -> EthEntry {
        let fork_id = ID::new(chain);
        EthEntry {
            fork_id,
            rest: r#"rlp:"tail""#.to_owned()
        }
    }
}
