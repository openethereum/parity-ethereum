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

//! Unsafe Signing RPC implementation.

use std::sync::{Arc, Weak};
use util::Hashable;

use ethcore::account_provider::AccountProvider;
use ethcore::miner::MinerService;
use ethcore::client::MiningBlockChainClient;

use futures::{self, BoxFuture, Future};
use jsonrpc_core::Error;
use v1::helpers::errors;
use v1::helpers::dispatch;
use v1::traits::{EthSigning, ParitySigning};
use v1::types::{
	U256 as RpcU256,
	H160 as RpcH160, H256 as RpcH256, H520 as RpcH520, Bytes as RpcBytes,
	Either as RpcEither,
	RichRawTransaction as RpcRichRawTransaction,
	TransactionRequest as RpcTransactionRequest,
	ConfirmationPayload as RpcConfirmationPayload,
	ConfirmationResponse as RpcConfirmationResponse,
};

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

	fn handle(&self, payload: RpcConfirmationPayload) -> Result<RpcConfirmationResponse, Error> {
		self.active()?;
		let client = take_weak!(self.client);
		let miner = take_weak!(self.miner);
		let accounts = take_weak!(self.accounts);

		let payload = dispatch::from_rpc(payload, &*client, &*miner);
		dispatch::execute(&*client, &*miner, &*accounts, payload, dispatch::SignWith::Nothing)
			.map(|v| v.into_value())
	}
}

impl<C: 'static, M: 'static> EthSigning for SigningUnsafeClient<C, M> where
	C: MiningBlockChainClient,
	M: MinerService,
{
	fn sign(&self, address: RpcH160, data: RpcBytes) -> BoxFuture<RpcH520, Error> {
		let hash = data.0.sha3().into();
		let result = match self.handle(RpcConfirmationPayload::Signature((address, hash).into())) {
			Ok(RpcConfirmationResponse::Signature(signature)) => Ok(signature),
			Err(e) => Err(e),
			e => Err(errors::internal("Unexpected result", e)),
		};

		futures::done(result).boxed()
	}

	fn send_transaction(&self, request: RpcTransactionRequest) -> BoxFuture<RpcH256, Error> {
		let result = match self.handle(RpcConfirmationPayload::SendTransaction(request)) {
			Ok(RpcConfirmationResponse::SendTransaction(hash)) => Ok(hash),
			Err(e) => Err(e),
			e => Err(errors::internal("Unexpected result", e)),
		};

		futures::done(result).boxed()
	}

	fn sign_transaction(&self, request: RpcTransactionRequest) -> BoxFuture<RpcRichRawTransaction, Error> {
		let result = match self.handle(RpcConfirmationPayload::SignTransaction(request)) {
			Ok(RpcConfirmationResponse::SignTransaction(tx)) => Ok(tx),
			Err(e) => Err(e),
			e => Err(errors::internal("Unexpected result", e)),
		};

		futures::done(result).boxed()
	}
}

impl<C: 'static, M: 'static> ParitySigning for SigningUnsafeClient<C, M> where
	C: MiningBlockChainClient,
	M: MinerService,
{
	fn decrypt_message(&self, address: RpcH160, data: RpcBytes) -> BoxFuture<RpcBytes, Error> {
		let result = match self.handle(RpcConfirmationPayload::Decrypt((address, data).into())) {
			Ok(RpcConfirmationResponse::Decrypt(data)) => Ok(data),
			Err(e) => Err(e),
			e => Err(errors::internal("Unexpected result", e)),
		};

		futures::done(result).boxed()
	}

	fn post_sign(&self, _: RpcH160, _: RpcH256) -> Result<RpcEither<RpcU256, RpcConfirmationResponse>, Error> {
		// We don't support this in non-signer mode.
		Err(errors::signer_disabled())
	}

	fn post_transaction(&self, _: RpcTransactionRequest) -> Result<RpcEither<RpcU256, RpcConfirmationResponse>, Error> {
		// We don't support this in non-signer mode.
		Err(errors::signer_disabled())
	}

	fn check_request(&self, _: RpcU256) -> Result<Option<RpcConfirmationResponse>, Error> {
		// We don't support this in non-signer mode.
		Err(errors::signer_disabled())
	}
}
