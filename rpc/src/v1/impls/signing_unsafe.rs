// Copyright 2015, 2016 Ethcore (UK) Ltd.
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

//! Unsafe Signing RPC implementation.

use std::sync::{Arc, Weak};

use ethcore::account_provider::AccountProvider;
use ethcore::miner::MinerService;
use ethcore::client::MiningBlockChainClient;

use jsonrpc_core::*;
use v1::helpers::errors;
use v1::helpers::dispatch::{sign_and_dispatch, sign, decrypt};
use v1::traits::{EthSigning, ParitySigning};
use v1::types::{TransactionRequest, H160 as RpcH160, H256 as RpcH256, Bytes as RpcBytes};

/// Implementation of functions that require signing when no trusted signer is used.
pub struct SigningUnsafeClient<C, M> where
	C: MiningBlockChainClient,
	M: MinerService,
{
	accounts: Weak<AccountProvider>,
	client: Weak<C>,
	miner: Weak<M>,
}

impl<C, M> SigningUnsafeClient<C, M> where
	C: MiningBlockChainClient,
	M: MinerService,
{

	/// Creates new SigningUnsafeClient.
	pub fn new(client: &Arc<C>, accounts: &Arc<AccountProvider>, miner: &Arc<M>)
		-> Self {
		SigningUnsafeClient {
			client: Arc::downgrade(client),
			miner: Arc::downgrade(miner),
			accounts: Arc::downgrade(accounts),
		}
	}

	fn active(&self) -> Result<(), Error> {
		// TODO: only call every 30s at most.
		take_weak!(self.client).keep_alive();
		Ok(())
	}
}

impl<C: 'static, M: 'static> EthSigning for SigningUnsafeClient<C, M> where
	C: MiningBlockChainClient,
	M: MinerService,
{
	fn sign(&self, params: Params, ready: Ready) {
		ready.ready(self.active()
			.and_then(|_| from_params::<(RpcH160, RpcH256)>(params))
			.and_then(|(address, msg)| {
				sign(&*take_weak!(self.accounts), address.into(), None, msg.into())
			}))
	}

	fn send_transaction(&self, params: Params, ready: Ready) {
		ready.ready(self.active()
			.and_then(|_| from_params::<(TransactionRequest, )>(params))
			.and_then(|(request, )| {
				sign_and_dispatch(&*take_weak!(self.client), &*take_weak!(self.miner), &*take_weak!(self.accounts), request.into(), None).map(to_value)
			}))
	}
}

impl<C: 'static, M: 'static> ParitySigning for SigningUnsafeClient<C, M> where
	C: MiningBlockChainClient,
	M: MinerService,
{
	fn decrypt_message(&self, params: Params, ready: Ready) {
		ready.ready(self.active()
			.and_then(|_| from_params::<(RpcH160, RpcBytes)>(params))
			.and_then(|(address, ciphertext)| {
				decrypt(&*take_weak!(self.accounts), address.into(), None, ciphertext.0)
			}))
	}

	fn post_sign(&self, _: Params) -> Result<Value, Error> {
		// We don't support this in non-signer mode.
		Err(errors::signer_disabled())
	}

	fn post_transaction(&self, _: Params) -> Result<Value, Error> {
		// We don't support this in non-signer mode.
		Err(errors::signer_disabled())
	}

	fn check_request(&self, _: Params) -> Result<Value, Error> {
		// We don't support this in non-signer mode.
		Err(errors::signer_disabled())
	}
}
