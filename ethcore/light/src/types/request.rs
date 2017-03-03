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

//! Light protocol request types.

use std::collections::HashMap;

use ethcore::transaction::Action;
use util::{Address, H256, U256, Uint};

// re-exports of request types.
pub use self::header::{
	Complete as CompleteHeadersRequest,
	Incomplete as IncompleteHeadersRequest,
	Response as HeadersResponse
};
pub use self::header_proof::{
	Complete as CompleteHeaderProofRequest,
	Incomplete as IncompleteHeaderProofRequest,
	Response as HeaderProofResponse
};
pub use self::block_body::{
	Complete as CompleteBodyRequest,
	Incomplete as IncompleteBodyRequest,
	Response as BodyResponse
};
pub use self::receipts::{
	Complete as CompleteReceiptsRequest,
	Incomplete as IncompleteReceiptsRequest
	Response as ReceiptsResponse
};
pub use self::account::{
	Complete as CompleteAccountRequest,
	Incomplete as IncompleteAccountRequest,
	Response as AccountResponse,
};
pub use self::storage::{
	Complete as CompleteStorageRequest,
	Incomplete as IncompleteStorageRequest,
	Response as StorageResponse
};
pub use self::contract_code::{
	Complete as CompleteCodeRequest,
	Incomplete as IncompleteCodeRequest,
	Response as CodeResponse,
};

/// Error indicating a reference to a non-existent or wrongly-typed output.
pub struct NoSuchOutput;

/// An input to a request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Field<T> {
	/// A pre-specified input.
	Scalar(T),
	/// An input which can be resolved later on.
	/// (Request index, output index)
	BackReference(usize, usize),
}

impl From<T> for Field<T> {
	fn from(val: T) -> Self {
		Field::Scalar(val)
	}
}

/// Request outputs which can be reused as inputs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Output {
	/// A 32-byte hash output.
	Hash(H256),
	/// An unsigned-integer output.
	Number(u64),
}

/// Response output kinds which can be used as back-references.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OutputKind {
	/// A 32-byte hash output.
	Hash,
	/// An unsigned-integer output.
	Number,
}

/// Either a hash or a number.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "ipc", binary)]
pub enum HashOrNumber {
	/// Block hash variant.
	Hash(H256),
	/// Block number variant.
	Number(u64),
}

impl From<H256> for HashOrNumber {
	fn from(hash: H256) -> Self {
		HashOrNumber::Hash(hash)
	}
}

impl From<u64> for HashOrNumber {
	fn from(num: u64) -> Self {
		HashOrNumber::Number(num)
	}
}

/// A potentially incomplete request.
pub trait IncompleteRequest: Sized {
	type Complete;

	/// Check prior outputs against the needed inputs.
	///
	/// This is called to ensure consistency of this request with
	/// others in the same packet.
	fn check_outputs<F>(&self, f: F) -> Result<(), NoSuchOutput>
		where F: FnMut(usize, usize, OutputKind) -> Result<(), NoSuchOutput>;

	/// Note that this request will produce the following outputs.
	fn note_outputs<F>(&self, f: F) where F: FnMut(usize, OutputKind);

	/// Fill the request.
	///
	/// This function is provided an "output oracle" which allows fetching of
	/// prior request outputs.
	/// Only outputs previously checked with `check_outputs` will be available.
	fn fill<F>(self, oracle: F) -> Result<Self::Complete, NoSuchOutput>
		where F: Fn(usize, usize) -> Result<Output, NoSuchOutput>;
}

/// Header request.
pub mod header {
	use super::{Field, HashOrNumber, NoSuchOutput, OutputKind, Output};
	use ethcore::encoded;
	use util::U256;

	/// Potentially incomplete headers request.
	#[derive(Debug, Clone, PartialEq, Eq)]
	pub struct Incomplete {
		/// Start block.
		pub start: Field<HashOrNumber>,
		/// Skip between.
		pub skip: U256,
		/// Maximum to return.
		pub max: U256,
		/// Whether to reverse from start.
		pub reverse: bool,
	}

	impl super::IncompleteRequest for Incomplete {
		type Complete = Complete;

		fn check_outputs<F>(&self, mut f: F) -> Result<(), NoSuchOutput>
			where F: FnMut(usize, usize, OutputKind) -> Result<(), NoSuchOutput>
		{
			match self.start {
				Field::Scalar(_) => Ok(()),
				Field::BackReference(req, idx) =>
					f(req, idx, OutputKind::Hash).or_else(|| f(req, idx, OutputKind::Number))
			}
		}

		fn note_outputs<F>(&self, _: F) where F: FnMut(usize, OutputKind) { }

		fn fill<F>(self, oracle: F) -> Result<Self::Complete, NoSuchOutput>
			where F: Fn(usize, usize) -> Result<Output, NoSuchOutput>
		{
			let start = match self.start {
				Field::Scalar(start) => start,
				Field::BackReference(req, idx) => match oracle(req, idx)? {
					Output::Hash(hash) => hash.into(),
					Output::Number(num) => num.into(),
				}
			};

			Ok(Complete {
				start: start,
				skip: self.skip,
				max: self.max,
				reverse: self.reverse,
			})
		}

	}

	/// A complete header request.
	#[derive(Debug, Clone, PartialEq, Eq)]
	pub struct Complete {
		/// Start block.
		pub start: HashOrNumber,
		/// Skip between.
		pub skip: U256,
		/// Maximum to return.
		pub max: U256,
		/// Whether to reverse from start.
		pub reverse: bool,
	}

	/// The output of a request for headers.
	#[derive(Debug, Clone, PartialEq, Eq)]
	pub struct Response {
		header: Vec<encoded::Header>,
	}

	impl Response {
		/// Fill reusable outputs by writing them into the function.
		pub fn fill_outputs<F>(&self, _: F) where F: FnMut(usize, Output) { }
	}
}

/// Request and response for header proofs.
pub mod header_proof {
	use super::{Field, NoSuchOutput, OutputKind, Output};
	use util::{Bytes, U256, H256};

	/// Potentially incomplete header proof request.
	#[derive(Debug, Clone, PartialEq, Eq)]
	pub struct Incomplete {
		/// Block number.
		pub num: Field<u64>,
	}

	impl super::IncompleteRequest for Incomplete {
		type Complete = Complete;

		fn check_outputs<F>(&self, mut f: F) -> Result<(), NoSuchOutput>
			where F: FnMut(usize, usize, OutputKind) -> Result<(), NoSuchOutput>
		{
			match self.num {
				Field::Scalar(_) => Ok(()),
				Field::BackReference(req, idx) => f(req, idx, OutputKind::Number),
			}
		}

		fn note_outputs<F>(&self, mut note: F) where F: FnMut(usize, OutputKind) {
			note(1, OutputKind::Hash);
		}

		fn fill<F>(self, oracle: F) -> Result<Self::Complete, NoSuchOutput>
			where F: Fn(usize, usize) -> Result<Output, NoSuchOutput>
		{
			let num = match self.num {
				Field::Scalar(num) => num,
				Field::BackReference(req, idx) => match oracle(req, idx)? {
					Output::Number(num) => num,
					_ => return Err(NoSuchOutput),
				}
			};

			Ok(Complete {
				num: num,
			})
		}

	}

	/// A complete header proof request.
	#[derive(Debug, Clone, PartialEq, Eq)]
	pub struct Complete {
		/// The number to get a header proof for.
		pub num: u64,
	}

	/// The output of a request for a header proof.
	#[derive(Debug, Clone, PartialEq, Eq)]
	pub struct Response {
		/// Inclusion proof of the header and total difficulty in the CHT.
		pub proof: Vec<Bytes>,
		/// The proved header's hash.
		pub hash: H256,
		/// The proved header's total difficulty.
		pub td: U256,
	}

	impl Response {
		/// Fill reusable outputs by providing them to the function.
		pub fn fill_outputs<F>(&self, mut f: F) where F: FnMut(usize, Output) {
			f(1, Output::Hash(self.hash));
		}
	}
}

/// Request and response for block receipts
pub mod block_receipts {
	use super::{Field, NoSuchOutput, OutputKind, Output};
	use util::{Bytes, U256, H256};

	/// Potentially incomplete block receipts request.
	#[derive(Debug, Clone, PartialEq, Eq)]
	pub struct Incomplete {
		/// Block hash to get receipts for.
		pub hash: Field<H256>,
	}

	impl super::IncompleteRequest for Incomplete {
		type Complete = Complete;

		fn check_outputs<F>(&self, mut f: F) -> Result<(), NoSuchOutput>
			where F: FnMut(usize, usize, OutputKind) -> Result<(), NoSuchOutput>
		{
			match self.num {
				Field::Scalar(_) => Ok(()),
				Field::BackReference(req, idx) => f(req, idx, OutputKind::Hash),
			}
		}

		fn note_outputs<F>(&self, _: F) where F: FnMut(usize, OutputKind) {}

		fn fill<F>(self, oracle: F) -> Result<Self::Complete, NoSuchOutput>
			where F: Fn(usize, usize) -> Result<Output, NoSuchOutput>
		{
			let hash = match self.hash {
				Field::Scalar(hash) => hash,
				Field::BackReference(req, idx) => match oracle(req, idx)? {
					Output::Hash(hash) => hash,
					_ => return Err(NoSuchOutput),
				}
			};

			Ok(Complete {
				hash: hash,
			})
		}

	}

	/// A complete block receipts request.
	#[derive(Debug, Clone, PartialEq, Eq)]
	pub struct Complete {
		/// The number to get block receipts for.
		pub hash: H256,
	}

	/// The output of a request for block receipts.
	#[derive(Debug, Clone, PartialEq, Eq)]
	pub struct Response {
		/// The block receipts.
		pub receipts: Vec<Receipt>
	}

	impl Response {
		/// Fill reusable outputs by providing them to the function.
		pub fn fill_outputs<F>(&self, _: F) where F: FnMut(usize, Output) {}
	}
}

/// Request and response for a block body
pub mod block_body {
	use super::{Field, NoSuchOutput, OutputKind, Output};
	use ethcore::encoded;
	use util::{Bytes, U256, H256};

	/// Potentially incomplete block body request.
	#[derive(Debug, Clone, PartialEq, Eq)]
	pub struct Incomplete {
		/// Block hash to get receipts for.
		pub hash: Field<H256>,
	}

	impl super::IncompleteRequest for Incomplete {
		type Complete = Complete;

		fn check_outputs<F>(&self, mut f: F) -> Result<(), NoSuchOutput>
			where F: FnMut(usize, usize, OutputKind) -> Result<(), NoSuchOutput>
		{
			match self.num {
				Field::Scalar(_) => Ok(()),
				Field::BackReference(req, idx) => f(req, idx, OutputKind::Hash),
			}
		}

		fn note_outputs<F>(&self, _: F) where F: FnMut(usize, OutputKind) {}

		fn fill<F>(self, oracle: F) -> Result<Self::Complete, NoSuchOutput>
			where F: Fn(usize, usize) -> Result<Output, NoSuchOutput>
		{
			let hash = match self.hash {
				Field::Scalar(hash) => hash,
				Field::BackReference(req, idx) => match oracle(req, idx)? {
					Output::Hash(hash) => hash,
					_ => return Err(NoSuchOutput),
				}
			};

			Ok(Complete {
				hash: hash,
			})
		}

	}

	/// A complete block body request.
	#[derive(Debug, Clone, PartialEq, Eq)]
	pub struct Complete {
		/// The hash to get a block body for.
		pub hash: H256,
	}

	/// The output of a request for block body.
	#[derive(Debug, Clone, PartialEq, Eq)]
	pub struct Response {
		/// The block body.
		pub body: encoded::Body,
	}

	impl Response {
		/// Fill reusable outputs by providing them to the function.
		pub fn fill_outputs<F>(&self, _: F) where F: FnMut(usize, Output) {}
	}
}

/// A request for an account proof.
pub mod account {
	use super::{Field, NoSuchOutput, OutputKind, Output};
	use ethcore::encoded;
	use util::{Bytes, U256, H256};

	/// Potentially incomplete request for an account proof.
	#[derive(Debug, Clone, PartialEq, Eq)]
	pub struct Incomplete {
		/// Block hash to request state proof for.
		pub block_hash: Field<H256>,
		/// Hash of the account's address.
		pub address_hash: Field<H256>,
	}

	impl super::IncompleteRequest for Incomplete {
		type Complete = Complete;

		fn check_outputs<F>(&self, mut f: F) -> Result<(), NoSuchOutput>
			where F: FnMut(usize, usize, OutputKind) -> Result<(), NoSuchOutput>
		{
			if let Field::BackReference(req, idx) = self.block_hash {
				f(req, idx, OutputKind::Hash)?
			}

			if let Field::BackReference(req, idx) = self.address_hash {
				f(req, idx, OutputKind::Hash)?
			}

			Ok(())
		}

		fn note_outputs<F>(&self, mut f: F) where F: FnMut(usize, OutputKind) {
			f(0, OutputKind::Hash);
			f(1, OutputKind::Hash);
		}

		fn fill<F>(self, oracle: F) -> Result<Self::Complete, NoSuchOutput>
			where F: Fn(usize, usize) -> Result<Output, NoSuchOutput>
		{
			let block_hash = match self.block_hash {
				Field::Scalar(hash) => hash,
				Field::BackReference(req, idx) => match oracle(req, idx)? {
					Output::Hash(hash) => hash,
					_ => return Err(NoSuchOutput)?,
				}
			};

			let address_hash = match self.address_hash {
				Field::Scalar(hash) => hash,
				Field::BackReference(req, idx) => match oracle(req, idx)? {
					Output::Hash(hash) => hash,
					_ => return Err(NoSuchOutput)?,
				}
			};

			Ok(Complete {
				block_hash: block_hash,
				address_hash: address_hash,
			})
		}

	}

	/// A complete request for an account.
	#[derive(Debug, Clone, PartialEq, Eq)]
	pub struct Complete {
		/// Block hash to request state proof for.
		pub block_hash: H256,
		/// Hash of the account's address.
		pub address_hash: H256,
	}

	/// The output of a request for an account state proof.
	#[derive(Debug, Clone, PartialEq, Eq)]
	pub struct Response {
		/// Inclusion/exclusion proof
		pub proof: Vec<Bytes>,
		/// Account nonce.
		pub nonce: U256,
		/// Account balance.
		pub balance: U256,
		/// Account's code hash.
		pub code_hash: H256,
		/// Account's storage trie root.
		pub storage_root: H256,
	}

	impl Response {
		/// Fill reusable outputs by providing them to the function.
		pub fn fill_outputs<F>(&self, mut f: F) where F: FnMut(usize, Output) {
			f(0, Output::Hash(self.code_hash));
			f(1, Output::Hash(self.storage_root));
		}
	}
}

/// A request for a storage proof.
pub mod storage {
	use super::{Field, NoSuchOutput, OutputKind, Output};
	use ethcore::encoded;
	use util::{Bytes, U256, H256};

	/// Potentially incomplete request for an storage proof.
	#[derive(Debug, Clone, PartialEq, Eq)]
	pub struct Incomplete {
		/// Block hash to request state proof for.
		pub block_hash: Field<H256>,
		/// Hash of the account's address.
		pub address_hash: Field<H256>,
		/// Hash of the storage key.
		pub key_hash: Field<H256>,
	}

	impl super::IncompleteRequest for Incomplete {
		type Complete = Complete;

		fn check_outputs<F>(&self, mut f: F) -> Result<(), NoSuchOutput>
			where F: FnMut(usize, usize, OutputKind) -> Result<(), NoSuchOutput>
		{
			if let Field::BackReference(req, idx) = self.block_hash {
				f(req, idx, OutputKind::Hash)?
			}

			if let Field::BackReference(req, idx) = self.address_hash {
				f(req, idx, OutputKind::Hash)?
			}

			if let Field::BackReference(req, idx) = self.key_hash {
				f(req, idx, OutputKind::Hash)?
			}

			Ok(())
		}

		fn note_outputs<F>(&self, mut f: F) where F: FnMut(usize, OutputKind) {
			f(0, OutputKind::Hash);
		}

		fn fill<F>(self, oracle: F) -> Result<Self::Complete, NoSuchOutput>
			where F: Fn(usize, usize) -> Result<Output, NoSuchOutput>
		{
			let block_hash = match self.block_hash {
				Field::Scalar(hash) => hash,
				Field::BackReference(req, idx) => match oracle(req, idx)? {
					Output::Hash(hash) => hash,
					_ => return Err(NoSuchOutput)?,
				}
			};

			let address_hash = match self.address_hash {
				Field::Scalar(hash) => hash,
				Field::BackReference(req, idx) => match oracle(req, idx)? {
					Output::Hash(hash) => hash,
					_ => return Err(NoSuchOutput)?,
				}
			};

			let key_hash = match self.key_hash {
				Field::Scalar(hash) => hash,
				Field::BackReference(req, idx) => match oracle(req, idx)? {
					Output::Hash(hash) => hash,
					_ => return Err(NoSuchOutput)?,
				}
			};

			Ok(Complete {
				block_hash: block_hash,
				address_hash: address_hash,
				key_hash: key_hash
			})
		}

	}

	/// A complete request for a storage proof.
	#[derive(Debug, Clone, PartialEq, Eq)]
	pub struct Complete {
		/// Block hash to request state proof for.
		pub block_hash: H256,
		/// Hash of the account's address.
		pub address_hash: H256,
		/// Storage key hash.
		pub key_hash: H256,
	}

	/// The output of a request for an account state proof.
	#[derive(Debug, Clone, PartialEq, Eq)]
	pub struct Response {
		/// Inclusion/exclusion proof
		pub proof: Vec<Bytes>,
		/// Storage value.
		pub value: H256,
	}

	impl Response {
		/// Fill reusable outputs by providing them to the function.
		pub fn fill_outputs<F>(&self, mut f: F) where F: FnMut(usize, Output) {
			f(0, Output::Hash(self.value));
		}
	}
}

/// A request for contract code.
pub mod contract_code {
	use super::{Field, NoSuchOutput, OutputKind, Output};
	use ethcore::encoded;
	use util::{Bytes, U256, H256};

	/// Potentially incomplete _ request.
	#[derive(Debug, Clone, PartialEq, Eq)]
	pub struct Incomplete {
		/// The block hash to request the state for.
		pub block_hash: Field<H256>,
		/// The code hash.
		pub code_hash: Field<H256>,
	}

	impl super::IncompleteRequest for Incomplete {
		type Complete = Complete;

		fn check_outputs<F>(&self, mut f: F) -> Result<(), NoSuchOutput>
			where F: FnMut(usize, usize, OutputKind) -> Result<(), NoSuchOutput>
		{
			if let Field::BackReference(req, idx) = self.block_hash {
				f(req, idx, OutputKind::Hash)?;
			}
			if let Field::BackReference(req, idx) = self.code_hash {
				f(req, idx, OutputKind::Hash)?;
			}

			Ok(())
		}

		fn note_outputs<F>(&self, _: F) where F: FnMut(usize, OutputKind) {}

		fn fill<F>(self, oracle: F) -> Result<Self::Complete, NoSuchOutput>
			where F: Fn(usize, usize) -> Result<Output, NoSuchOutput>
		{
			let block_hash = match self.block_hash {
				Field::Scalar(hash) => hash,
				Field::BackReference(req, idx) => match oracle(req, idx)? {
					Output::Hash(hash) => hash,
					_ => return Err(NoSuchOutput)?,
				}
			};

			let code_hash = match self.code_hash {
				Field::Scalar(hash) => hash,
				Field::BackReference(req, idx) => match oracle(req, idx)? {
					Output::Hash(hash) => hash,
					_ => return Err(NoSuchOutput)?,
				}
			};

			Ok(Complete {
				block_hash: block_hash,
				code_hash: code_hash,
			})
		}

	}

	/// A complete request.
	#[derive(Debug, Clone, PartialEq, Eq)]
	pub struct Complete {
		/// The block hash to request the state for.
		pub block_hash: H256,
		/// The code hash.
		pub code_hash: H256,
	}

	/// The output of a request for
	#[derive(Debug, Clone, PartialEq, Eq)]
	pub struct Response {
		/// The requested code.
		pub code: Bytes,
	}

	impl Response {
		/// Fill reusable outputs by providing them to the function.
		pub fn fill_outputs<F>(&self, _: F) where F: FnMut(usize, Output) {}
	}
}
