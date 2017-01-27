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

use std::sync::Arc;
use std::ops::Deref;
use util::Mutex;
use transient_hashmap::TransientHashMap;

use ethstore::random_string;

use v1::helpers::signing_queue::{ConfirmationsQueue};

const TOKEN_LIFETIME_SECS: u64 = 3600;

/// Manages communication with Signer crate
pub struct SignerService {
	queue: Arc<ConfirmationsQueue>,
	web_proxy_tokens: Mutex<TransientHashMap<String, ()>>,
	generate_new_token: Box<Fn() -> Result<String, String> + Send + Sync + 'static>,
	address: Option<(String, u16)>,
}

impl SignerService {
	/// Creates new Signer Service given function to generate new tokens.
	pub fn new<F>(new_token: F, address: Option<(String, u16)>) -> Self
		where F: Fn() -> Result<String, String> + Send + Sync + 'static {
		SignerService {
			queue: Arc::new(ConfirmationsQueue::default()),
			web_proxy_tokens: Mutex::new(TransientHashMap::new(TOKEN_LIFETIME_SECS)),
			generate_new_token: Box::new(new_token),
			address: address,
		}
	}

	/// Checks if the token is valid web proxy access token.
	pub fn is_valid_web_proxy_access_token(&self, token: &String) -> bool {
		self.web_proxy_tokens.lock().contains_key(&token)
	}

	/// Generates a new web proxy access token.
	pub fn generate_web_proxy_access_token(&self) -> String {
		let token = random_string(16);
		let mut tokens = self.web_proxy_tokens.lock();
		tokens.prune();
		tokens.insert(token.clone(), ());
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

	/// Returns signer address (if signer enabled) or `None` otherwise
	pub fn address(&self) -> Option<(String, u16)> {
		self.address.clone()
	}

	/// Returns true if Signer is enabled.
	pub fn is_enabled(&self) -> bool {
		self.address.is_some()
	}

	#[cfg(test)]
	/// Creates new Signer Service for tests.
	pub fn new_test(address: Option<(String, u16)>) -> Self {
		SignerService::new(|| Ok("new_token".into()), address)
	}
}

impl Deref for SignerService {
	type Target = ConfirmationsQueue;
	fn deref(&self) -> &Self::Target {
		&self.queue
	}
}

