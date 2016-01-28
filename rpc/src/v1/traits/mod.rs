//! Ethereum rpc interfaces.

macro_rules! rpc_unimplemented {
	() => (Err(Error::internal_error()))
}

pub mod web3;
pub mod eth;
pub mod net;

pub use self::web3::Web3;
pub use self::eth::{Eth, EthFilter};
pub use self::net::Net;
