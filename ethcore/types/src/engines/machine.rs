// todo: module header

use ethereum_types::Address;

use crate::receipt;

/// Type alias for a function we can make calls through synchronously.
/// Returns the call result and state proof for each call.
pub type Call<'a> = dyn Fn(Address, Vec<u8>) -> Result<(Vec<u8>, Vec<Vec<u8>>), String> + 'a;

/// Request for auxiliary data of a block.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AuxiliaryRequest {
	/// Needs the body.
	Body,
	/// Needs the receipts.
	Receipts,
	/// Needs both body and receipts.
	Both,
}

/// Auxiliary data fetcher for an Ethereum machine. In Ethereum-like machines
/// there are two kinds of auxiliary data: bodies and receipts.
#[derive(Default, Clone)]
pub struct AuxiliaryData<'a> {
	/// The full block bytes, including the header.
	pub bytes: Option<&'a [u8]>,
	/// The block receipts.
	pub receipts: Option<&'a [receipt::Receipt]>,
}
