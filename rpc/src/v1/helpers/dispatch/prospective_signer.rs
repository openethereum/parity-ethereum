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

use ethereum_types::U256;
use jsonrpc_core::{Result, Error};
use jsonrpc_core::futures::{Future, Poll, Async, IntoFuture};
use types::transaction::SignedTransaction;

use v1::helpers::{errors, nonce, FilledTransactionRequest};
use super::{Accounts, SignWith, WithToken, PostSign};

#[derive(Debug, Clone, Copy)]
enum ProspectiveSignerState {
	TryProspectiveSign,
	WaitForPostSign,
	WaitForNonce,
}

pub struct ProspectiveSigner<P: PostSign> {
	signer: Arc<dyn Accounts>,
	filled: FilledTransactionRequest,
	chain_id: Option<u64>,
	reserved: nonce::Reserved,
	password: SignWith,
	state: ProspectiveSignerState,
	prospective: Option<WithToken<SignedTransaction>>,
	ready: Option<nonce::Ready>,
	post_sign: Option<P>,
	post_sign_future: Option<<P::Out as IntoFuture>::Future>
}

impl<P: PostSign> ProspectiveSigner<P> {
	pub fn new(
		signer: Arc<dyn Accounts>,
		filled: FilledTransactionRequest,
		chain_id: Option<u64>,
		reserved: nonce::Reserved,
		password: SignWith,
		post_sign: P
	) -> Self {
		let supports_prospective = signer.supports_prospective_signing(&filled.from, &password);

		ProspectiveSigner {
			signer,
			filled,
			chain_id,
			reserved,
			password,
			state: if supports_prospective {
				ProspectiveSignerState::TryProspectiveSign
			} else {
				ProspectiveSignerState::WaitForNonce
			},
			prospective: None,
			ready: None,
			post_sign: Some(post_sign),
			post_sign_future: None
		}
	}

	fn sign(&self, nonce: &U256) -> Result<WithToken<SignedTransaction>> {
		self.signer.sign_transaction(
			self.filled.clone(),
			self.chain_id,
			*nonce,
			self.password.clone()
		)
	}

	fn poll_reserved(&mut self) -> Poll<nonce::Ready, Error> {
		self.reserved.poll().map_err(|_| errors::internal("Nonce reservation failure", ""))
	}
}

impl<P: PostSign> Future for ProspectiveSigner<P> {
	type Item = P::Item;
	type Error = Error;

	fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
		use self::ProspectiveSignerState::*;

		loop {
			match self.state {
				TryProspectiveSign => {
					// Try to poll reserved, it might be ready.
					match self.poll_reserved()? {
						Async::NotReady => {
							self.state = WaitForNonce;
							self.prospective = Some(self.sign(self.reserved.prospective_value())?);
						},
						Async::Ready(nonce) => {
							self.state = WaitForPostSign;
							self.post_sign_future = Some(
								self.post_sign.take()
									.expect("post_sign is set on creation; qed")
									.execute(self.sign(nonce.value())?)
									.into_future()
							);
							self.ready = Some(nonce);
						},
					}
				},
				WaitForNonce => {
					let nonce = try_ready!(self.poll_reserved());
					let prospective = match (self.prospective.take(), nonce.matches_prospective()) {
						(Some(prospective), true) => prospective,
						_ => self.sign(nonce.value())?,
					};
					self.ready = Some(nonce);
					self.state = WaitForPostSign;
					self.post_sign_future = Some(self.post_sign.take()
						.expect("post_sign is set on creation; qed")
						.execute(prospective)
						.into_future());
				},
				WaitForPostSign => {
					if let Some(fut) = self.post_sign_future.as_mut() {
						match fut.poll()? {
							Async::Ready(item) => {
								let nonce = self.ready
									.take()
									.expect("nonce is set before state transitions to WaitForPostSign; qed");
								nonce.mark_used();
								return Ok(Async::Ready(item))
							},
							Async::NotReady => {
								return Ok(Async::NotReady)
							}
						}
					} else {
						panic!("Poll after ready.");
					}
				}
			}
		}
	}
}
