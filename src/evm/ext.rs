//! Interface for Evm externalities.

use util::hash::*;
use util::uint::*;
use evm::{Schedule, Error};
use env_info::*;

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
	/// Returns gas_left and contract address if contract creation was succesfull.
	fn create(&mut self, gas: &U256, value: &U256, code: &[u8]) -> (U256, Option<Address>);

	/// Message call.
	/// 
	/// Returns Err, if we run out of gas.
	/// Otherwise returns call_result which contains gas left 
	/// and true if subcall was successfull.
	fn call(&mut self, 
			gas: &U256, 
			call_gas: &U256, 
			receive_address: &Address, 
			value: &U256, 
			data: &[u8], 
			code_address: &Address, 
			output: &mut [u8]) -> Result<(U256, bool), Error>;

	/// Returns code at given address
	fn extcode(&self, address: &Address) -> Vec<u8>;

	/// Creates log entry with given topics and data
	fn log(&mut self, topics: Vec<H256>, data: &[u8]);

	/// Should be called when transaction calls `RETURN` opcode.
	/// Returns gas_left if cost of returning the data is not too high.
	fn ret(&mut self, gas: &U256, data: &[u8]) -> Result<U256, Error>;

	/// Should be called when contract commits suicide.
	/// Address to which funds should be refunded.
	fn suicide(&mut self, refund_address: &Address);

	/// Returns schedule.
	fn schedule(&self) -> &Schedule;

	/// Returns environment info.
	fn env_info(&self) -> &EnvInfo;
}
