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

use parking_lot::Mutex;

/// Trait which should be implemented by a private transaction handler.
pub trait PrivateTxHandler: Send + Sync + 'static {
	/// Function called on new private transaction received.
	fn import_private_transaction(&self, rlp: &[u8]) -> Result<(), String>;

	/// Function called on new signed private transaction received.
	fn import_signed_private_transaction(&self, rlp: &[u8]) -> Result<(), String>;
}

/// Nonoperative private transaction handler.
pub struct NoopPrivateTxHandler;

impl PrivateTxHandler for NoopPrivateTxHandler {
	fn import_private_transaction(&self, _rlp: &[u8]) -> Result<(), String> {
		Ok(())
	}

	fn import_signed_private_transaction(&self, _rlp: &[u8]) -> Result<(), String> {
		Ok(())
	}
}

/// Simple private transaction handler. Used for tests.
#[derive(Default)]
pub struct SimplePrivateTxHandler {
	/// imported private transactions
	pub txs: Mutex<Vec<Vec<u8>>>,
	/// imported signed private transactions
	pub signed_txs: Mutex<Vec<Vec<u8>>>,
}

impl PrivateTxHandler for SimplePrivateTxHandler {
	fn import_private_transaction(&self, rlp: &[u8]) -> Result<(), String> {
		self.txs.lock().push(rlp.to_vec());
		Ok(())
	}

	fn import_signed_private_transaction(&self, rlp: &[u8]) -> Result<(), String> {
		self.signed_txs.lock().push(rlp.to_vec());
		Ok(())
	}
}
