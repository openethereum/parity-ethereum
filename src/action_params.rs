//! Evm input params.
use util::hash::*;
use util::uint::*;
use util::bytes::*;

// TODO: should be a trait, possible to avoid cloning everything from a Transaction(/View).

/// Action (call/create) input params. Everything else should be specified in Externalities.
#[derive(Clone, Debug)]
pub struct ActionParams {
	/// Address of currently executed code.
	pub address: Address,
	/// Sender of current part of the transaction.
	pub sender: Address,
	/// Transaction initiator.
	pub origin: Address,
	/// Gas paid up front for transaction execution
	pub gas: U256,
	/// Gas price.
	pub gas_price: U256,
	/// Transaction value.
	pub value: U256,
	/// Code being executed.
	pub code: Bytes,
	/// Input data.
	pub data: Bytes
}

impl ActionParams {
	pub fn new() -> ActionParams {
		ActionParams {
			address: Address::new(),
			sender: Address::new(),
			origin: Address::new(),
			gas: U256::zero(),
			gas_price: U256::zero(),
			value: U256::zero(),
			code: vec![],
			data: vec![],
		}
	}
}
