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

use std::sync::Arc;
use std::ops::Deref;
use v1::helpers::signing_queue::{ConfirmationsQueue};

/// Manages communication with Signer crate
pub struct SignerService {
	queue: Arc<ConfirmationsQueue>,
	generate_new_token: Box<Fn() -> Result<String, String> + Send + Sync + 'static>,
}

impl SignerService {

	/// Creates new Signer Service given function to generate new tokens.
	pub fn new<F>(new_token: F) -> Self
		where F: Fn() -> Result<String, String> + Send + Sync + 'static {
		SignerService {
			queue: Arc::new(ConfirmationsQueue::default()),
			generate_new_token: Box::new(new_token),
		}
	}

	/// Generates new token.
	pub fn generate_token(&self) -> Result<String, String> {
		(self.generate_new_token)()
	}

	/// Returns a reference to `ConfirmationsQueue`
	pub fn queue(&self) -> Arc<ConfirmationsQueue> {
		self.queue.clone()
	}

	#[cfg(test)]
	/// Creates new Signer Service for tests.
	pub fn new_test() -> Self {
		SignerService::new(|| Ok("new_token".into()))
	}
}

impl Deref for SignerService {
	type Target = ConfirmationsQueue;
	fn deref(&self) -> &Self::Target {
		&self.queue
	}
}

