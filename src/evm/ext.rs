//! Interface for Evm externalities.

use util::hash::*;
use util::uint::*;
use util::bytes::*;

pub trait Ext {
	/// Returns a value for given key.
	fn sload(&self, key: &H256) -> H256;

	/// Stores a value for given key.
	fn sstore(&mut self, key: H256, value: H256);

	/// Returns address balance.
	fn balance(&self, address: &Address) -> U256;

	/// Returns the hash of one of the 256 most recent complete blocks.
	fn blockhash(&self, number: &U256) -> H256;

	/// Creates new contract.
	/// If contract creation is successfull, 
	/// returns new contract address and gas left,
	/// otherwise `None`.
	fn create(&mut self, gas: u64, endowment: &U256, code: &[u8]) -> Option<(Address, u64)>;

	/// Message call.
	/// If call is successfull, returns call output and gas left.
	/// otherwise `None`.
	fn call(&mut self, gas: u64, call_gas: u64, receive_address: &Address, value: &U256, data: &[u8], code_address: &Address) -> Option<(Vec<u8>, u64)>;

	/// Returns code at given address
	fn extcode(&self, address: &Address) -> Vec<u8>;

	/// Creates log entry with given topics and data
	fn log(&mut self, topics: Vec<H256>, data: Bytes);
}
