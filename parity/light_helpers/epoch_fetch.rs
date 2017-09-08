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

use std::sync::{Arc, Weak};

use ethcore::encoded;
use ethcore::engines::{Engine, StateDependentProof};
use ethcore::header::Header;
use ethcore::receipt::Receipt;
use ethsync::LightSync;

use futures::{future, Future, BoxFuture};

use light::client::fetch::ChainDataFetcher;
use light::on_demand::{request, OnDemand};

use parking_lot::RwLock;
use bigint::hash::H256;

const ALL_VALID_BACKREFS: &str = "no back-references, therefore all back-references valid; qed";

/// Allows on-demand fetch of data useful for the light client.
pub struct EpochFetch {
	/// A handle to the sync service.
	pub sync: Arc<RwLock<Weak<LightSync>>>,
	/// The on-demand request service.
	pub on_demand: Arc<OnDemand>,
}

impl EpochFetch {
	fn request<T>(&self, req: T) -> BoxFuture<T::Out, &'static str>
		where T: Send + request::RequestAdapter + 'static, T::Out: Send + 'static
	{
		match self.sync.read().upgrade() {
			Some(sync) => {
				let on_demand = &self.on_demand;
				let maybe_future = sync.with_context(move |ctx| {
					on_demand.request(ctx, req).expect(ALL_VALID_BACKREFS)
				});

				match maybe_future {
					Some(x) => x.map_err(|_| "Request canceled").boxed(),
					None => future::err("Unable to access network.").boxed(),
				}
			}
			None => future::err("Unable to access network").boxed(),
		}
	}
}

impl ChainDataFetcher for EpochFetch {
	type Error = &'static str;

	type Body = BoxFuture<encoded::Block, &'static str>;
	type Receipts = BoxFuture<Vec<Receipt>, &'static str>;
	type Transition = BoxFuture<Vec<u8>, &'static str>;

	fn block_body(&self, header: &Header) -> Self::Body {
		self.request(request::Body(header.encoded().into()))
	}

	/// Fetch block receipts.
	fn block_receipts(&self, header: &Header) -> Self::Receipts {
		self.request(request::BlockReceipts(header.encoded().into()))
	}

	/// Fetch epoch transition proof at given header.
	fn epoch_transition(&self, hash: H256, engine: Arc<Engine>, checker: Arc<StateDependentProof>)
		-> Self::Transition
	{
		self.request(request::Signal {
			hash: hash,
			engine: engine,
			proof_check: checker,
		})
	}
}
