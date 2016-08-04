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

use std::collections::HashSet;
use ethkey::Address;
use dir::KeyDirectory;
use Error;

pub fn import_accounts(src: &KeyDirectory, dst: &KeyDirectory) -> Result<Vec<Address>, Error> {
	let accounts = try!(src.load());
	let existing_accounts = try!(dst.load()).into_iter().map(|a| a.address).collect::<HashSet<_>>();

	accounts.into_iter()
		.filter(|a| !existing_accounts.contains(&a.address))
		.map(|a| {
			let address = a.address.clone();
			try!(dst.insert(a));
			Ok(address)
		}).collect()
}
