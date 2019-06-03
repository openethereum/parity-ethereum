//! A registry for engines not defined in `ethcore` itself.
//!
//! External crates can implement the `Engine` trait and register it using `inventory::submit!`:
//! ```
//! # extern crate inventory;
//! # extern crate serde_json;
//! # extern crate common_types as types;
//! #
//! # use std::sync::Arc;
//! #
//! # use ethcore::engines::{Engine, EthEngine, ForkChoice};
//! # use ethcore::engines::registry::EnginePlugin;
//! # use ethcore::error::Error;
//! # use types::header::{Header, ExtendedHeader};
//! # use ethcore::machine::EthereumMachine;
//! #
//! # fn main() {}
//! #
//! pub struct MyEngine {
//!		params: serde_json::Value,
//!		machine: EthereumMachine,
//! }
//!
//! impl Engine<EthereumMachine> for MyEngine {
//! 	fn name(&self) -> &str { "MyEngine" }
//! 	fn machine(&self) -> &EthereumMachine { unimplemented!() }
//! 	fn verify_local_seal(&self, _header: &Header) -> Result<(), Error> { Ok(()) }
//! 	fn fork_choice(&self, new: &ExtendedHeader, current: &ExtendedHeader) -> ForkChoice {
//!			unimplemented!()
//!		}
//! }
//!
//! impl MyEngine {
//! 	fn new(params: &serde_json::Value, machine: EthereumMachine)
//!			-> Result<Arc<EthEngine>, Box<Error>>
//!		{
//!			Ok(Arc::new(MyEngine { params: params.clone(), machine }))
//! 	}
//! }
//!
//! inventory::submit!(EnginePlugin("MyEngine", MyEngine::new));
//! ```

use std::sync::Arc;

use serde_json::Value;

use engines::EthEngine;
use error::Error;
use machine::EthereumMachine;

/// A name and constructor for an engine implementation.
pub struct EnginePlugin(pub &'static str, pub EngineConstructor);

/// A constructor that instantiates an engine from its chain spec parameters.
pub type EngineConstructor = fn(params: &Value, machine: EthereumMachine) -> Result<Arc<EthEngine>, Box<Error>>;

inventory::collect!(EnginePlugin);

