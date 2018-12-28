// Copyright 2015-2018 Parity Technologies (UK) Ltd.
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

use std::{cmp, thread};
use std::sync::Arc;
use std::collections::VecDeque;
use parking_lot::Mutex;

use ethstore::{ethkey::Password, PresaleWallet, Error};
use num_cpus;

pub fn run(passwords: VecDeque<Password>, wallet_path: &str) -> Result<(), Error> {
	let passwords = Arc::new(Mutex::new(passwords));

	let mut handles = Vec::new();

	for _ in 0..num_cpus::get() {
		let passwords = passwords.clone();
		let wallet = PresaleWallet::open(&wallet_path)?;
		handles.push(thread::spawn(move || {
			look_for_password(passwords, wallet);
		}));
	}

	for handle in handles {
		handle.join().map_err(|err| Error::Custom(format!("Error finishing thread: {:?}", err)))?;
	}

	Ok(())
}

fn look_for_password(passwords: Arc<Mutex<VecDeque<Password>>>, wallet: PresaleWallet) {
	let mut counter = 0;
	while !passwords.lock().is_empty() {
		let package = {
			let mut passwords = passwords.lock();
			let len = passwords.len();
			passwords.split_off(cmp::min(len, 32))
		};
		for pass in package {
			counter += 1;
			match wallet.decrypt(&pass) {
				Ok(_) => {
					println!("Found password: {}", pass.as_str());
					passwords.lock().clear();
					return;
				},
				_ if counter % 100 == 0 => print!("."),
				_ => {},
			}
		}
	}
}
