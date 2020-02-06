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

//! EIP-2124 implementation based on <https://eips.ethereum.org/EIPS/eip-2124>

use crc::crc32;
use ethereum_types::H256;
use maplit::*;
use rlp::{DecoderError, Rlp, RlpStream};
use rlp_derive::*;
use std::collections::{BTreeMap, BTreeSet};

pub type BlockNumber = u64;

/// `CRC32` hash of all previous forks starting from genesis block.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ForkHash(pub u32);

impl rlp::Encodable for ForkHash {
	fn rlp_append(&self, s: &mut RlpStream) {
		s.encoder().encode_value(&self.0.to_be_bytes());
	}
}

impl rlp::Decodable for ForkHash {
	fn decode(rlp: &Rlp) -> Result<Self, DecoderError> {
		rlp.decoder().decode_value(|b| {
			if b.len() != 4 {
				return Err(DecoderError::RlpInvalidLength);
			}

			let mut blob = [0; 4];
			blob.copy_from_slice(&b[..]);

			return Ok(Self(u32::from_be_bytes(blob)))
		})
	}
}

impl From<H256> for ForkHash {
	fn from(genesis: H256) -> Self {
		Self(crc32::checksum_ieee(&genesis[..]))
	}
}

impl std::ops::AddAssign<BlockNumber> for ForkHash {
	fn add_assign(&mut self, height: BlockNumber) {
		let blob = height.to_be_bytes();
		self.0 = crc32::update(self.0, &crc32::IEEE_TABLE, &blob)
	}
}

impl std::ops::Add<BlockNumber> for ForkHash {
	type Output = Self;
	fn add(mut self, height: BlockNumber) -> Self {
		self += height;
		self
	}
}

/// A fork identifier as defined by EIP-2124.
/// Serves as the chain compatibility identifier.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, RlpEncodable, RlpDecodable)]
pub struct ForkId {
	/// CRC32 checksum of the all fork blocks from genesis.
	pub hash: ForkHash,     
	/// Next upcoming fork block number, 0 if not yet known.
	pub next: BlockNumber
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum RejectReason {
	RemoteStale,
	LocalIncompatibleOrStale,
}

/// Filter that describes the state of blockchain and can be used to check incoming `ForkId`s for compatibility.
#[derive(Clone, Debug)]
pub struct ForkFilter {
	/// Blockchain head
	pub head: BlockNumber,
	past_forks: BTreeMap<BlockNumber, ForkHash>,
	next_forks: BTreeSet<BlockNumber>,
}

impl ForkFilter {
	/// Create the filter from provided head, genesis block hash, past forks and expected future forks.
	pub fn new<PF, NF>(head: BlockNumber, genesis: H256, past_forks: PF, next_forks: NF) -> Self
	where
		PF: IntoIterator<Item = BlockNumber>,
		NF: IntoIterator<Item = BlockNumber>,
	{
		let genesis_fork_hash = ForkHash::from(genesis);
		Self {
			head,
			past_forks: past_forks.into_iter().fold((btreemap! { 0 => genesis_fork_hash }, genesis_fork_hash), |(mut acc, base_hash), block| {
				let fork_hash = base_hash + block;
				acc.insert(block, fork_hash);
				(acc, fork_hash)
			}).0,
			next_forks: next_forks.into_iter().collect(),
		}
	}

	fn current_fork_hash(&self) -> ForkHash {
		*self.past_forks.values().next_back().unwrap()
	}

	fn future_fork_hashes(&self) -> Vec<ForkHash> {
		self.next_forks.iter().fold((Vec::new(), self.current_fork_hash()), |(mut acc, hash), fork| {
			let next = hash + *fork;
			acc.push(next);
			(acc, next)
		}).0
	}

	/// Insert a new past fork
	pub fn insert_past_fork(&mut self, height: BlockNumber) {
		self.past_forks.insert(height, self.current_fork_hash() + height);
	}

	/// Insert a new upcoming fork
	pub fn insert_next_fork(&mut self, height: BlockNumber) {
		self.next_forks.insert(height);
	}

	/// Mark an upcoming fork as already happened and immutable.
	/// Returns `false` if no such fork existed and the call was a no-op.
	pub fn promote_next_fork(&mut self, height: BlockNumber) -> bool {
		let promoted = self.next_forks.remove(&height);
		if promoted {
			self.insert_past_fork(height);
		}
		promoted
	}

	/// Check whether the provided `ForkId` is compatible based on the validation rules in `EIP-2124`.
	pub fn is_valid(&self, fork_id: ForkId) -> Result<(), RejectReason> {
		// 1) If local and remote FORK_HASH matches...
		if self.current_fork_hash() == fork_id.hash {
			if fork_id.next == 0 {
				// 1b
				return Ok(())
			}

			//... compare local head to FORK_NEXT.
			if self.head < fork_id.next {
				return Ok(())
			} else {
				return Err(RejectReason::LocalIncompatibleOrStale)
			}
		}
		
		// 2) If the remote FORK_HASH is a subset of the local past forks...
		let mut it = self.past_forks.iter();
		while let Some((_, hash)) = it.next() {
			if *hash == fork_id.hash {
				// ...and the remote FORK_NEXT matches with the locally following fork block number, connect.
				if let Some((actual_fork_block, _)) = it.next() {
					if *actual_fork_block == fork_id.next {
						return Ok(())
					} else {
						return Err(RejectReason::RemoteStale);
					}
				}

				break;
			}
		}

		// 3) If the remote FORK_HASH is a superset of the local past forks and can be completed with locally known future forks, connect.
		for future_fork_hash in self.future_fork_hashes() {
			if future_fork_hash == fork_id.hash {
				return Ok(())
			}
		}

		// 4) Reject in all other cases
		Err(RejectReason::LocalIncompatibleOrStale)
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use hex_literal::hex;

	const GENESIS_HASH: &str = "d4e56740f876aef8c010b86a40d5f56745a118d0906a34e69aec8c0db1cb8fa3";
	const BYZANTIUM_FORK_HEIGHT: BlockNumber = 4370000;
	const PETERSBURG_FORK_HEIGHT: BlockNumber = 7280000;

	#[test]
	fn test_forkhash() {
		let mut fork_hash = ForkHash::from(GENESIS_HASH.parse::<H256>().unwrap());
		assert_eq!(fork_hash.0, 0xfc64ec04);

		fork_hash += 1150000;
		assert_eq!(fork_hash.0, 0x97c2c34c);

		fork_hash += 1920000;
		assert_eq!(fork_hash.0, 0x91d1f948);
	}

	#[test]
	fn test_compatibility_check() {
		let spurious_filter = ForkFilter::new(
			4369999,
			GENESIS_HASH.parse().unwrap(),
			vec![1150000, 1920000, 2463000, 2675000],
			vec![BYZANTIUM_FORK_HEIGHT]
		);
		let mut byzantium_filter = spurious_filter.clone();
		byzantium_filter.promote_next_fork(BYZANTIUM_FORK_HEIGHT);
		byzantium_filter.insert_next_fork(PETERSBURG_FORK_HEIGHT);
		byzantium_filter.head = 7279999;

		let mut petersburg_filter = byzantium_filter.clone();
		petersburg_filter.promote_next_fork(PETERSBURG_FORK_HEIGHT);
		petersburg_filter.head = 7987396;

		// Local is mainnet Petersburg, remote announces the same. No future fork is announced.
		assert_eq!(petersburg_filter.is_valid(ForkId { hash: ForkHash(0x668db0af), next: 0 }), Ok(()));

		// Local is mainnet Petersburg, remote announces the same. Remote also announces a next fork
		// at block 0xffffffff, but that is uncertain.
		assert_eq!(petersburg_filter.is_valid(ForkId { hash: ForkHash(0x668db0af), next: BlockNumber::max_value() }), Ok(()));

		// Local is mainnet currently in Byzantium only (so it's aware of Petersburg),remote announces
		// also Byzantium, but it's not yet aware of Petersburg (e.g. non updated node before the fork).
		// In this case we don't know if Petersburg passed yet or not.
		assert_eq!(byzantium_filter.is_valid(ForkId { hash: ForkHash(0xa00bc324), next: 0 }), Ok(()));

		// Local is mainnet currently in Byzantium only (so it's aware of Petersburg), remote announces
		// also Byzantium, and it's also aware of Petersburg (e.g. updated node before the fork). We
		// don't know if Petersburg passed yet (will pass) or not.
		assert_eq!(byzantium_filter.is_valid(ForkId { hash: ForkHash(0xa00bc324), next: PETERSBURG_FORK_HEIGHT }), Ok(()));

		// Local is mainnet currently in Byzantium only (so it's aware of Petersburg), remote announces
		// also Byzantium, and it's also aware of some random fork (e.g. misconfigured Petersburg). As
		// neither forks passed at neither nodes, they may mismatch, but we still connect for now.
		assert_eq!(byzantium_filter.is_valid(ForkId { hash: ForkHash(0xa00bc324), next: BlockNumber::max_value() }), Ok(()));

		// Local is mainnet Petersburg, remote announces Byzantium + knowledge about Petersburg. Remote is simply out of sync, accept.
		assert_eq!(petersburg_filter.is_valid(ForkId { hash: ForkHash(0xa00bc324), next: PETERSBURG_FORK_HEIGHT }), Ok(()));

		// Local is mainnet Petersburg, remote announces Spurious + knowledge about Byzantium. Remote
		// is definitely out of sync. It may or may not need the Petersburg update, we don't know yet.
		assert_eq!(petersburg_filter.is_valid(ForkId { hash: ForkHash(0x3edd5b10), next: 4370000 }), Ok(()));

		// Local is mainnet Byzantium, remote announces Petersburg. Local is out of sync, accept.
		assert_eq!(byzantium_filter.is_valid(ForkId { hash: ForkHash(0x668db0af), next: 0 }), Ok(()));

		// Local is mainnet Spurious, remote announces Byzantium, but is not aware of Petersburg. Local
		// out of sync. Local also knows about a future fork, but that is uncertain yet.
		assert_eq!(spurious_filter.is_valid(ForkId { hash: ForkHash(0xa00bc324), next: 0 }), Ok(()));

		// Local is mainnet Petersburg. remote announces Byzantium but is not aware of further forks.
		// Remote needs software update.
		assert_eq!(petersburg_filter.is_valid(ForkId { hash: ForkHash(0xa00bc324), next: 0 }), Err(RejectReason::RemoteStale));

		// Local is mainnet Petersburg, and isn't aware of more forks. Remote announces Petersburg +
		// 0xffffffff. Local needs software update, reject.
		assert_eq!(petersburg_filter.is_valid(ForkId { hash: ForkHash(0x5cddc0e1), next: 0 }), Err(RejectReason::LocalIncompatibleOrStale));

		// Local is mainnet Byzantium, and is aware of Petersburg. Remote announces Petersburg +
		// 0xffffffff. Local needs software update, reject.
		assert_eq!(byzantium_filter.is_valid(ForkId { hash: ForkHash(0x5cddc0e1), next: 0 }), Err(RejectReason::LocalIncompatibleOrStale));

		// Local is mainnet Petersburg, remote is Rinkeby Petersburg.
		assert_eq!(petersburg_filter.is_valid(ForkId { hash: ForkHash(0xafec6b27), next: 0 }), Err(RejectReason::LocalIncompatibleOrStale));

		// Local is mainnet Petersburg, far in the future. Remote announces Gopherium (non existing fork)
		// at some future block 88888888, for itself, but past block for local. Local is incompatible.
		//
		// This case detects non-upgraded nodes with majority hash power (typical Ropsten mess).
		let mut far_away_petersburg = petersburg_filter.clone();
		far_away_petersburg.head = 88888888;
		assert_eq!(far_away_petersburg.is_valid(ForkId { hash: ForkHash(0x668db0af), next: 88888888 }), Err(RejectReason::LocalIncompatibleOrStale));

		// Local is mainnet Byzantium. Remote is also in Byzantium, but announces Gopherium (non existing
		// fork) at block 7279999, before Petersburg. Local is incompatible.
		assert_eq!(byzantium_filter.is_valid(ForkId { hash: ForkHash(0xa00bc324), next: 7279999 }), Err(RejectReason::LocalIncompatibleOrStale));
	}

	#[test]
	fn test_forkid_serialization() {
		assert_eq!(rlp::encode(&ForkId { hash: ForkHash(0), next: 0 }), hex!("c6840000000080"));
		assert_eq!(rlp::encode(&ForkId { hash: ForkHash(0xdeadbeef), next: 0xBADDCAFE }), hex!("ca84deadbeef84baddcafe"));
		assert_eq!(rlp::encode(&ForkId { hash: ForkHash(u32::max_value()), next: u64::max_value() }), hex!("ce84ffffffff88ffffffffffffffff"));

		assert_eq!(rlp::decode::<ForkId>(&hex!("c6840000000080")).unwrap(), ForkId { hash: ForkHash(0), next: 0 });
		assert_eq!(rlp::decode::<ForkId>(&hex!("ca84deadbeef84baddcafe")).unwrap(), ForkId { hash: ForkHash(0xdeadbeef), next: 0xBADDCAFE });
		assert_eq!(rlp::decode::<ForkId>(&hex!("ce84ffffffff88ffffffffffffffff")).unwrap(), ForkId { hash: ForkHash(u32::max_value()), next: u64::max_value() });
	}
}
