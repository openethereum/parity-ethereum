//! Evm input params.
use util::hash::*;
use util::uint::*;
use util::bytes::*;

// TODO: should be a trait, possible to avoid cloning everything from a Transaction(/View).

/// Action (call/create) input params. Everything else should be specified in Externalities.
#[derive(Clone, Debug)]
pub struct ActionParams {
	/// Address of currently executed code.
	pub code_address: Address,
	/// Receive address. Usually equal to code_address,
	/// except when called using CALLCODE.
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
	pub code: Option<Bytes>,
	/// Input data.
	pub data: Option<Bytes>
}

impl ActionParams {
	/// TODO [Gav Wood] Please document me
	pub fn new() -> ActionParams {
		ActionParams {
			code_address: Address::new(),
			address: Address::new(),
			sender: Address::new(),
			origin: Address::new(),
			gas: U256::zero(),
			gas_price: U256::zero(),
			value: U256::zero(),
			code: None,
			data: None
		}
	}
}
