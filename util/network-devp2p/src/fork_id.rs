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

use std::io::{Error, ErrorKind};

use common_types::{
    BlockNumber,
    engines::params::CommonParams
};
use log::{error, Metadata};
use crc::crc32;
use ethereum_types::H256;

/// A fork identifier as defined by EIP-2124.
#[derive(Debug)]
pub struct ID {
    hash: u32,          // CRC32 checksum of the all fork blocks from genesis.
    next: BlockNumber   // Next upcoming fork block number, 0 if not yet known.
}

impl ID {
    /// Calculates the Ethereum fork ID from the chain_info.
    pub fn new(params: &CommonParams, genesis_hash: &H256, best_block: BlockNumber) -> Self {
        ID::new_inner(
            params,
            genesis_hash,
            best_block
        )
    }

    // Use *_inner to allow testing the IDs without
    // having to simulate an entire blockchain.
    fn new_inner(params: &CommonParams, genesis: &H256, head: BlockNumber) -> Self {
        let mut hash = crc32::checksum_ieee(&genesis[..]);
        let mut next= 0;
        let forks = Filter::get_forks_history(params);
        let _: Vec<_> = forks.into_iter()
            .filter(|fork| *fork <= head)
            .map(|fork| {
                hash = Filter::update_checksum(hash, &fork);
                next = fork;
            })
            .collect();

        ID {
            hash,
            next
        }
    }
}

/// A filter that returns if a fork ID should be rejected or not
/// based on the local chain's status.
pub struct Filter {
    check_sums: Vec<u32>,
    forks: Vec<BlockNumber>,
    head: BlockNumber,
}

impl Filter {
    /// Make new Filter for checking if a fork ID should be rejected to connect or not.
    pub fn new(params: &CommonParams, genesis_hash: H256) -> Self {
        Filter::new_inner(
            0,
            params,
            genesis_hash
        )
    }

    /// Calling inner function is to allow testing it without having to simulate an entire blockchain.
    pub fn new_inner(head: BlockNumber, params: &CommonParams, genesis: H256) -> Self {
        let mut check_sums = Vec::new();
        let forks = Filter::get_forks_history(params);
        let mut hash = crc32::checksum_ieee(&genesis[..]);
        check_sums.push(hash);
        check_sums.extend(forks.iter()
            .map(|fork| {
                hash = Filter::update_checksum(hash, fork);
                hash
            })
        );

        Filter {
            check_sums,
            forks,
            head
        }
    }

    /// Check if a fork block ID should be accepted or rejected to connect.
    pub fn is_valid(&self, id: ID) -> Result<(), Error> {
        // The fork checksum validation ruleset is:
        //   1. If local and remote fork checksum matches, compare local head to FORK_NEXT.
        //        The two nodes are in the same fork state currently.
        //        They might know different future forks, but that's not matter until
        //          the fork actually will happen.
        //      1-1. A remote node announced but remote node does not passed a block and
        //          the block is already passed at local node,
        //          this is invalid because the chains are incompatible.
        //      1-2. Remote node does not announce fork,
        //          Or the fork does not yet passed at local node, then it is valid.
        //   2. If the remote FORK_CSUM is a subset of the local forks set and the
        //      remote FORK_NEXT matches with a fork block number of local forks set,
        //      it is valid.
        //        Remote node is currently syncing. It may diverge in the future,
        //        but at this point we don't have enough information.
        //   3. If the remote FORK_CSUM is a superset of the local forks set and can
        //      be completed with future forks of local set, it is valid.
        //        Local node is currently syncing. It may diverge from in the future,
        //        but at this point we don't have enough information.
        //   4. Reject in all other cases.

        let _ = self.forks.iter().enumerate()
            .filter(|(_i, fork)| &self.head <= fork)
            .map(|(i, _fork)| {
                // First unpassed fork block is found, check if our current state matches
                // the remote checksum (rule #1).
                if self.check_sums[i] == id.hash {
                    // Checksum matches, check if remote future fork block already passed (rule #1-1)
                    if id.next > 0 && self.head >= id.next {
                        return Err(Error::new(ErrorKind::Other, ""))
                    }
                    // Local node does not yet passed a remote fork, valid (rule #1-2)
                    return Ok(())
                }

                // The local and remote nodes are in different forks currently, check if the
                // remote checksum is a subset of our local forks (rule #2).
                let mut j = 0;
                while j < i {
                    j += 1;
                    if self.check_sums[j] == id.hash {
                        // Remote checksum is a subset, validate based on the announced next fork
                        if self.forks[j] != id.next {
                            return Err(Error::new(ErrorKind::Other, ""))
                        }
                        return Ok(())
                    }
                }
                // Remote chain is not a subset of our local one, check if it's a superset by
                // any chance, signalling that we're simply out of sync (rule #3).
                let mut j = i;
                while j < self.check_sums.len() {
                    j += 1;
                    if self.check_sums[j] == id.hash {
                        // Yay, remote checksum is a superset, ignore upcoming forks
                        return Ok(())
                    }
                }
                // No exact, subset or superset match. We are on differing chains, reject.
                return Err(Error::new(ErrorKind::Other, ""))
            }).collect::<Vec<_>>();

        error!(target: "network", "Impossible fork ID validation error: id = {:?}.", id);
        Ok(()) // Something is wrong, accept rather than reject.
    }

    // Calculates the next IEEE CRC32 checksum based on the formula of CRC32(original-blob || fork).
    fn update_checksum(hash: u32, fork: &BlockNumber) -> u32 {
        let blob = fork.to_be_bytes();
        crc32::update(hash, &crc32::IEEE_TABLE, &blob)
    }

    // Get forks history data
    fn get_forks_history(params: &CommonParams) -> Vec<BlockNumber> {
        let mut fork_history = Vec::new();

        fork_history.push(params.eip150_transition);
        fork_history.push(params.eip160_transition);
        fork_history.push(params.eip161abc_transition);
        fork_history.push(params.eip161d_transition);
        fork_history.push(params.eip98_transition);
        fork_history.push(params.eip658_transition);
        fork_history.push(params.eip155_transition);
        fork_history.push(params.eip140_transition);
        fork_history.push(params.eip210_transition);
        fork_history.push(params.eip211_transition);
        fork_history.push(params.eip214_transition);
        fork_history.push(params.eip145_transition);
        fork_history.push(params.eip1052_transition);
        fork_history.push(params.eip1283_transition);
        fork_history.push(params.eip1014_transition);
        fork_history.push(params.eip1706_transition);
        fork_history.push(params.eip1344_transition);
        fork_history.push(params.eip1884_transition);
        fork_history.push(params.eip2028_transition);
        fork_history.push(params.dust_protection_transition);
        fork_history.push(params.wasm_activation_transition);
        fork_history.push(params.kip4_transition);
        fork_history.push(params.kip6_transition);
        fork_history.push(params.max_code_size_transition);
        fork_history.push(params.transaction_permission_contract_transition);

        fork_history.sort();

        fork_history
    }
}

