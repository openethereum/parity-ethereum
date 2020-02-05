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
use rlp::RlpStream;

pub type BlockNumber = u64;

#[derive(Clone, Copy, Debug)]
pub struct ForkHash(pub u32);

impl rlp::Encodable for ForkHash {
	fn rlp_append(&self, s: &mut RlpStream) {
		s.encoder().encode_value(&self.0.to_be_bytes());
	}
}

impl From<H256> for ForkHash {
	fn from(genesis: H256) -> Self {
		Self(crc32::checksum_ieee(&genesis[..]))
	}
}

impl std::ops::Add<BlockNumber> for ForkHash {
	type Output = Self;
	fn add(self, height: BlockNumber) -> Self {
		let blob = height.to_be_bytes();
		Self(crc32::update(self.0, &crc32::IEEE_TABLE, &blob))
	}
}

/// A fork identifier as defined by EIP-2124.
#[derive(Clone, Copy, Debug)]
pub struct ForkId {
	/// CRC32 checksum of the all fork blocks from genesis.
	pub hash: ForkHash,     
	/// Next upcoming fork block number, 0 if not yet known.
	pub next: BlockNumber
}

impl rlp::Encodable for ForkId {
	fn rlp_append(&self, s: &mut RlpStream) {
		s.begin_list(2);
		s.append(&self.hash);
		s.append(&self.next);
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use hex_literal::hex;

	const GENESIS_HASH: &str = "d4e56740f876aef8c010b86a40d5f56745a118d0906a34e69aec8c0db1cb8fa3";

	#[test]
	fn test_forkhash() {
		let mut fork_hash = ForkHash::from(GENESIS_HASH.parse::<H256>().unwrap());
		assert_eq!(fork_hash.0, 0xfc64ec04);

		fork_hash = fork_hash + 1150000;
		assert_eq!(fork_hash.0, 0x97c2c34c);

		fork_hash = fork_hash + 1920000;
		assert_eq!(fork_hash.0, 0x91d1f948);
	}

	#[test]
	fn test_forkid_serialization() {
		assert_eq!(rlp::encode(&ForkId { hash: ForkHash(0), next: 0 }), hex!("c6840000000080"));
		assert_eq!(rlp::encode(&ForkId { hash: ForkHash(0xdeadbeef), next: 0xBADDCAFE }), hex!("ca84deadbeef84baddcafe"));
		assert_eq!(rlp::encode(&ForkId { hash: ForkHash(u32::max_value()), next: u64::max_value() }), hex!("ce84ffffffff88ffffffffffffffff"));
	}
}
