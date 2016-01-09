use util::*;
use header::Header;
use client::BlockNumber;
use engine::{Engine, VerificationMode};
use views::BlockView;

#[derive(Debug)]
pub struct VerificationError {
	pub block: Option<Bytes>,
	pub error: VerificationErrorOption,
}

impl VerificationError {
	pub fn block(error: BlockVerificationError, block: Option<Bytes>) -> VerificationError {
		VerificationError {
			block: block,
			error: VerificationErrorOption::Block(error),
		}
	}
	pub fn transaction(error: TransactionVerificationError, block: Option<Bytes>) -> VerificationError {
		VerificationError {
			block: block,
			error: VerificationErrorOption::Transaction(error),
		}
	}
}

#[derive(Debug)]
pub enum VerificationErrorOption {
	Transaction(TransactionVerificationError),
	Block(BlockVerificationError),
}

#[derive(Debug)]
pub enum TransactionVerificationError {
	OutOfGasBase,
	OutOfGasIntrinsic,
	NotEnoughCash,
	GasPriceTooLow,
	BlockGasLimitReached,
	FeeTooSmall,
	TooMuchGasUsed {
		used: U256,
		limit: U256
	},
	InvalidSignature,
	InvalidTransactionFormat,
}

#[derive(Debug)]
pub enum BlockVerificationError {
	TooMuchGasUsed {
		used: U256,
		limit: U256,
	},
	InvalidBlockFormat,
	ExtraDataTooBig {
		required: U256,
		got: U256,
	},
	InvalidUnclesHash {
		required: H256,
		got: H256,
	},
	TooManyUncles,
	UncleTooOld,
	UncleIsBrother,
	UncleInChain,
	UncleParentNotInChain,
	InvalidStateRoot,
	InvalidGasUsed,
	InvalidTransactionsRoot {
		required: H256,
		got: H256,
	},
	InvalidDifficulty {
		required: U256,
		got: U256,
	},
	InvalidGasLimit {
		min: U256,
		max: U256,
		got: U256,
	},
	InvalidReceiptsStateRoot,
	InvalidTimestamp,
	InvalidLogBloom,
	InvalidNonce,
	InvalidBlockHeaderItemCount,
	InvalidBlockNonce,
	InvalidParentHash,
	InvalidUncleParentHash,
	InvalidNumber,
	BlockNotFound,
	UnknownParent,
}


pub fn verify_header(header: &Header) -> Result<(), BlockVerificationError> {
	if header.number > From::from(BlockNumber::max_value()) {
		return Err(BlockVerificationError::InvalidNumber)
	}
	if header.gas_used > header.gas_limit {
		return Err(BlockVerificationError::TooMuchGasUsed {
			used: header.gas_used,
			limit: header.gas_limit,
		});
	}
	Ok(())
}

pub fn verify_parent(header: &Header, parent: &Header) -> Result<(), BlockVerificationError> {
	if !header.parent_hash.is_zero() && parent.hash() != header.parent_hash {
		return Err(BlockVerificationError::InvalidParentHash)
	}
	if header.timestamp <= parent.timestamp {
		return Err(BlockVerificationError::InvalidTimestamp)
	}
	if header.number <= parent.number {
		return Err(BlockVerificationError::InvalidNumber)
	}
	Ok(())
}

pub fn verify_block_integrity(block: &[u8], transactions_root: &H256, uncles_hash: &H256) -> Result<(), BlockVerificationError> {
	let block = Rlp::new(block);
	let tx = block.at(1);
	let expected_root = ordered_trie_root(tx.iter().map(|r| r.as_raw().to_vec()).collect()); //TODO: get rid of vectors here
	if &expected_root != transactions_root {
		return Err(BlockVerificationError::InvalidTransactionsRoot {
			required: expected_root.clone(),
			got: transactions_root.clone(),
		});
	}
	let expected_uncles = block.at(2).as_raw().sha3();
	if &expected_uncles != uncles_hash {
		return Err(BlockVerificationError::InvalidUnclesHash {
			required: expected_uncles.clone(),
			got: uncles_hash.clone(),
		});
	}
	Ok(())
}

pub fn verify_block_basic(bytes: &[u8], parent: &Header, engine: &mut Engine) -> Result<(), BlockVerificationError> {
	let block = BlockView::new(bytes);
	let header = block.header();
	try!(verify_header(&header));
	try!(verify_parent(&header, parent));
	try!(verify_block_integrity(bytes, &header.transactions_root, &header.uncles_hash));

	Ok(())
}

pub fn verify_block_unordered(block: &[u8]) -> Result<(), BlockVerificationError> {
	Ok(())
}
