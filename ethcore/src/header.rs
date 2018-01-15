// Copyright 2015-2017 Parity Technologies (UK) Ltd.
// This file is part of Parity.

// Parity is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity.  If not, see <http://www.gnu.org/licenses/>.

//! Block header.

use std::cmp;
use std::cell::RefCell;
use hash::{KECCAK_NULL_RLP, KECCAK_EMPTY_LIST_RLP, keccak};
use heapsize::HeapSizeOf;
use ethereum_types::{H256, U256, Address, Bloom};
use bytes::Bytes;
use basic_types::{ZERO_LOGBLOOM};
use time::get_time;
use rlp::*;

pub use basic_types::Seal;

pub use types::BlockNumber;

/// A block header.
///
/// Reflects the specific RLP fields of a block in the chain with additional room for the seal
/// which is non-specific.
///
/// Doesn't do all that much on its own.
#[derive(Debug, Clone, Eq)]
pub struct Header {
	/// Parent hash.
	parent_hash: H256,
	/// Block timestamp.
	timestamp: u64,
	/// Block number.
	number: BlockNumber,
	/// Block author.
	author: Address,

	/// Transactions root.
	transactions_root: H256,
	/// Block uncles hash.
	uncles_hash: H256,
	/// Block extra data.
	extra_data: Bytes,

	/// State root.
	state_root: H256,
	/// Block receipts root.
	receipts_root: H256,
	/// Block bloom.
	log_bloom: Bloom,
	/// Gas used for contracts execution.
	gas_used: U256,
	/// Block gas limit.
	gas_limit: U256,

	/// Block difficulty.
	difficulty: U256,
	/// Vector of post-RLP-encoded fields.
	seal: Vec<Bytes>,

	/// The memoized hash of the RLP representation *including* the seal fields.
	hash: RefCell<Option<H256>>,
	/// The memoized hash of the RLP representation *without* the seal fields.
	bare_hash: RefCell<Option<H256>>,
}

impl PartialEq for Header {
	fn eq(&self, c: &Header) -> bool {
		self.parent_hash == c.parent_hash &&
		self.timestamp == c.timestamp &&
		self.number == c.number &&
		self.author == c.author &&
		self.transactions_root == c.transactions_root &&
		self.uncles_hash == c.uncles_hash &&
		self.extra_data == c.extra_data &&
		self.state_root == c.state_root &&
		self.receipts_root == c.receipts_root &&
		self.log_bloom == c.log_bloom &&
		self.gas_used == c.gas_used &&
		self.gas_limit == c.gas_limit &&
		self.difficulty == c.difficulty &&
		self.seal == c.seal
	}
}

impl Default for Header {
	fn default() -> Self {
		Header {
			parent_hash: H256::default(),
			timestamp: 0,
			number: 0,
			author: Address::default(),

			transactions_root: KECCAK_NULL_RLP,
			uncles_hash: KECCAK_EMPTY_LIST_RLP,
			extra_data: vec![],

			state_root: KECCAK_NULL_RLP,
			receipts_root: KECCAK_NULL_RLP,
			log_bloom: ZERO_LOGBLOOM.clone(),
			gas_used: U256::default(),
			gas_limit: U256::default(),

			difficulty: U256::default(),
			seal: vec![],
			hash: RefCell::new(None),
			bare_hash: RefCell::new(None),
		}
	}
}

impl Header {
	/// Create a new, default-valued, header.
	pub fn new() -> Self {
		Self::default()
	}

	/// Get the parent_hash field of the header.
	pub fn parent_hash(&self) -> &H256 { &self.parent_hash }
	/// Get the timestamp field of the header.
	pub fn timestamp(&self) -> u64 { self.timestamp }
	/// Get the number field of the header.
	pub fn number(&self) -> BlockNumber { self.number }
	/// Get the author field of the header.
	pub fn author(&self) -> &Address { &self.author }

	/// Get the extra data field of the header.
	pub fn extra_data(&self) -> &Bytes { &self.extra_data }
	/// Get a mutable reference to extra_data
	pub fn extra_data_mut(&mut self) -> &mut Bytes { self.note_dirty(); &mut self.extra_data }

	/// Get the state root field of the header.
	pub fn state_root(&self) -> &H256 { &self.state_root }
	/// Get the receipts root field of the header.
	pub fn receipts_root(&self) -> &H256 { &self.receipts_root }
	/// Get the log bloom field of the header.
	pub fn log_bloom(&self) -> &Bloom { &self.log_bloom }
	/// Get the transactions root field of the header.
	pub fn transactions_root(&self) -> &H256 { &self.transactions_root }
	/// Get the uncles hash field of the header.
	pub fn uncles_hash(&self) -> &H256 { &self.uncles_hash }
	/// Get the gas used field of the header.
	pub fn gas_used(&self) -> &U256 { &self.gas_used }
	/// Get the gas limit field of the header.
	pub fn gas_limit(&self) -> &U256 { &self.gas_limit }

	/// Get the difficulty field of the header.
	pub fn difficulty(&self) -> &U256 { &self.difficulty }
	/// Get the seal field of the header.
	pub fn seal(&self) -> &[Bytes] { &self.seal }
	/// Get the seal field with RLP-decoded values as bytes.
	pub fn decode_seal<'a, T: ::std::iter::FromIterator<&'a [u8]>>(&'a self) -> Result<T, DecoderError> {
		self.seal.iter().map(|rlp| {
			UntrustedRlp::new(rlp).data()
		}).collect()
	}

	// TODO: seal_at, set_seal_at &c.

	/// Set the number field of the header.
	pub fn set_parent_hash(&mut self, a: H256) { self.parent_hash = a; self.note_dirty(); }
	/// Set the uncles hash field of the header.
	pub fn set_uncles_hash(&mut self, a: H256) { self.uncles_hash = a; self.note_dirty(); }
	/// Set the state root field of the header.
	pub fn set_state_root(&mut self, a: H256) { self.state_root = a; self.note_dirty(); }
	/// Set the transactions root field of the header.
	pub fn set_transactions_root(&mut self, a: H256) { self.transactions_root = a; self.note_dirty() }
	/// Set the receipts root field of the header.
	pub fn set_receipts_root(&mut self, a: H256) { self.receipts_root = a; self.note_dirty() }
	/// Set the log bloom field of the header.
	pub fn set_log_bloom(&mut self, a: Bloom) { self.log_bloom = a; self.note_dirty() }
	/// Set the timestamp field of the header.
	pub fn set_timestamp(&mut self, a: u64) { self.timestamp = a; self.note_dirty(); }
	/// Set the timestamp field of the header to the current time.
	pub fn set_timestamp_now(&mut self, but_later_than: u64) { self.timestamp = cmp::max(get_time().sec as u64, but_later_than + 1); self.note_dirty(); }
	/// Set the number field of the header.
	pub fn set_number(&mut self, a: BlockNumber) { self.number = a; self.note_dirty(); }
	/// Set the author field of the header.
	pub fn set_author(&mut self, a: Address) { if a != self.author { self.author = a; self.note_dirty(); } }

	/// Set the extra data field of the header.
	pub fn set_extra_data(&mut self, a: Bytes) { if a != self.extra_data { self.extra_data = a; self.note_dirty(); } }

	/// Set the gas used field of the header.
	pub fn set_gas_used(&mut self, a: U256) { self.gas_used = a; self.note_dirty(); }
	/// Set the gas limit field of the header.
	pub fn set_gas_limit(&mut self, a: U256) { self.gas_limit = a; self.note_dirty(); }

	/// Set the difficulty field of the header.
	pub fn set_difficulty(&mut self, a: U256) { self.difficulty = a; self.note_dirty(); }
	/// Set the seal field of the header.
	pub fn set_seal(&mut self, a: Vec<Bytes>) { self.seal = a; self.note_dirty(); }

	/// Get the hash of this header (keccak of the RLP).
	pub fn hash(&self) -> H256 {
 		let mut hash = self.hash.borrow_mut();
 		match &mut *hash {
 			&mut Some(ref h) => h.clone(),
 			hash @ &mut None => {
				let h = self.rlp_keccak(Seal::With);
 				*hash = Some(h.clone());
 				h
 			}
		}
	}

	/// Get the hash of the header excluding the seal
	pub fn bare_hash(&self) -> H256 {
		let mut hash = self.bare_hash.borrow_mut();
		match &mut *hash {
			&mut Some(ref h) => h.clone(),
			hash @ &mut None => {
				let h = self.rlp_keccak(Seal::Without);
				*hash = Some(h.clone());
				h
			}
		}
	}

	/// Note that some fields have changed. Resets the memoised hash.
	pub fn note_dirty(&self) {
 		*self.hash.borrow_mut() = None;
 		*self.bare_hash.borrow_mut() = None;
	}

	// TODO: make these functions traity
	/// Place this header into an RLP stream `s`, optionally `with_seal`.
	pub fn stream_rlp(&self, s: &mut RlpStream, with_seal: Seal) {
		s.begin_list(13 + match with_seal { Seal::With => self.seal.len(), _ => 0 });
		s.append(&self.parent_hash);
		s.append(&self.uncles_hash);
		s.append(&self.author);
		s.append(&self.state_root);
		s.append(&self.transactions_root);
		s.append(&self.receipts_root);
		s.append(&self.log_bloom);
		s.append(&self.difficulty);
		s.append(&self.number);
		s.append(&self.gas_limit);
		s.append(&self.gas_used);
		s.append(&self.timestamp);
		s.append(&self.extra_data);
		if let Seal::With = with_seal {
			for b in &self.seal {
				s.append_raw(b, 1);
			}
		}
	}

	/// Get the RLP of this header, optionally `with_seal`.
	pub fn rlp(&self, with_seal: Seal) -> Bytes {
		let mut s = RlpStream::new();
		self.stream_rlp(&mut s, with_seal);
		s.out()
	}

	/// Get the SHA3 (Keccak) of this header, optionally `with_seal`.
	pub fn rlp_keccak(&self, with_seal: Seal) -> H256 { keccak(self.rlp(with_seal)) }

	/// Encode the header, getting a type-safe wrapper around the RLP.
	pub fn encoded(&self) -> ::encoded::Header {
		::encoded::Header::new(self.rlp(Seal::With))
	}
}

impl Decodable for Header {
	fn decode(r: &UntrustedRlp) -> Result<Self, DecoderError> {
		let mut blockheader = Header {
			parent_hash: r.val_at(0)?,
			uncles_hash: r.val_at(1)?,
			author: r.val_at(2)?,
			state_root: r.val_at(3)?,
			transactions_root: r.val_at(4)?,
			receipts_root: r.val_at(5)?,
			log_bloom: r.val_at(6)?,
			difficulty: r.val_at(7)?,
			number: r.val_at(8)?,
			gas_limit: r.val_at(9)?,
			gas_used: r.val_at(10)?,
			timestamp: cmp::min(r.val_at::<U256>(11)?, u64::max_value().into()).as_u64(),
			extra_data: r.val_at(12)?,
			seal: vec![],
			hash: RefCell::new(Some(keccak(r.as_raw()))),
			bare_hash: RefCell::new(None),
		};

		for i in 13..r.item_count()? {
			blockheader.seal.push(r.at(i)?.as_raw().to_vec())
		}

		Ok(blockheader)
	}
}

impl Encodable for Header {
	fn rlp_append(&self, s: &mut RlpStream) {
		self.stream_rlp(s, Seal::With);
	}
}

impl HeapSizeOf for Header {
	fn heap_size_of_children(&self) -> usize {
		self.extra_data.heap_size_of_children() + self.seal.heap_size_of_children()
	}
}

impl ::parity_machine::Header for Header {
	fn bare_hash(&self) -> H256 { Header::bare_hash(self) }

	fn hash(&self) -> H256 { Header::hash(self) }

	fn seal(&self) -> &[Vec<u8>] { Header::seal(self) }

	fn author(&self) -> &Address { Header::author(self) }

	fn number(&self) -> BlockNumber { Header::number(self) }
}

impl ::parity_machine::ScoredHeader for Header {
	fn score(&self) -> &U256 { self.difficulty() }
	fn set_score(&mut self, score: U256) { self.set_difficulty(score) }
}

#[cfg(test)]
mod tests {
	use rustc_hex::FromHex;
	use rlp;
	use super::Header;

	#[test]
	fn test_header_seal_fields() {
		// that's rlp of block header created with ethash engine.
		let header_rlp = "f901f9a0d405da4e66f1445d455195229624e133f5baafe72b5cf7b3c36c12c8146e98b7a01dcc4de8dec75d7aab85b567b6ccd41ad312451b948a7413f0a142fd40d49347948888f1f195afa192cfee860698584c030f4c9db1a05fb2b4bfdef7b314451cb138a534d225c922fc0e5fbe25e451142732c3e25c25a088d2ec6b9860aae1a2c3b299f72b6a5d70d7f7ba4722c78f2c49ba96273c2158a007c6fdfa8eea7e86b81f5b0fc0f78f90cc19f4aa60d323151e0cac660199e9a1b90100000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000008302008003832fefba82524d84568e932a80a0a0349d8c3df71f1a48a9df7d03fd5f14aeee7d91332c009ecaff0a71ead405bd88ab4e252a7e8c2a23".from_hex().unwrap();
		let mix_hash = "a0a0349d8c3df71f1a48a9df7d03fd5f14aeee7d91332c009ecaff0a71ead405bd".from_hex().unwrap();
		let mix_hash_decoded = "a0349d8c3df71f1a48a9df7d03fd5f14aeee7d91332c009ecaff0a71ead405bd".from_hex().unwrap();
		let nonce = "88ab4e252a7e8c2a23".from_hex().unwrap();
		let nonce_decoded = "ab4e252a7e8c2a23".from_hex().unwrap();

		let header: Header = rlp::decode(&header_rlp);
		let seal_fields = header.seal.clone();
		assert_eq!(seal_fields.len(), 2);
		assert_eq!(seal_fields[0], mix_hash);
		assert_eq!(seal_fields[1], nonce);

		let decoded_seal = header.decode_seal::<Vec<_>>().unwrap();
		assert_eq!(decoded_seal.len(), 2);
		assert_eq!(decoded_seal[0], &*mix_hash_decoded);
		assert_eq!(decoded_seal[1], &*nonce_decoded);
	}

	#[test]
	fn decode_and_encode_header() {
		// that's rlp of block header created with ethash engine.
		let header_rlp = "f901f9a0d405da4e66f1445d455195229624e133f5baafe72b5cf7b3c36c12c8146e98b7a01dcc4de8dec75d7aab85b567b6ccd41ad312451b948a7413f0a142fd40d49347948888f1f195afa192cfee860698584c030f4c9db1a05fb2b4bfdef7b314451cb138a534d225c922fc0e5fbe25e451142732c3e25c25a088d2ec6b9860aae1a2c3b299f72b6a5d70d7f7ba4722c78f2c49ba96273c2158a007c6fdfa8eea7e86b81f5b0fc0f78f90cc19f4aa60d323151e0cac660199e9a1b90100000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000008302008003832fefba82524d84568e932a80a0a0349d8c3df71f1a48a9df7d03fd5f14aeee7d91332c009ecaff0a71ead405bd88ab4e252a7e8c2a23".from_hex().unwrap();

		let header: Header = rlp::decode(&header_rlp);
		let encoded_header = rlp::encode(&header).into_vec();

		assert_eq!(header_rlp, encoded_header);
	}
}
