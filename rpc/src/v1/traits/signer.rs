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

//! Parity Signer-related rpc interface.
use jsonrpc_core::{BoxFuture, Result};
use jsonrpc_pubsub::{typed::Subscriber, SubscriptionId};
use jsonrpc_derive::rpc;

use v1::types::{U256, Bytes, TransactionModification, ConfirmationRequest, ConfirmationResponse, ConfirmationResponseWithToken};

/// Signer extension for confirmations rpc interface.
#[rpc]
pub trait Signer {
	/// RPC Metadata
	type Metadata;

	/// Returns a list of items to confirm.
	#[rpc(name = "signer_requestsToConfirm")]
	fn requests_to_confirm(&self) -> Result<Vec<ConfirmationRequest>>;

	/// Confirm specific request.
	#[rpc(name = "signer_confirmRequest")]
	fn confirm_request(&self, U256, TransactionModification, String) -> BoxFuture<ConfirmationResponse>;

	/// Confirm specific request with token.
	#[rpc(name = "signer_confirmRequestWithToken")]
	fn confirm_request_with_token(&self, U256, TransactionModification, String) -> BoxFuture<ConfirmationResponseWithToken>;

	/// Confirm specific request with already signed data.
	#[rpc(name = "signer_confirmRequestRaw")]
	fn confirm_request_raw(&self, U256, Bytes) -> Result<ConfirmationResponse>;

	/// Reject the confirmation request.
	#[rpc(name = "signer_rejectRequest")]
	fn reject_request(&self, U256) -> Result<bool>;

	/// Generates new authorization token.
	#[rpc(name = "signer_generateAuthorizationToken")]
	fn generate_token(&self) -> Result<String>;

	/// Subscribe to new pending requests on signer interface.
	#[pubsub(subscription = "signer_pending", subscribe, name = "signer_subscribePending")]
	fn subscribe_pending(&self, Self::Metadata, Subscriber<Vec<ConfirmationRequest>>);

	/// Unsubscribe from pending requests subscription.
	#[pubsub(subscription = "signer_pending", unsubscribe, name = "signer_unsubscribePending")]
	fn unsubscribe_pending(&self, Option<Self::Metadata>, SubscriptionId) -> Result<bool>;
}
