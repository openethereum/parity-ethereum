//!

#![allow(unused_imports, missing_docs)]

mod hbbft_daemon;

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

// The HoneyBadger batch size.
const DEFAULT_BATCH_SIZE: usize = 200;
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
    ///
    pub bind_address: SocketAddr,
    ///
    pub remote_addresses: HashSet<SocketAddr>,
    ///
    pub batch_size: usize,
    ///
    pub txn_gen_count: usize,
    ///
    pub txn_gen_interval: u64,
    ///
    // TODO: Make this a range:
    pub txn_gen_bytes: usize,
    ///
    pub keygen_peer_count: usize,
    ///
    pub output_extra_delay_ms: u64,
}

impl HbbftConfig {
    ///
    pub fn to_hydrabadger(&self) -> HydrabadgerConfig {
        HydrabadgerConfig {
            batch_size: self.batch_size,
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
            batch_size: DEFAULT_BATCH_SIZE,
            txn_gen_count: DEFAULT_TXN_GEN_COUNT,
            txn_gen_interval: DEFAULT_TXN_GEN_INTERVAL,
            txn_gen_bytes: DEFAULT_TXN_GEN_BYTES,
            keygen_peer_count: DEFAULT_KEYGEN_PEER_COUNT,
            output_extra_delay_ms: DEFAULT_OUTPUT_EXTRA_DELAY_MS,
        }
    }
}

impl From<HbbftConfig> for HydrabadgerConfig {
    fn from(cfg: HbbftConfig) ->  HydrabadgerConfig {
        HydrabadgerConfig {
            batch_size: cfg.batch_size,
            txn_gen_count: cfg.txn_gen_count,
            txn_gen_interval: cfg.txn_gen_interval,
            txn_gen_bytes: cfg.txn_gen_bytes,
            keygen_peer_count: cfg.keygen_peer_count,
            output_extra_delay_ms: cfg.output_extra_delay_ms,
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