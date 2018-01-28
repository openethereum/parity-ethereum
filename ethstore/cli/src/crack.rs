use std::{cmp, thread};
use std::sync::Arc;
use std::collections::VecDeque;
use parking_lot::Mutex;

use ethstore::{PresaleWallet, Error};
use num_cpus;

pub fn run(passwords: VecDeque<String>, wallet_path: &str) -> Result<(), Error> {
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

fn look_for_password(passwords: Arc<Mutex<VecDeque<String>>>, wallet: PresaleWallet) {
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
					println!("Found password: {}", &pass);
					passwords.lock().clear();
					return;
				},
				_ if counter % 100 == 0 => print!("."),
				_ => {},
			}
		}
	}
}
