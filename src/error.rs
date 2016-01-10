//! General error types for use in ethcore.

use util::*;

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
	TooManyUncles,
	UncleWrongGeneration,
	ExtraDataOutOfBounds(OutOfBounds<usize>),
	InvalidSealArity(Mismatch<usize>),
}

#[derive(Debug)]
pub enum ImportError {
	Bad(BlockError),
	AlreadyInChain,
	AlreadyQueued,
}

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
