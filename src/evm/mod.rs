//! Ethereum virtual machine.

pub mod env;
pub mod runtime_data;
pub mod evm;
#[cfg(feature = "jit" )]
pub mod jit;

pub use self::evm::{Evm, ReturnCode};
pub use self::env::Env;
pub use self::runtime_data::RuntimeData;

#[cfg(feature = "jit" )]
pub use self::jit::JitEvm;
