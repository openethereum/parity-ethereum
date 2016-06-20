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

use ethkey::{Address, Message, Signature, Secret};
use Error;

pub trait SecretStore: Send + Sync {
	fn insert_account(&self, secret: Secret, password: &str) -> Result<Address, Error>;

	fn accounts(&self) -> Vec<Address>;

	fn change_password(&self, account: &Address, old_password: &str, new_password: &str) -> Result<(), Error>;

	fn remove_account(&self, account: &Address, password: &str) -> Result<(), Error>;

	fn sign(&self, account: &Address, password: &str, message: &Message) -> Result<Signature, Error>;
}

