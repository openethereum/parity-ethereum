// Copyright 2015-2019 Parity Technologies (UK) Ltd.
// This file is part of Parity Ethereum.

// Parity Ethereum is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity Ethereum is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity Ethereum.  If not, see <http://www.gnu.org/licenses/>.

//! Account state encoding and decoding

use std::collections::HashSet;

use account_db::{AccountDB, AccountDBMut};
use bytes::Bytes;
use common_types::{
	basic_account::BasicAccount,
	snapshot::Progress,
	errors::SnapshotError as Error,
};
use ethereum_types::{H256, U256};
use ethtrie::{TrieDB, TrieDBMut};
use hash_db::HashDB;
use keccak_hash::{KECCAK_EMPTY, KECCAK_NULL_RLP};
use log::{trace, warn};
use parking_lot::RwLock;
use rlp::{RlpStream, Rlp};
use trie_db::{Trie, TrieMut};

// An empty account -- these were replaced with RLP null data for a space optimization in v1.
pub const ACC_EMPTY: BasicAccount = BasicAccount {
	nonce: U256([0, 0, 0, 0]),
	balance: U256([0, 0, 0, 0]),
	storage_root: KECCAK_NULL_RLP,
	code_hash: KECCAK_EMPTY,
	code_version: U256([0, 0, 0, 0]),
};

// whether an encoded account has code and how it is referred to.
#[repr(u8)]
enum CodeState {
	// the account has no code.
	Empty = 0,
	// raw code is encoded.
	Inline = 1,
	// the code is referred to by hash.
	Hash = 2,
}

impl CodeState {
	fn from(x: u8) -> Result<Self, Error> {
		match x {
			0 => Ok(CodeState::Empty),
			1 => Ok(CodeState::Inline),
			2 => Ok(CodeState::Hash),
			_ => Err(Error::UnrecognizedCodeState(x))
		}
	}

	fn raw(self) -> u8 {
		self as u8
	}
}

// walk the account's storage trie, returning a vector of RLP items containing the
// account address hash, account properties and the storage. Each item contains at most `max_storage_items`
// storage records split according to snapshot format definition.
pub fn to_fat_rlps(
	account_hash: &H256,
	acc: &BasicAccount,
	acct_db: &AccountDB,
	used_code: &mut HashSet<H256>,
	first_chunk_size: usize,
	max_chunk_size: usize,
	p: &RwLock<Progress>,
) -> Result<Vec<Bytes>, Error> {
	let db = &(acct_db as &dyn HashDB<_,_>);
	let db = TrieDB::new(db, &acc.storage_root)?;
	let mut chunks = Vec::new();
	let mut db_iter = db.iter()?;
	let mut target_chunk_size = first_chunk_size;
	let mut account_stream = RlpStream::new_list(2);
	let mut leftover: Option<Vec<u8>> = None;
	loop {
		account_stream.append(account_hash);
		let use_short_version = acc.code_version.is_zero();
		match use_short_version {
			true => { account_stream.begin_list(5); },
			false => { account_stream.begin_list(6); },
		}

		account_stream.append(&acc.nonce)
			.append(&acc.balance);

		// [has_code, code_hash].
		if acc.code_hash == KECCAK_EMPTY {
			account_stream.append(&CodeState::Empty.raw()).append_empty_data();
		} else if used_code.contains(&acc.code_hash) {
			account_stream.append(&CodeState::Hash.raw()).append(&acc.code_hash);
		} else {
			match acct_db.get(&acc.code_hash, hash_db::EMPTY_PREFIX) {
				Some(c) => {
					used_code.insert(acc.code_hash.clone());
					account_stream.append(&CodeState::Inline.raw()).append(&&*c);
				}
				None => {
					warn!("code lookup failed during snapshot");
					account_stream.append(&false).append_empty_data();
				}
			}
		}

		if !use_short_version {
			account_stream.append(&acc.code_version);
		}

		account_stream.begin_unbounded_list();
		if account_stream.len() > target_chunk_size {
			// account does not fit, push an empty record to mark a new chunk
			target_chunk_size = max_chunk_size;
			chunks.push(Vec::new());
		}

		if let Some(pair) = leftover.take() {
			if !account_stream.append_raw_checked(&pair, 1, target_chunk_size) {
				return Err(Error::ChunkTooSmall);
			}
		}

		loop {
			if p.read().abort {
				trace!(target: "snapshot", "to_fat_rlps: aborting snapshot");
				return Err(Error::SnapshotAborted);
			}
			match db_iter.next() {
				Some(Ok((k, v))) => {
					let pair = {
						let mut stream = RlpStream::new_list(2);
						stream.append(&k).append(&&*v);
						stream.drain()
					};
					if !account_stream.append_raw_checked(&pair, 1, target_chunk_size) {
						account_stream.finalize_unbounded_list();
						let stream = ::std::mem::replace(&mut account_stream, RlpStream::new_list(2));
						chunks.push(stream.out());
						target_chunk_size = max_chunk_size;
						leftover = Some(pair);
						break;
					}
				},
				Some(Err(e)) => {
					return Err(e.into());
				},
				None => {
					account_stream.finalize_unbounded_list();
					let stream = ::std::mem::replace(&mut account_stream, RlpStream::new_list(2));
					chunks.push(stream.out());
					return Ok(chunks);
				}
			}

		}
	}
}

// decode a fat rlp, and rebuild the storage trie as we go.
// returns the account structure along with its newly recovered code,
// if it exists.
pub fn from_fat_rlp(
	acct_db: &mut AccountDBMut,
	rlp: Rlp,
	mut storage_root: H256,
) -> Result<(BasicAccount, Option<Bytes>), Error> {

	// check for special case of empty account.
	if rlp.is_empty() {
		return Ok((ACC_EMPTY, None));
	}

	let use_short_version = match rlp.item_count()? {
		5 => true,
		6 => false,
		_ => return Err(rlp::DecoderError::RlpIncorrectListLen.into()),
	};

	let nonce = rlp.val_at(0)?;
	let balance = rlp.val_at(1)?;
	let code_state: CodeState = {
		let raw: u8 = rlp.val_at(2)?;
		CodeState::from(raw)?
	};

	// load the code if it exists.
	let (code_hash, new_code) = match code_state {
		CodeState::Empty => (KECCAK_EMPTY, None),
		CodeState::Inline => {
			let code: Bytes = rlp.val_at(3)?;
			let code_hash = acct_db.insert(hash_db::EMPTY_PREFIX, &code);

			(code_hash, Some(code))
		}
		CodeState::Hash => {
			let code_hash = rlp.val_at(3)?;

			(code_hash, None)
		}
	};

	let code_version = if use_short_version {
		U256::zero()
	} else {
		rlp.val_at(4)?
	};

	{
		let mut storage_trie = if storage_root.is_zero() {
			TrieDBMut::new(acct_db, &mut storage_root)
		} else {
			TrieDBMut::from_existing(acct_db, &mut storage_root)?
		};
		let pairs = rlp.at(if use_short_version { 4 } else { 5 })?;
		for pair_rlp in pairs.iter() {
			let k: Bytes = pair_rlp.val_at(0)?;
			let v: Bytes = pair_rlp.val_at(1)?;

			storage_trie.insert(&k, &v)?;
		}
	}

	let acc = BasicAccount {
		nonce,
		balance,
		storage_root,
		code_hash,
		code_version,
	};

	Ok((acc, new_code))
}
