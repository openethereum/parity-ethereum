//! Interface for Evm externalities.

use util::hash::*;
use util::uint::*;
use util::bytes::*;
use evm::{Schedule, Error};
use env_info::*;

// TODO: replace all u64 with u256
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
	/// 
	/// If contract creation is successfull, return gas_left and contract address,
	/// If depth is too big or transfer value exceeds balance, return None
	/// Otherwise return appropriate `Error`.
	fn create(&mut self, gas: u64, value: &U256, code: &[u8]) -> Result<(u64, Option<Address>), Error>;

	/// Message call.
	/// 
	/// If call is successfull, returns gas left.
	/// otherwise `Error`.
	fn call(&mut self, 
			gas: u64, 
			call_gas: u64, 
			receive_address: &Address, 
			value: &U256, 
			data: &[u8], 
			code_address: &Address, 
			output: &mut [u8]) -> Result<u64, Error>;

	/// Returns code at given address
	fn extcode(&self, address: &Address) -> Vec<u8>;

	/// Creates log entry with given topics and data
	fn log(&mut self, topics: Vec<H256>, data: Bytes);

	/// Should be called when transaction calls `RETURN` opcode.
	/// Returns gas_left if cost of returning the data is not too high.
	fn ret(&mut self, gas: u64, data: &[u8]) -> Result<u64, Error>;

	/// Should be called when contract commits suicide.
	/// Address to which funds should be refunded.
	fn suicide(&mut self, refund_address: &Address);

	/// Returns schedule.
	fn schedule(&self) -> &Schedule;

	/// Returns environment info.
	fn env_info(&self) -> &EnvInfo;
}
