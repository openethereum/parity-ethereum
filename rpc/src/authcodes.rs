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

use std::io::{self, Read, Write};
use std::path::Path;
use std::{fs, time, mem};

use itertools::Itertools;
use rand::Rng;
use rand::os::OsRng;
use hash::keccak;
use util::H256;

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
/// minimal length of hash
const TOKEN_LENGTH: usize = 16;
/// special "initial" token used for authorization when there are no tokens yet.
const INITIAL_TOKEN: &'static str = "initial";
/// Separator between fields in serialized tokens file.
const SEPARATOR: &'static str = ";";
/// Number of seconds to keep unused tokens.
const UNUSED_TOKEN_TIMEOUT: u64 = 3600 * 24; // a day

struct Code {
	code: String,
	/// Duration since unix_epoch
	created_at: time::Duration,
	/// Duration since unix_epoch
	last_used_at: Option<time::Duration>,
}

fn decode_time(val: &str) -> Option<time::Duration> {
	let time = val.parse::<u64>().ok();
	time.map(time::Duration::from_secs)
}

fn encode_time(time: time::Duration) -> String {
	format!("{}", time.as_secs())
}

/// Manages authorization codes for `SignerUIs`
pub struct AuthCodes<T: TimeProvider = DefaultTimeProvider> {
	codes: Vec<Code>,
	now: T,
}

impl AuthCodes<DefaultTimeProvider> {

	/// Reads `AuthCodes` from file and creates new instance using `DefaultTimeProvider`.
	#[cfg_attr(feature="dev", allow(single_char_pattern))]
	pub fn from_file(file: &Path) -> io::Result<AuthCodes> {
		let content = {
			if let Ok(mut file) = fs::File::open(file) {
				let mut s = String::new();
				let _ = file.read_to_string(&mut s)?;
				s
			} else {
				"".into()
			}
		};
		let time_provider = DefaultTimeProvider::default();

		let codes = content.lines()
			.filter_map(|line| {
				let mut parts = line.split(SEPARATOR);
				let token = parts.next();
				let created = parts.next();
				let used = parts.next();

				match token {
					None => None,
					Some(token) if token.len() < TOKEN_LENGTH => None,
					Some(token) => {
						Some(Code {
							code: token.into(),
							last_used_at: used.and_then(decode_time),
							created_at: created.and_then(decode_time)
											.unwrap_or_else(|| time::Duration::from_secs(time_provider.now())),
						})
					}
				}
			})
			.collect();
		Ok(AuthCodes {
			codes: codes,
			now: time_provider,
		})
	}

}

impl<T: TimeProvider> AuthCodes<T> {

	/// Writes all `AuthCodes` to a disk.
	pub fn to_file(&self, file: &Path) -> io::Result<()> {
		let mut file = fs::File::create(file)?;
		let content = self.codes.iter().map(|code| {
			let mut data = vec![code.code.clone(), encode_time(code.created_at.clone())];
			if let Some(used_at) = code.last_used_at {
				data.push(encode_time(used_at));
			}
			data.join(SEPARATOR)
		}).join("\n");
		file.write_all(content.as_bytes())
	}

	/// Creates a new `AuthCodes` store with given `TimeProvider`.
	pub fn new(codes: Vec<String>, now: T) -> Self {
		AuthCodes {
			codes: codes.into_iter().map(|code| Code {
				code: code,
				created_at: time::Duration::from_secs(now.now()),
				last_used_at: None,
			}).collect(),
			now: now,
		}
	}

	/// Checks if given hash is correct authcode of `SignerUI`
	/// Updates this hash last used field in case it's valid.
	#[cfg_attr(feature="dev", allow(wrong_self_convention))]
	pub fn is_valid(&mut self, hash: &H256, time: u64) -> bool {
		let now = self.now.now();
		// check time
		if time >= now + TIME_THRESHOLD || time <= now - TIME_THRESHOLD {
			warn!(target: "signer", "Received old authentication request. ({} vs {})", now, time);
			return false;
		}

		let as_token = |code| keccak(format!("{}:{}", code, time));

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
		for mut code in &mut self.codes {
			if &as_token(&code.code) == hash {
				code.last_used_at = Some(time::Duration::from_secs(now));
				return true;
			}
		}

		false
	}

	/// Generates and returns a new code that can be used by `SignerUIs`
	pub fn generate_new(&mut self) -> io::Result<String> {
		let mut rng = OsRng::new()?;
		let code = rng.gen_ascii_chars().take(TOKEN_LENGTH).collect::<String>();
		let readable_code = code.as_bytes()
			.chunks(4)
			.filter_map(|f| String::from_utf8(f.to_vec()).ok())
			.collect::<Vec<String>>()
			.join("-");
		trace!(target: "signer", "New authentication token generated.");
		self.codes.push(Code {
			code: code,
			created_at: time::Duration::from_secs(self.now.now()),
			last_used_at: None,
		});
		Ok(readable_code)
	}

	/// Returns true if there are no tokens in this store
	pub fn is_empty(&self) -> bool {
		self.codes.is_empty()
	}

	/// Removes old tokens that have not been used since creation.
	pub fn clear_garbage(&mut self) {
		let now = self.now.now();
		let threshold = time::Duration::from_secs(now.saturating_sub(UNUSED_TOKEN_TIMEOUT));

		let codes = mem::replace(&mut self.codes, Vec::new());
		for code in codes {
			// Skip codes that are old and were never used.
			if code.last_used_at.is_none() && code.created_at <= threshold {
				continue;
			}
			self.codes.push(code);
		}
	}
}

#[cfg(test)]
mod tests {

	use devtools;
	use std::io::{Read, Write};
	use std::{time, fs};
	use std::cell::Cell;
	use hash::keccak;

	use util::H256;
	use super::*;

	fn generate_hash(val: &str, time: u64) -> H256 {
		keccak(format!("{}:{}", val, time))
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

	#[test]
	fn should_read_old_format_from_file() {
		// given
		let path = devtools::RandomTempPath::new();
		let code = "23521352asdfasdfadf";
		{
			let mut file = fs::File::create(&path).unwrap();
			file.write_all(b"a\n23521352asdfasdfadf\nb\n").unwrap();
		}

		// when
		let mut authcodes = AuthCodes::from_file(&path).unwrap();
		let time = time::UNIX_EPOCH.elapsed().unwrap().as_secs();

		// then
		assert!(authcodes.is_valid(&generate_hash(code, time), time), "Code should be read from file");
	}

	#[test]
	fn should_remove_old_unused_tokens() {
		// given
		let path = devtools::RandomTempPath::new();
		let code1 = "11111111asdfasdf111";
		let code2 = "22222222asdfasdf222";
		let code3 = "33333333asdfasdf333";

		let time = Cell::new(100);
		let mut codes = AuthCodes::new(vec![code1.into(), code2.into(), code3.into()], || time.get());
		// `code2` should not be removed (we never remove tokens that were used)
		codes.is_valid(&generate_hash(code2, time.get()), time.get());

		// when
		time.set(100 + 10_000_000);
		// mark `code1` as used now
		codes.is_valid(&generate_hash(code1, time.get()), time.get());

		let new_code = codes.generate_new().unwrap().replace('-', "");
		codes.clear_garbage();
		codes.to_file(&path).unwrap();

		// then
		let mut content = String::new();
		let mut file = fs::File::open(&path).unwrap();
		file.read_to_string(&mut content).unwrap();

		assert_eq!(content, format!("{};100;10000100\n{};100;100\n{};10000100", code1, code2, new_code));
	}

}
