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
use jsonrpc_core::futures::{Future, Poll, Async};
use types::transaction::SignedTransaction;

use v1::helpers::{errors, nonce, FilledTransactionRequest};
use super::{Accounts, SignWith, WithToken};

#[derive(Debug, Clone, Copy)]
enum ProspectiveSignerState {
	TryProspectiveSign,
	WaitForNonce,
	Finish,
}

pub struct ProspectiveSigner {
	signer: Arc<Accounts>,
	filled: FilledTransactionRequest,
	chain_id: Option<u64>,
	reserved: nonce::Reserved,
	password: SignWith,
	state: ProspectiveSignerState,
	prospective: Option<Result<WithToken<SignedTransaction>>>,
	ready: Option<nonce::Ready>,
}

impl ProspectiveSigner {
	pub fn new(
		signer: Arc<Accounts>,
		filled: FilledTransactionRequest,
		chain_id: Option<u64>,
		reserved: nonce::Reserved,
		password: SignWith,
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

impl Future for ProspectiveSigner {
	type Item = WithToken<SignedTransaction>;
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
							self.prospective = Some(self.sign(self.reserved.prospective_value()));
						},
						Async::Ready(nonce) => {
							self.state = Finish;
							self.prospective = Some(self.sign(nonce.value()));
							self.ready = Some(nonce);
						},
					}
				},
				WaitForNonce => {
					let nonce = try_ready!(self.poll_reserved());
					let result = match (self.prospective.take(), nonce.matches_prospective()) {
						(Some(prospective), true) => prospective,
						_ => self.sign(nonce.value()),
					};
					self.state = Finish;
					self.prospective = Some(result);
					self.ready = Some(nonce);
				},
				Finish => {
					if let (Some(result), Some(nonce)) = (self.prospective.take(), self.ready.take()) {
						// Mark nonce as used on successful signing
						return result.map(move |tx| {
							nonce.mark_used();
							Async::Ready(tx)
						})
					} else {
						panic!("Poll after ready.");
					}
				}
			}
		}
	}
}
