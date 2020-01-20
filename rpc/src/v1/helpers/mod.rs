// Copyright 2015-2020 Parity Technologies (UK) Ltd.
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

#[macro_use]
pub mod errors;

pub mod block_import;
pub mod deprecated;
pub mod dispatch;
#[cfg(any(test, feature = "accounts"))]
pub mod eip191;
#[cfg(any(test, feature = "accounts"))]
pub mod engine_signer;
pub mod external_signer;
pub mod fake_sign;
pub mod ipfs;
pub mod light_fetch;
pub mod nonce;
#[cfg(any(test, feature = "accounts"))]
pub mod secretstore;

mod network_settings;
mod poll_filter;
mod poll_manager;
mod requests;
mod subscribers;
mod subscription_manager;
mod work;
mod signature;

pub use self::dispatch::{Dispatcher, FullDispatcher, LightDispatcher};
pub use self::signature::verify_signature;
pub use self::network_settings::NetworkSettings;
pub use self::poll_manager::PollManager;
pub use self::poll_filter::{PollFilter, SyncPollFilter, limit_logs};
pub use self::requests::{
	TransactionRequest, FilledTransactionRequest, ConfirmationRequest, ConfirmationPayload, CallRequest,
};
pub use self::subscribers::Subscribers;
pub use self::subscription_manager::GenericPollManager;
pub use self::work::submit_work_detail;

pub fn to_url(address: &Option<::Host>) -> Option<String> {
	address.as_ref().map(|host| (**host).to_owned())
}
