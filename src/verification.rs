use util::uint::*;
use util::hash::*;
use util::rlp::*;
use util::sha3::Hashable;
use util::triehash::ordered_trie_root;
use header::Header;
use client::BlockNumber;

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
	ExtraDataTooBig,
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
	InvalidUnclesHash {
		expected: H256,
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
		expected: H256,
		got: H256,
	},
	InvalidDifficulty,
	InvalidGasLimit,
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
			expected: expected_root.clone(),
			got: transactions_root.clone(),
		});
	}
	let expected_uncles = block.at(2).as_raw().sha3();
	if &expected_uncles != uncles_hash {
		return Err(BlockVerificationError::InvalidUnclesHash {
			expected: expected_uncles.clone(),
			got: uncles_hash.clone(),
		});
	}
	Ok(())
}

