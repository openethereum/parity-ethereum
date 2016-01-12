//! General error types for use in ethcore.

use util::*;
use header::BlockNumber;

#[derive(Debug, PartialEq, Eq)]
pub struct Mismatch<T: fmt::Debug> {
	pub expected: T,
	pub found: T,
}

#[derive(Debug, PartialEq, Eq)]
pub struct OutOfBounds<T: fmt::Debug> {
	pub min: Option<T>,
	pub max: Option<T>,
	pub found: T,
}

/// Result of executing the transaction.
#[derive(PartialEq, Debug)]
pub enum ExecutionError {
	/// Returned when block (gas_used + gas) > gas_limit.
	/// 
	/// If gas =< gas_limit, upstream may try to execute the transaction
	/// in next block.
	BlockGasLimitReached { gas_limit: U256, gas_used: U256, gas: U256 },
	/// Returned when transaction nonce does not match state nonce.
	InvalidNonce { expected: U256, is: U256 },
	/// Returned when cost of transaction (value + gas_price * gas) exceeds 
	/// current sender balance.
	NotEnoughCash { required: U256, is: U256 },
	/// Returned when internal evm error occurs.
	Internal
}

#[derive(Debug)]
pub enum TransactionError {
	InvalidGasLimit(OutOfBounds<U256>),
}

#[derive(Debug, PartialEq, Eq)]
pub enum BlockError {
	TooManyUncles(OutOfBounds<usize>),
	UncleWrongGeneration,
	ExtraDataOutOfBounds(OutOfBounds<usize>),
	InvalidSealArity(Mismatch<usize>),
	TooMuchGasUsed(OutOfBounds<U256>),
	InvalidUnclesHash(Mismatch<H256>),
	UncleTooOld(OutOfBounds<BlockNumber>),
	UncleIsBrother(OutOfBounds<BlockNumber>),
	UncleInChain(H256),
	UncleParentNotInChain(H256),
	InvalidStateRoot,
	InvalidGasUsed,
	InvalidTransactionsRoot(Mismatch<H256>),
	InvalidDifficulty(Mismatch<U256>),
	InvalidGasLimit(OutOfBounds<U256>),
	InvalidReceiptsStateRoot,
	InvalidTimestamp(OutOfBounds<u64>),
	InvalidLogBloom,
	InvalidBlockNonce,
	InvalidParentHash(Mismatch<H256>),
	InvalidNumber(OutOfBounds<BlockNumber>),
	UnknownParent(H256),
	UnknownUncleParent(H256),
}

#[derive(Debug)]
pub enum ImportError {
	Bad(Error),
	AlreadyInChain,
	AlreadyQueued,
}

impl From<Error> for ImportError {
	fn from(err: Error) -> ImportError {
		ImportError::Bad(err)
	}
}

/// Result of import block operation.
pub type ImportResult = Result<(), ImportError>;

#[derive(Debug)]
/// General error type which should be capable of representing all errors in ethcore.
pub enum Error {
	Util(UtilError),
	Block(BlockError),
	UnknownEngineName(String),
	Execution(ExecutionError),
	Transaction(TransactionError),
}

impl From<TransactionError> for Error {
	fn from(err: TransactionError) -> Error {
		Error::Transaction(err)
	}
}

impl From<BlockError> for Error {
	fn from(err: BlockError) -> Error {
		Error::Block(err)
	}
}

impl From<ExecutionError> for Error {
	fn from(err: ExecutionError) -> Error {
		Error::Execution(err)
	}
}

impl From<CryptoError> for Error {
	fn from(err: CryptoError) -> Error {
		Error::Util(UtilError::Crypto(err))
	}
}

impl From<DecoderError> for Error {
	fn from(err: DecoderError) -> Error {
		Error::Util(UtilError::Decoder(err))
	}
}

// TODO: uncomment below once https://github.com/rust-lang/rust/issues/27336 sorted.
/*#![feature(concat_idents)]
macro_rules! assimilate {
    ($name:ident) => (
		impl From<concat_idents!($name, Error)> for Error {
			fn from(err: concat_idents!($name, Error)) -> Error {
				Error:: $name (err)
			}
		}
    )
}
assimilate!(FromHex);
assimilate!(BaseData);*/
