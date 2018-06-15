// Copyright 2015-2018 Parity Technologies (UK) Ltd.
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

//! Trait for fetching chain data.

use std::sync::Arc;

use ethcore::encoded;
use ethcore::engines::{EthEngine, StateDependentProof};
use ethcore::machine::EthereumMachine;
use ethcore::header::Header;
use ethcore::receipt::Receipt;
use futures::future::IntoFuture;
use ethereum_types::H256;

/// Provides full chain data.
pub trait ChainDataFetcher: Send + Sync + 'static {
	/// Error type when data unavailable.
	type Error: ::std::fmt::Debug;

	/// Future for fetching block body.
	type Body: IntoFuture<Item=encoded::Block, Error=Self::Error>;
	/// Future for fetching block receipts.
	type Receipts: IntoFuture<Item=Vec<Receipt>, Error=Self::Error>;
	/// Future for fetching epoch transition
	type Transition: IntoFuture<Item=Vec<u8>, Error=Self::Error>;

	/// Fetch a block body.
	fn block_body(&self, header: &Header) -> Self::Body;

	/// Fetch block receipts.
	fn block_receipts(&self, header: &Header) -> Self::Receipts;

	/// Fetch epoch transition proof at given header.
	fn epoch_transition(
		&self,
		_hash: H256,
		_engine: Arc<EthEngine>,
		_checker: Arc<StateDependentProof<EthereumMachine>>
	) -> Self::Transition;
}

/// Fetcher implementation which cannot fetch anything.
pub struct Unavailable;

/// Create a fetcher which has all data unavailable.
pub fn unavailable() -> Unavailable { Unavailable }

impl ChainDataFetcher for Unavailable {
	type Error = &'static str;

	type Body = Result<encoded::Block, &'static str>;
	type Receipts = Result<Vec<Receipt>, &'static str>;
	type Transition = Result<Vec<u8>, &'static str>;

	fn block_body(&self, _header: &Header) -> Self::Body {
		Err("fetching block bodies unavailable")
	}

	fn block_receipts(&self, _header: &Header) -> Self::Receipts {
		Err("fetching block receipts unavailable")
	}

	fn epoch_transition(
		&self,
		_hash: H256,
		_engine: Arc<EthEngine>,
		_checker: Arc<StateDependentProof<EthereumMachine>>
	) -> Self::Transition {
		Err("fetching epoch transition proofs unavailable")
	}
}
