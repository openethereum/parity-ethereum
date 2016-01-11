//! Ethereum virtual machine.

pub mod ext;
pub mod evm;
pub mod vmfactory;
pub mod logentry;
pub mod executive;
pub mod params;
#[cfg(feature = "jit" )]
mod jit;

pub use self::evm::{Evm, EvmError, EvmResult};
pub use self::ext::{Ext};
pub use self::logentry::LogEntry;
pub use self::vmfactory::VmFactory;
pub use self::executive::{Executive, ExecutionResult, Externalities, Substate};
pub use self::params::EvmParams;
