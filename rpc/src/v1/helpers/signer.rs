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
use std::ops::Deref;
use http::Origin;
use parking_lot::Mutex;
use transient_hashmap::TransientHashMap;

use ethstore::random_string;

use v1::helpers::signing_queue::{ConfirmationsQueue};

const TOKEN_LIFETIME_SECS: u32 = 3600;

/// Manages communication with Signer crate
pub struct SignerService {
	is_enabled: bool,
	queue: Arc<ConfirmationsQueue>,
	web_proxy_tokens: Mutex<TransientHashMap<String, Origin>>,
	generate_new_token: Box<Fn() -> Result<String, String> + Send + Sync + 'static>,
}

impl SignerService {
	/// Creates new Signer Service given function to generate new tokens.
	pub fn new<F>(new_token: F, is_enabled: bool) -> Self
		where F: Fn() -> Result<String, String> + Send + Sync + 'static {
		SignerService {
			queue: Arc::new(ConfirmationsQueue::default()),
			web_proxy_tokens: Mutex::new(TransientHashMap::new(TOKEN_LIFETIME_SECS)),
			generate_new_token: Box::new(new_token),
			is_enabled: is_enabled,
		}
	}

	/// Checks if the token is valid web proxy access token.
	pub fn web_proxy_access_token_domain(&self, token: &String) -> Option<Origin> {
		self.web_proxy_tokens.lock().get(token).cloned()
	}

	/// Generates a new web proxy access token.
	pub fn generate_web_proxy_access_token(&self, domain: Origin) -> String {
		let token = random_string(16);
		let mut tokens = self.web_proxy_tokens.lock();
		tokens.prune();
		tokens.insert(token.clone(), domain);
		token
	}

	/// Generates new signer authorization token.
	pub fn generate_token(&self) -> Result<String, String> {
		(self.generate_new_token)()
	}

	/// Returns a reference to `ConfirmationsQueue`
	pub fn queue(&self) -> Arc<ConfirmationsQueue> {
		self.queue.clone()
	}

	/// Returns true if Signer is enabled.
	pub fn is_enabled(&self) -> bool {
		self.is_enabled
	}

	#[cfg(test)]
	/// Creates new Signer Service for tests.
	pub fn new_test(is_enabled: bool) -> Self {
		SignerService::new(|| Ok("new_token".into()), is_enabled)
	}
}

impl Deref for SignerService {
	type Target = ConfirmationsQueue;
	fn deref(&self) -> &Self::Target {
		&self.queue
	}
}
