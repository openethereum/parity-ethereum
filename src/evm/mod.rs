//! Ethereum virtual machine.

pub mod ext;
pub mod evm;
pub mod vmfactory;
//pub mod logentry;
pub mod schedule;
#[cfg(feature = "jit" )]
mod jit;

// TODO: Error -> evm::Error, Result -> evm::Result
pub use self::evm::{Evm, Error, Result};
pub use self::ext::Ext;
// TODO: VmFactory -> evm::Factory
// TODO: module rename vmfactory -> factory
pub use self::vmfactory::VmFactory;
pub use self::schedule::Schedule;
