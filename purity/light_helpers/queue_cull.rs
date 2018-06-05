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

//! Service for culling the light client's transaction queue.

use std::sync::Arc;
use std::time::Duration;

use ethcore::client::ClientIoMessage;
use sync::LightSync;
use io::{IoContext, IoHandler, TimerToken};

use light::client::LightChainClient;
use light::on_demand::{request, OnDemand};
use light::TransactionQueue;

use futures::{future, Future};

use parity_reactor::Remote;

use parking_lot::RwLock;

// Attepmt to cull once every 10 minutes.
const TOKEN: TimerToken = 1;
const TIMEOUT: Duration = Duration::from_secs(60 * 10);

// But make each attempt last only 9 minutes
const PURGE_TIMEOUT: Duration = Duration::from_secs(60 * 9);

/// Periodically culls the transaction queue of mined transactions.
pub struct QueueCull<T> {
	/// A handle to the client, for getting the latest block header.
	pub client: Arc<T>,
	/// A handle to the sync service.
	pub sync: Arc<LightSync>,
	/// The on-demand request service.
	pub on_demand: Arc<OnDemand>,
	/// The transaction queue.
	pub txq: Arc<RwLock<TransactionQueue>>,
	/// Event loop remote.
	pub remote: Remote,
}

impl<T: LightChainClient + 'static> IoHandler<ClientIoMessage> for QueueCull<T> {
	fn initialize(&self, io: &IoContext<ClientIoMessage>) {
		io.register_timer(TOKEN, TIMEOUT).expect("Error registering timer");
	}

	fn timeout(&self, _io: &IoContext<ClientIoMessage>, timer: TimerToken) {
		if timer != TOKEN { return }

		let senders = self.txq.read().queued_senders();
		if senders.is_empty() { return }

		let (sync, on_demand, txq) = (self.sync.clone(), self.on_demand.clone(), self.txq.clone());
		let best_header = self.client.best_block_header();
		let start_nonce = self.client.engine().account_start_nonce(best_header.number());

		info!(target: "cull", "Attempting to cull queued transactions from {} senders.", senders.len());
		self.remote.spawn_with_timeout(move |_| {
			let maybe_fetching = sync.with_context(move |ctx| {
				// fetch the nonce of each sender in the queue.
				let nonce_reqs = senders.iter()
					.map(|&address| request::Account { header: best_header.clone().into(), address: address })
					.collect::<Vec<_>>();

				// when they come in, update each sender to the new nonce.
				on_demand.request(ctx, nonce_reqs)
					.expect("No back-references; therefore all back-references are valid; qed")
					.map(move |accs| {
						let txq = txq.write();
						let _ = accs.into_iter()
							.map(|maybe_acc| maybe_acc.map_or(start_nonce, |acc| acc.nonce))
							.zip(senders)
							.fold(txq, |mut txq, (nonce, addr)| {
								txq.cull(addr, nonce);
								txq
							});
					})
					.map_err(|_| debug!(target: "cull", "OnDemand prematurely closed channel."))
			});

			match maybe_fetching {
				Some(fut) => future::Either::A(fut),
				None => {
					debug!(target: "cull", "Unable to acquire network context; qed");
					future::Either::B(future::ok(()))
				},
			}
		}, PURGE_TIMEOUT, || {})
	}
}
