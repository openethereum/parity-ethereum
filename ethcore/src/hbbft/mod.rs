//!

#![allow(unused_imports, missing_docs)]

mod hbbft_daemon;
mod laboratory;

use std::str::FromStr;
use std::collections::HashSet;
use std::net::{SocketAddr, IpAddr, Ipv4Addr};
#[cfg(feature = "unused")]
use sync::{Node, NetworkConfiguration};
// use network;
use hydrabadger::Config as HydrabadgerConfig;

pub use self::hbbft_daemon::{HbbftDaemon, HbbftClientExt};

///
pub const DEFAULT_HBBFT_PORT: u16 = 5900;

// The number of random transactions to generate per interval.
const DEFAULT_TXN_GEN_COUNT: usize = 5;
// The interval between randomly generated transactions.
const DEFAULT_TXN_GEN_INTERVAL: u64 = 5000;
// The number of bytes per randomly generated transaction.
const DEFAULT_TXN_GEN_BYTES: usize = 2;
// The minimum number of peers needed to spawn a HB instance.
const DEFAULT_KEYGEN_PEER_COUNT: usize = 2;
// Causes the primary hydrabadger thread to sleep after every batch. Used for
// debugging.
const DEFAULT_OUTPUT_EXTRA_DELAY_MS: u64 = 0;

///
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HbbftConfig {
    /// Our bind address.
    pub bind_address: SocketAddr,
    /// Remote nodes to connect to upon startup.
    pub remote_addresses: HashSet<SocketAddr>,
    /// The time interval to wait between contribution proposal attempts.
    pub contribution_delay_ms: u64,
    /// The maximum batch size used as a starting point when determining
    /// whether or not it's time to propose a contribution of transactions.
    /// Each `contribution_delay_ms` interval, the minimum number of
    /// transactions required is reduced by half (until it reaches 1, where it
    /// remains indefinitely).
    pub contribution_size_max_log2: usize,
    ///
    pub txn_gen_count: usize,
    ///
    pub txn_gen_interval: u64,
    ///
    // TODO: Make this a range:
    pub txn_gen_bytes: usize,
    /// The minimum number of peers needed to begin key generation and start
    /// a hbbft network.
    pub keygen_peer_count: usize,
    ///
    pub output_extra_delay_ms: u64,
}

impl HbbftConfig {
    ///
    pub fn to_hydrabadger(&self) -> HydrabadgerConfig {
        HydrabadgerConfig {
			start_epoch: 1,
            txn_gen_count: self.txn_gen_count,
            txn_gen_interval: self.txn_gen_interval,
            txn_gen_bytes: self.txn_gen_bytes,
            keygen_peer_count: self.keygen_peer_count,
            output_extra_delay_ms: self.output_extra_delay_ms,
        }
    }
}

impl Default for HbbftConfig {
    fn default() -> HbbftConfig {
        HbbftConfig {
            bind_address: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), DEFAULT_HBBFT_PORT),
            remote_addresses: HashSet::new(),
            contribution_delay_ms: 100,
            contribution_size_max_log2: 16,
            txn_gen_count: DEFAULT_TXN_GEN_COUNT,
            txn_gen_interval: DEFAULT_TXN_GEN_INTERVAL,
            txn_gen_bytes: DEFAULT_TXN_GEN_BYTES,
            keygen_peer_count: DEFAULT_KEYGEN_PEER_COUNT,
            output_extra_delay_ms: DEFAULT_OUTPUT_EXTRA_DELAY_MS,
        }
    }
}


/// Creates a list of socket addresses using defined boot and reserved nodes .
#[cfg(feature = "unused")]
#[allow(dead_code)]
pub fn to_peer_addrs(net_conf: &NetworkConfiguration) -> HashSet<SocketAddr> {
    net_conf.boot_nodes.iter().chain(net_conf.reserved_nodes.iter()).filter_map(|node_str| {
        Node::from_str(node_str).ok().map(|node| node.endpoint.address)
    }).collect()
}
