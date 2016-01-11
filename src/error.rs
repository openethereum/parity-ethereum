//! General error types for use in ethcore.

use util::*;
use header::BlockNumber;

#[derive(Debug)]
pub struct Mismatch<T: fmt::Debug> {
	pub expected: T,
	pub found: T,
}

#[derive(Debug)]
pub struct OutOfBounds<T: fmt::Debug> {
	pub min: T,
	pub max: T,
	pub found: T,
}

#[derive(Debug)]
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
}

impl From<BlockError> for Error {
	fn from(err: BlockError) -> Error {
		Error::Block(err)
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
