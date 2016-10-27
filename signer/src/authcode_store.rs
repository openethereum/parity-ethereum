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

use rand::Rng;
use rand::os::OsRng;
use std::io;
use std::io::{Read, Write};
use std::fs;
use std::path::Path;
use std::time;
use util::{H256, Hashable};

/// Providing current time in seconds
pub trait TimeProvider {
	/// Returns timestamp (in seconds since epoch)
	fn now(&self) -> u64;
}

impl<F : Fn() -> u64> TimeProvider for F {
	fn now(&self) -> u64 {
		self()
	}
}

/// Default implementation of `TimeProvider` using system time.
#[derive(Default)]
pub struct DefaultTimeProvider;

impl TimeProvider for DefaultTimeProvider {
	fn now(&self) -> u64 {
		time::UNIX_EPOCH.elapsed().expect("Valid time has to be set in your system.").as_secs()
	}
}

/// No of seconds the hash is valid
const TIME_THRESHOLD: u64 = 7;
const TOKEN_LENGTH: usize = 16;
const INITIAL_TOKEN: &'static str = "initial";

/// Manages authorization codes for `SignerUIs`
pub struct AuthCodes<T: TimeProvider = DefaultTimeProvider> {
	codes: Vec<String>,
	now: T,
}

impl AuthCodes<DefaultTimeProvider> {

	/// Reads `AuthCodes` from file and creates new instance using `DefaultTimeProvider`.
	pub fn from_file(file: &Path) -> io::Result<AuthCodes> {
		let content = {
			if let Ok(mut file) = fs::File::open(file) {
				let mut s = String::new();
				let _ = try!(file.read_to_string(&mut s));
				s
			} else {
				"".into()
			}
		};
		let codes = content.lines()
			.filter(|f| f.len() >= TOKEN_LENGTH)
			.map(String::from)
			.collect();
		Ok(AuthCodes {
			codes: codes,
			now: DefaultTimeProvider::default(),
		})
	}

}

impl<T: TimeProvider> AuthCodes<T> {

	/// Writes all `AuthCodes` to a disk.
	pub fn to_file(&self, file: &Path) -> io::Result<()> {
		let mut file = try!(fs::File::create(file));
		let content = self.codes.join("\n");
		file.write_all(content.as_bytes())
	}

	/// Creates a new `AuthCodes` store with given `TimeProvider`.
	pub fn new(codes: Vec<String>, now: T) -> Self {
		AuthCodes {
			codes: codes,
			now: now,
		}
	}

	/// Checks if given hash is correct identifier of `SignerUI`
	#[cfg_attr(feature="dev", allow(wrong_self_convention))]
	pub fn is_valid(&mut self, hash: &H256, time: u64) -> bool {
		let now = self.now.now();
		// check time
		if time >= now + TIME_THRESHOLD || time <= now - TIME_THRESHOLD {
			warn!(target: "signer", "Received old authentication request. ({} vs {})", now, time);
			return false;
		}

		let as_token = |code| format!("{}:{}", code, time).sha3();

		// Check if it's the initial token.
		if self.is_empty() {
			let initial = &as_token(INITIAL_TOKEN) == hash;
			// Initial token can be used only once.
			if initial {
				let _ = self.generate_new();
			}
			return initial;
		}

		// look for code
		self.codes.iter()
			.any(|code| &as_token(code) == hash)
	}

	/// Generates and returns a new code that can be used by `SignerUIs`
	pub fn generate_new(&mut self) -> io::Result<String> {
		let mut rng = try!(OsRng::new());
		let code = rng.gen_ascii_chars().take(TOKEN_LENGTH).collect::<String>();
		let readable_code = code.as_bytes()
			.chunks(4)
			.filter_map(|f| String::from_utf8(f.to_vec()).ok())
			.collect::<Vec<String>>()
			.join("-");
		trace!(target: "signer", "New authentication token generated.");
		self.codes.push(code);
		Ok(readable_code)
	}

	/// Returns true if there are no tokens in this store
	pub fn is_empty(&self) -> bool {
		self.codes.is_empty()
	}
}


#[cfg(test)]
mod tests {

	use util::{H256, Hashable};
	use super::*;

	fn generate_hash(val: &str, time: u64) -> H256 {
		format!("{}:{}", val, time).sha3()
	}

	#[test]
	fn should_return_true_if_code_is_initial_and_store_is_empty() {
		// given
		let code = "initial";
		let time = 99;
		let mut codes = AuthCodes::new(vec![], || 100);

		// when
		let res1 = codes.is_valid(&generate_hash(code, time), time);
		let res2 = codes.is_valid(&generate_hash(code, time), time);

		// then
		assert_eq!(res1, true);
		assert_eq!(res2, false);
	}

	#[test]
	fn should_return_true_if_hash_is_valid() {
		// given
		let code = "23521352asdfasdfadf";
		let time = 99;
		let mut codes = AuthCodes::new(vec![code.into()], || 100);

		// when
		let res = codes.is_valid(&generate_hash(code, time), time);

		// then
		assert_eq!(res, true);
	}

	#[test]
	fn should_return_false_if_code_is_unknown() {
		// given
		let code = "23521352asdfasdfadf";
		let time = 99;
		let mut codes = AuthCodes::new(vec!["1".into()], || 100);

		// when
		let res = codes.is_valid(&generate_hash(code, time), time);

		// then
		assert_eq!(res, false);
	}

	#[test]
	fn should_return_false_if_hash_is_valid_but_time_is_invalid() {
		// given
		let code = "23521352asdfasdfadf";
		let time = 107;
		let time2 = 93;
		let mut codes = AuthCodes::new(vec![code.into()], || 100);

		// when
		let res1 = codes.is_valid(&generate_hash(code, time), time);
		let res2 = codes.is_valid(&generate_hash(code, time2), time2);

		// then
		assert_eq!(res1, false);
		assert_eq!(res2, false);
	}

}


