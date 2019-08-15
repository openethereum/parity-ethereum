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

use std::sync::Arc;

use client_traits::BlockChainClient;
use ethcore::miner::{self, MinerService};
use ethereum_types::{H256, U256, Address};
use types::transaction::{SignedTransaction, PendingTransaction};
use parking_lot::Mutex;

use jsonrpc_core::{BoxFuture, Result};
use jsonrpc_core::futures::{future, Future, IntoFuture};
use v1::helpers::{errors, nonce, TransactionRequest, FilledTransactionRequest};
use v1::types::{RichRawTransaction as RpcRichRawTransaction};

use super::prospective_signer::ProspectiveSigner;
use super::{Dispatcher, Accounts, SignWith, PostSign, default_gas_price};

/// A dispatcher which uses references to a client and miner in order to sign
/// requests locally.
#[derive(Debug)]
pub struct FullDispatcher<C, M> {
	client: Arc<C>,
	miner: Arc<M>,
	nonces: Arc<Mutex<nonce::Reservations>>,
	gas_price_percentile: usize,
}

impl<C, M> FullDispatcher<C, M> {
	/// Create a `FullDispatcher` from Arc references to a client and miner.
	pub fn new(
		client: Arc<C>,
		miner: Arc<M>,
		nonces: Arc<Mutex<nonce::Reservations>>,
		gas_price_percentile: usize,
	) -> Self {
		FullDispatcher {
			client,
			miner,
			nonces,
			gas_price_percentile,
		}
	}
}

impl<C, M> Clone for FullDispatcher<C, M> {
	fn clone(&self) -> Self {
		FullDispatcher {
			client: self.client.clone(),
			miner: self.miner.clone(),
			nonces: self.nonces.clone(),
			gas_price_percentile: self.gas_price_percentile,
		}
	}
}

impl<C: miner::BlockChainClient, M: MinerService> FullDispatcher<C, M> {
	fn state_nonce(&self, from: &Address) -> U256 {
		self.miner.next_nonce(&*self.client, from)
	}

	/// Post transaction to the network.
	///
	/// If transaction is trusted we are more likely to assume it is coming from a local account.
	pub fn dispatch_transaction(client: &C, miner: &M, signed_transaction: PendingTransaction, trusted: bool) -> Result<H256> {
		let hash = signed_transaction.transaction.hash();

		// use `import_claimed_local_transaction` so we can decide (based on config flags) if we want to treat
		// it as local or not. Nodes with public RPC interfaces will want these transactions to be treated like
		// external transactions.
		miner.import_claimed_local_transaction(client, signed_transaction, trusted)
			.map_err(errors::transaction)
			.map(|_| hash)
	}
}

impl<C: miner::BlockChainClient + BlockChainClient, M: MinerService> Dispatcher for FullDispatcher<C, M> {
	fn fill_optional_fields(&self, request: TransactionRequest, default_sender: Address, force_nonce: bool)
		-> BoxFuture<FilledTransactionRequest>
	{
		let request = request;
		let from = request.from.unwrap_or(default_sender);
		let nonce = if force_nonce {
			request.nonce.or_else(|| Some(self.state_nonce(&from)))
		} else {
			request.nonce
		};

		Box::new(future::ok(FilledTransactionRequest {
			from,
			used_default_from: request.from.is_none(),
			to: request.to,
			nonce,
			gas_price: request.gas_price.unwrap_or_else(|| {
				default_gas_price(&*self.client, &*self.miner, self.gas_price_percentile)
			}),
			gas: request.gas.unwrap_or_else(|| self.miner.sensible_gas_limit()),
			value: request.value.unwrap_or_else(|| 0.into()),
			data: request.data.unwrap_or_else(Vec::new),
			condition: request.condition,
		}))
	}

	fn sign<P>(
		&self,
		filled: FilledTransactionRequest,
		signer: &Arc<Accounts>,
		password: SignWith,
		post_sign: P,
	) -> BoxFuture<P::Item>
		where
			P: PostSign + 'static,
		    <P::Out as IntoFuture>::Future: Send,
	{
		let chain_id = self.client.signing_chain_id();

		if let Some(nonce) = filled.nonce {
			let future = signer.sign_transaction(filled, chain_id, nonce, password)
				.into_future()
				.and_then(move |signed| post_sign.execute(signed));
			Box::new(future)
		} else {
			let state = self.state_nonce(&filled.from);
			let reserved = self.nonces.lock().reserve(filled.from, state);

			Box::new(ProspectiveSigner::new(signer.clone(), filled, chain_id, reserved, password, post_sign))
		}
	}

	fn enrich(&self, signed_transaction: SignedTransaction) -> RpcRichRawTransaction {
		RpcRichRawTransaction::from_signed(signed_transaction)
	}

	fn dispatch_transaction(&self, signed_transaction: PendingTransaction) -> Result<H256> {
		Self::dispatch_transaction(&*self.client, &*self.miner, signed_transaction, true)
	}
}
