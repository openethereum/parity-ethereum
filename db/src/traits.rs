//! Ethcore database trait

use ipc::BinaryConvertable;
use std::mem;
use ipc::binary::BinaryConvertError;
use std::collections::VecDeque;

pub type TransactionHandle = u32;
pub type IteratorHandle = u32;

#[derive(Binary)]
pub struct KeyValue {
	pub key: Vec<u8>,
	pub value: Vec<u8>,
}

#[derive(Debug, Binary)]
pub enum Error {
	AlreadyOpen,
	IsClosed,
	RocksDb(String),
	TransactionUnknown,
	IteratorUnknown,
	UncommitedTransactions,
}

/// Database configuration
#[derive(Binary)]
pub struct DatabaseConfig {
	/// Optional prefix size in bytes. Allows lookup by partial key.
	pub prefix_size: Option<usize>
}

pub trait DatabaseService {
	/// Opens database in the specified path
	fn open(&self, config: DatabaseConfig, path: String) -> Result<(), Error>;

	/// Closes database
	fn close(&self) -> Result<(), Error>;

	/// Insert a key-value pair in the transaction. Any existing value value will be overwritten.
	fn put(&self, key: &[u8], value: &[u8]) -> Result<(), Error>;

	/// Delete value by key.
	fn delete(&self, key: &[u8]) -> Result<(), Error>;

	/// Insert a key-value pair in the transaction. Any existing value value will be overwritten.
	fn transaction_put(&self, transaction: TransactionHandle, key: &[u8], value: &[u8]) -> Result<(), Error>;

	/// Delete value by key using transaction
	fn transaction_delete(&self, transaction: TransactionHandle, key: &[u8]) -> Result<(), Error>;

	/// Commit transaction to database.
	fn write(&self, tr: TransactionHandle) -> Result<(), Error>;

	/// Initiate new transaction on database
	fn new_transaction(&self) -> TransactionHandle;

	/// Get value by key.
	fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>, Error>;

	/// Get value by partial key. Prefix size should match configured prefix size.
	fn get_by_prefix(&self, prefix: &[u8]) -> Result<Option<Vec<u8>>, Error>;

	/// Check if there is anything in the database.
	fn is_empty(&self) -> Result<bool, Error>;

	/// Get handle to iterate through keys
	fn iter(&self) -> Result<IteratorHandle, Error>;

	/// Next key-value for the the given iterator
	fn iter_next(&self, iterator: IteratorHandle) -> Option<KeyValue>;
}
