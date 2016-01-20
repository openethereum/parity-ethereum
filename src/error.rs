//! General error types for use in ethcore.

use util::*;
use header::BlockNumber;
use basic_types::LogBloom;

#[derive(Debug, PartialEq, Eq)]
/// TODO [Gav Wood] Please document me
pub struct Mismatch<T: fmt::Debug> {
	/// TODO [Gav Wood] Please document me
	pub expected: T,
	/// TODO [Gav Wood] Please document me
	pub found: T,
}

#[derive(Debug, PartialEq, Eq)]
/// TODO [Gav Wood] Please document me
pub struct OutOfBounds<T: fmt::Debug> {
	/// TODO [Gav Wood] Please document me
	pub min: Option<T>,
	/// TODO [Gav Wood] Please document me
	pub max: Option<T>,
	/// TODO [Gav Wood] Please document me
	pub found: T,
}

/// Result of executing the transaction.
#[derive(PartialEq, Debug)]
pub enum ExecutionError {
	/// Returned when there gas paid for transaction execution is
	/// lower than base gas required.
	/// TODO [Gav Wood] Please document me
	NotEnoughBaseGas { 
		/// TODO [Gav Wood] Please document me
		required: U256, 
		/// TODO [Gav Wood] Please document me
		got: U256
	},
	/// Returned when block (gas_used + gas) > gas_limit.
	/// 
	/// If gas =< gas_limit, upstream may try to execute the transaction
	/// in next block.
	BlockGasLimitReached { 
		/// TODO [Gav Wood] Please document me
		gas_limit: U256,
		/// TODO [Gav Wood] Please document me
		gas_used: U256,
		/// TODO [Gav Wood] Please document me
		gas: U256 
	},
	/// Returned when transaction nonce does not match state nonce.
	InvalidNonce { 
		/// TODO [Gav Wood] Please document me
		expected: U256,
		/// TODO [Gav Wood] Please document me
		got: U256
	},
	/// Returned when cost of transaction (value + gas_price * gas) exceeds 
	/// current sender balance.
	NotEnoughCash { 
		/// TODO [Gav Wood] Please document me
		required: U512,
		/// TODO [Gav Wood] Please document me
		got: U512
	},
	/// Returned when internal evm error occurs.
	Internal
}

#[derive(Debug)]
/// TODO [Gav Wood] Please document me
pub enum TransactionError {
	/// TODO [Gav Wood] Please document me
	InvalidGasLimit(OutOfBounds<U256>),
}

#[derive(Debug, PartialEq, Eq)]
/// TODO [arkpar] Please document me
pub enum BlockError {
	/// TODO [Gav Wood] Please document me
	TooManyUncles(OutOfBounds<usize>),
	/// TODO [Gav Wood] Please document me
	UncleWrongGeneration,
	/// TODO [Gav Wood] Please document me
	ExtraDataOutOfBounds(OutOfBounds<usize>),
	/// TODO [arkpar] Please document me
	InvalidSealArity(Mismatch<usize>),
	/// TODO [arkpar] Please document me
	TooMuchGasUsed(OutOfBounds<U256>),
	/// TODO [arkpar] Please document me
	InvalidUnclesHash(Mismatch<H256>),
	/// TODO [arkpar] Please document me
	UncleTooOld(OutOfBounds<BlockNumber>),
	/// TODO [arkpar] Please document me
	UncleIsBrother(OutOfBounds<BlockNumber>),
	/// TODO [arkpar] Please document me
	UncleInChain(H256),
	/// TODO [arkpar] Please document me
	UncleParentNotInChain(H256),
	/// TODO [arkpar] Please document me
	InvalidStateRoot(Mismatch<H256>),
	/// TODO [arkpar] Please document me
	InvalidGasUsed(Mismatch<U256>),
	/// TODO [arkpar] Please document me
	InvalidTransactionsRoot(Mismatch<H256>),
	/// TODO [arkpar] Please document me
	InvalidDifficulty(Mismatch<U256>),
	/// TODO [arkpar] Please document me
	InvalidGasLimit(OutOfBounds<U256>),
	/// TODO [arkpar] Please document me
	InvalidReceiptsStateRoot(Mismatch<H256>),
	/// TODO [arkpar] Please document me
	InvalidTimestamp(OutOfBounds<u64>),
	/// TODO [arkpar] Please document me
	InvalidLogBloom(Mismatch<LogBloom>),
	/// TODO [arkpar] Please document me
	InvalidEthashDifficulty(Mismatch<U256>),
	/// TODO [arkpar] Please document me
	InvalidBlockNonce(Mismatch<H256>),
	/// TODO [arkpar] Please document me
	InvalidParentHash(Mismatch<H256>),
	/// TODO [arkpar] Please document me
	InvalidNumber(OutOfBounds<BlockNumber>),
	/// TODO [arkpar] Please document me
	UnknownParent(H256),
	/// TODO [Gav Wood] Please document me
	UnknownUncleParent(H256),
}

#[derive(Debug)]
/// TODO [arkpar] Please document me
pub enum ImportError {
	/// TODO [arkpar] Please document me
	Bad(Option<Error>),
	/// TODO [arkpar] Please document me
	AlreadyInChain,
	/// TODO [arkpar] Please document me
	AlreadyQueued,
}

impl From<Error> for ImportError {
	fn from(err: Error) -> ImportError {
		ImportError::Bad(Some(err))
	}
}

/// Result of import block operation.
pub type ImportResult = Result<(), ImportError>;

#[derive(Debug)]
/// General error type which should be capable of representing all errors in ethcore.
pub enum Error {
	/// TODO [Gav Wood] Please document me
	Util(UtilError),
	/// TODO [Gav Wood] Please document me
	Block(BlockError),
	/// TODO [Gav Wood] Please document me
	UnknownEngineName(String),
	/// TODO [Gav Wood] Please document me
	Execution(ExecutionError),
	/// TODO [Gav Wood] Please document me
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

impl From<UtilError> for Error {
	fn from(err: UtilError) -> Error {
		Error::Util(err)
	}
}

impl From<IoError> for Error {
	fn from(err: IoError) -> Error {
		Error::Util(From::from(err))
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
