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

use ethereum_types::{Address, H256};
use lru_cache::LruCache;
use parking_lot::RwLock;

use engines::clique::{SIGNER_SIG_LENGTH, SIGNER_VANITY_LENGTH};
use error::Error;
use ethkey::{public_to_address, recover as ec_recover, Signature};
use types::header::Header;

/// How many recovered signature to cache in the memory.
pub const CREATOR_CACHE_NUM: usize = 4096;
lazy_static! {
	/// key: header hash
	/// value: creator address
	static ref CREATOR_BY_HASH: RwLock<LruCache<H256, Address>> = RwLock::new(LruCache::new(CREATOR_CACHE_NUM));
}

/// Recover block creator from signature
pub fn recover_creator(header: &Header) -> Result<Address, Error> {
	// Initialization
	let mut cache = CREATOR_BY_HASH.write();

	if let Some(creator) = cache.get_mut(&header.hash()) {
		return Ok(*creator);
	}

	let data = header.extra_data();
	if data.len() < SIGNER_VANITY_LENGTH + SIGNER_SIG_LENGTH {
		return Err(From::from("extra_data length is not enough!"));
	}

	let mut sig_data = data[data.len() - SIGNER_SIG_LENGTH..].to_vec();
	sig_data.resize(SIGNER_SIG_LENGTH, 0);

	let mut sig = [0; SIGNER_SIG_LENGTH];
	sig.copy_from_slice(&sig_data[..]);

	let reduced_header = &mut header.clone();
	reduced_header.set_extra_data(data[..data.len() - SIGNER_SIG_LENGTH].to_vec());

	let msg = reduced_header.hash();
	let pubkey = ec_recover(&Signature::from(sig), &msg)?;
	let creator = public_to_address(&pubkey);

	cache.insert(header.hash(), creator.clone());
	Ok(creator)
}

/// Extract signer list from extra_data.
///
/// Layout of extra_data:
/// ----
/// VANITY: 32 bytes
/// Signers: N * 32 bytes as hex encoded (20 characters)
/// Signature: 65 bytes
/// --
pub fn extract_signers(header: &Header) -> Result<Vec<Address>, Error> {
	let data = header.extra_data();

	if data.len() <= SIGNER_VANITY_LENGTH + SIGNER_SIG_LENGTH {
		return Err(Box::new("Invalid extra_data size.").into());
	}

	// extract only the portion of extra_data which includes the signer list
	let signers_raw = &data[(SIGNER_VANITY_LENGTH)..data.len() - (SIGNER_SIG_LENGTH)];

	let address_length = 20;
	if signers_raw.len() % address_length != 0 {
		return Err(Box::new("bad signer list.").into());
	}

	let num_signers = signers_raw.len() / 20;
	let mut signers_list: Vec<Address> = Vec::with_capacity(num_signers);

	for i in 0..num_signers {
		let mut signer = Address::default();
		signer.copy_from_slice(&signers_raw[i * address_length..(i + 1) * address_length]);
		signers_list.push(signer);
	}

	// NOTE: signers list must be sorted by ascending order.
	signers_list.sort();

	Ok(signers_list)
}
