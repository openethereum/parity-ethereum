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

/// Parity-specific rpc interface for operations altering the settings.
use std::io;
use std::sync::Arc;
use std::time::Duration;

use client_traits::BlockChainClient;
use types::client_types::Mode;
use ethcore::miner::{self, MinerService};
use ethereum_types::{H160, H256, U256};
use ethkey;
use fetch::{self, Fetch};
use hash::keccak_buffer;
use sync::ManageNetwork;
use updater::{Service as UpdateService};

use jsonrpc_core::{BoxFuture, Result};
use jsonrpc_core::futures::Future;
use v1::helpers::errors;
use v1::traits::ParitySet;
use v1::types::{Bytes, ReleaseInfo, Transaction};

#[cfg(any(test, feature = "accounts"))]
pub mod accounts {
	use super::*;
	use accounts::AccountProvider;
	use v1::traits::ParitySetAccounts;
	use v1::helpers::deprecated::DeprecationNotice;
	use v1::helpers::engine_signer::EngineSigner;

	/// Parity-specific account-touching RPC interfaces.
	pub struct ParitySetAccountsClient<M> {
		miner: Arc<M>,
		accounts: Arc<AccountProvider>,
		deprecation_notice: DeprecationNotice,
	}

	impl<M> ParitySetAccountsClient<M> {
		/// Creates new ParitySetAccountsClient
		pub fn new(
			accounts: &Arc<AccountProvider>,
			miner: &Arc<M>,
		) -> Self {
			ParitySetAccountsClient {
				accounts: accounts.clone(),
				miner: miner.clone(),
				deprecation_notice: Default::default(),
			}
		}
	}

	impl<M: MinerService + 'static> ParitySetAccounts for ParitySetAccountsClient<M> {
		fn set_engine_signer(&self, address: H160, password: String) -> Result<bool> {
			self.deprecation_notice.print(
				"parity_setEngineSigner",
				"use `parity_setEngineSignerSecret` instead. See #9997 for context."
			);

			let signer = Box::new(EngineSigner::new(
				self.accounts.clone(),
				address.clone().into(),
				password.into(),
			));
			self.miner.set_author(miner::Author::Sealer(signer));
			Ok(true)
		}
	}
}

/// Parity-specific rpc interface for operations altering the settings.
pub struct ParitySetClient<C, M, U, F = fetch::Client> {
	client: Arc<C>,
	miner: Arc<M>,
	updater: Arc<U>,
	net: Arc<dyn ManageNetwork>,
	fetch: F,
}

impl<C, M, U, F> ParitySetClient<C, M, U, F>
	where C: BlockChainClient + 'static,
{
	/// Creates new `ParitySetClient` with given `Fetch`.
	pub fn new(
		client: &Arc<C>,
		miner: &Arc<M>,
		updater: &Arc<U>,
		net: &Arc<dyn ManageNetwork>,
		fetch: F,
	) -> Self {
		ParitySetClient {
			client: client.clone(),
			miner: miner.clone(),
			updater: updater.clone(),
			net: net.clone(),
			fetch,
		}
	}
}

impl<C, M, U, F> ParitySet for ParitySetClient<C, M, U, F> where
	C: BlockChainClient + 'static,
	M: MinerService + 'static,
	U: UpdateService + 'static,
	F: Fetch + 'static,
{

	fn set_min_gas_price(&self, gas_price: U256) -> Result<bool> {
		match self.miner.set_minimal_gas_price(gas_price) {
			Ok(success) => Ok(success),
			Err(e) => Err(errors::unsupported(e, None)),
		}
	}

	fn set_transactions_limit(&self, _limit: usize) -> Result<bool> {
		warn!("setTransactionsLimit is deprecated. Ignoring request.");
		Ok(false)
	}

	fn set_tx_gas_limit(&self, _limit: U256) -> Result<bool> {
		warn!("setTxGasLimit is deprecated. Ignoring request.");
		Ok(false)
	}

	fn set_gas_floor_target(&self, target: U256) -> Result<bool> {
		let mut range = self.miner.authoring_params().gas_range_target;
		range.0 = target;
		self.miner.set_gas_range_target(range);
		Ok(true)
	}

	fn set_gas_ceil_target(&self, target: U256) -> Result<bool> {
		let mut range = self.miner.authoring_params().gas_range_target;
		range.1 = target;
		self.miner.set_gas_range_target(range);
		Ok(true)
	}

	fn set_extra_data(&self, extra_data: Bytes) -> Result<bool> {
		self.miner.set_extra_data(extra_data.into_vec());
		Ok(true)
	}

	fn set_author(&self, address: H160) -> Result<bool> {
		self.miner.set_author(miner::Author::External(address));
		Ok(true)
	}

	fn set_engine_signer_secret(&self, secret: H256) -> Result<bool> {
		let keypair = ethkey::KeyPair::from_secret(secret.into()).map_err(|e| errors::account("Invalid secret", e))?;
		self.miner.set_author(miner::Author::Sealer(engine::signer::from_keypair(keypair)));
		Ok(true)
	}

	fn clear_engine_signer(&self) -> Result<bool> {
		self.miner.set_author(None);
		Ok(true)
	}

	fn add_reserved_peer(&self, peer: String) -> Result<bool> {
		match self.net.add_reserved_peer(peer) {
			Ok(()) => Ok(true),
			Err(e) => Err(errors::invalid_params("Peer address", e)),
		}
	}

	fn remove_reserved_peer(&self, peer: String) -> Result<bool> {
		match self.net.remove_reserved_peer(peer) {
			Ok(()) => Ok(true),
			Err(e) => Err(errors::invalid_params("Peer address", e)),
		}
	}

	fn drop_non_reserved_peers(&self) -> Result<bool> {
		self.net.deny_unreserved_peers();
		Ok(true)
	}

	fn accept_non_reserved_peers(&self) -> Result<bool> {
		self.net.accept_unreserved_peers();
		Ok(true)
	}

	fn start_network(&self) -> Result<bool> {
		self.net.start_network();
		Ok(true)
	}

	fn stop_network(&self) -> Result<bool> {
		self.net.stop_network();
		Ok(true)
	}

	fn set_mode(&self, mode: String) -> Result<bool> {
		self.client.set_mode(match mode.as_str() {
			"offline" => Mode::Off,
			"dark" => Mode::Dark(Duration::from_secs(300)),
			"passive" => Mode::Passive(Duration::from_secs(300), Duration::from_secs(3600)),
			"active" => Mode::Active,
			e => { return Err(errors::invalid_params("mode", e.to_owned())); },
		});
		Ok(true)
	}

	fn set_spec_name(&self, spec_name: String) -> Result<bool> {
		self.client.set_spec_name(spec_name).map(|_| true).map_err(|()| errors::cannot_restart())
	}

	fn hash_content(&self, url: String) -> BoxFuture<H256> {
		let future = self.fetch.get(&url, Default::default()).then(move |result| {
			result
				.map_err(errors::fetch)
				.and_then(move |response| {
					let mut reader = io::BufReader::new(fetch::BodyReader::new(response));
					keccak_buffer(&mut reader).map_err(errors::fetch)
				})
				.map(Into::into)
		});
		Box::new(future)
	}

	fn upgrade_ready(&self) -> Result<Option<ReleaseInfo>> {
		Ok(self.updater.upgrade_ready().map(Into::into))
	}

	fn execute_upgrade(&self) -> Result<bool> {
		Ok(self.updater.execute_upgrade())
	}

	fn remove_transaction(&self, hash: H256) -> Result<Option<Transaction>> {
		Ok(self.miner.remove_transaction(&hash)
			.map(|t| Transaction::from_pending(t.pending().clone()))
		)
	}
}
