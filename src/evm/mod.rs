//! Ethereum virtual machine.

pub mod ext;
pub mod evm;
pub mod vmfactory;
//pub mod logentry;
pub mod executive;
pub mod params;
pub mod schedule;
#[cfg(feature = "jit" )]
mod jit;

pub use self::evm::{Evm, EvmError, EvmResult};
pub use self::ext::{Ext};
//pub use self::logentry::LogEntry;
pub use self::vmfactory::VmFactory;
// TODO: reduce this to absolutely necessary things
pub use self::executive::{Executive, ExecutionResult, Externalities, Substate, OutputPolicy};
pub use self::params::EvmParams;
pub use self::schedule::Schedule;
