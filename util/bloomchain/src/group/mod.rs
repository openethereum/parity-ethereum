//! Bloom grouping.
//! 
//! Optimization gathering together blooms that are in the same index and are likely to be retrived together.

mod bridge;
mod chain;
mod database;
mod group;
mod position;

pub use self::bridge::GroupDatabaseBridge;
pub use self::chain::BloomGroupChain;

pub use self::database::BloomGroupDatabase;
pub use self::group::BloomGroup;
pub use self::position::GroupPosition;
