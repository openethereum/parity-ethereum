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

//! Network events and event listeners.

use network::PeerId;

use super::{Status, Capabilities, Announcement};

use transaction::SignedTransaction;

/// Peer connected
pub struct Connect(PeerId, Status, Capabilities);

/// Peer disconnected
pub struct Disconnect(PeerId);

/// Peer announces new capabilities.
pub struct Announcement(PeerId, Announcement);

/// Transactions to be relayed.
pub struct RelayTransactions(Vec<SignedTransaction>);

/// An LES event handler.
pub trait Handler {
	fn on_connect(&self, _event: Connect);
}