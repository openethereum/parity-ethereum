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
use rustc_serialize::hex::ToHex;

use ethabi::{Interface, Contract, Token};
use util::{Address, Bytes};

const COMMIT_LEN: usize = 20;

#[derive(Debug, PartialEq)]
pub struct GithubApp {
	pub account: String,
	pub repo: String,
	pub commit: [u8;COMMIT_LEN],
	pub owner: Address,
}

impl GithubApp {
	pub fn url(&self) -> String {
		// format!("https://github.com/{}/{}/archive/{}.zip", self.account, self.repo, self.commit.to_hex())
		format!("http://github.todr.me/{}/{}/zip/{}", self.account, self.repo, self.commit.to_hex())
	}

	fn commit(bytes: &[u8]) -> Option<[u8;COMMIT_LEN]> {
		if bytes.len() < COMMIT_LEN {
			return None;
		}

		let mut commit = [0; COMMIT_LEN];
		for i in 0..COMMIT_LEN {
			commit[i] = bytes[i];
		}

		Some(commit)
	}
}

/// RAW Registrar Contract interface.
/// Should execute transaction using current blockchain state.
pub trait RegistrarClient: Send + Sync {
	/// Call Registrar Contract
	fn call(&self, data: Bytes) -> Result<Bytes, String>;
}

/// URLHint Contract interface
pub trait URLHint {
	/// Resolves given id to registrar entry.
	fn resolve(&self, app_id: &str) -> Option<GithubApp>;
}

pub struct URLHintContract {
	contract: Contract,
	client: Arc<Box<RegistrarClient>>,
}

impl URLHintContract {
	pub fn new(client: Arc<Box<RegistrarClient>>) -> Self {
		let iface = Interface::load(include_bytes!("./urlhint.json")).expect("urlhint.json is valid ABI");
		let contract = Contract::new(iface);

		URLHintContract {
			contract: contract,
			client: client,
		}
	}

	fn encode_call(&self, app_id: &str) -> Option<Bytes> {
		let call = self.contract
			.function("entries".into())
			.and_then(|f| f.encode_call(vec![Token::FixedBytes(app_id.bytes().collect())]));

		match call {
			Ok(res) => Some(res),
			Err(e) => {
				warn!(target: "dapps", "Error while encoding registrar call: {:?}", e);
				None
			}
		}
	}

	fn decode_output(&self, output: Bytes) -> Option<GithubApp> {
		trace!(target: "dapps", "Output: {:?}", output);
		let output = self.contract
			.function("entries".into())
			.and_then(|f| f.decode_output(output));

		if let Ok(vec) = output {
			if vec.len() != 4 {
				warn!(target: "dapps", "Invalid contract output: {:?}", vec);
				return None;
			}

			let mut it = vec.into_iter();
			let account = it.next().unwrap();
			let repo = it.next().unwrap();
			let commit = it.next().unwrap();
			let owner = it.next().unwrap();


			trace!(target: "dapps", "Resolved output: {:?}/{:?}/{:?}; owner: {:?}", account, repo, commit, owner);

			match (account, repo, commit, owner) {
				(Token::String(account), Token::String(repo), Token::FixedBytes(commit), Token::Address(owner)) => {
					let owner = owner.into();
					if owner == Address::default() {
						return None;
					}

					GithubApp::commit(&commit).map(|commit| GithubApp {
						account: account,
						repo: repo,
						commit: commit,
						owner: owner,
					})
				},
				e => {
					warn!(target: "dapps", "Invalid contract output parameters: {:?}", e);
					None
				},
			}
		} else {
			warn!(target: "dapps", "Invalid contract output: {:?}", output);
			None
		}
	}
}

impl URLHint for URLHintContract {

	fn resolve(&self, app_id: &str) -> Option<GithubApp> {
		// Prepare contract call
		self.encode_call(app_id)
			.and_then(|data| {
				let call = self.client.call(data);
				if let Err(ref e) = call {
					warn!(target: "dapps", "Error while calling registrar: {:?}", e);
				}
				call.ok()
			})
			.and_then(|output| self.decode_output(output))
	}
}

#[cfg(test)]
mod tests {
	use std::sync::Arc;
	use std::str::FromStr;
	use rustc_serialize::hex::FromHex;

	use super::*;
	use util::{Bytes, Address, Mutex, ToPretty};

	struct FakeRegistrar {
		pub calls: Arc<Mutex<Vec<Bytes>>>,
		pub response: Result<Bytes, String>,
	}

	impl FakeRegistrar {
		fn new() -> Self {
			FakeRegistrar {
				calls: Arc::new(Mutex::new(Vec::new())),
				response: Ok(Vec::new()),
			}
		}
	}

	impl RegistrarClient for FakeRegistrar {
		fn call(&self, data: Bytes) -> Result<Bytes, String> {
			self.calls.lock().push(data);
			self.response.clone()
		}
	}

	#[test]
	fn should_create_urlhint_contract() {
		// given
		let registrar = FakeRegistrar::new();
		let calls = registrar.calls.clone();
		let urlhint = URLHintContract::new(Arc::new(Box::new(registrar)));

		// when
		let res = urlhint.resolve("test");

		// then
		assert!(res.is_none());
		assert_eq!(
			calls.lock().get(0).expect("Resolve called").to_hex(),
			"267b69227465737400000000000000000000000000000000000000000000000000000000".to_owned()
		);
	}

	#[test]
	fn should_decode_urlhint_output() {
		// given
		let mut registrar = FakeRegistrar::new();
		registrar.response = Ok(
			"000000000000000000000000000000000000000000000000000000000000008000000000000000000000000000000000000000000000000000000000000000c0ec4c1fe06c808fe3739858c347109b1f5f1ed4b5000000000000000000000000000000000000000000000000deadcafebeefbeefcafedeaddeedfeedffffffff0000000000000000000000000000000000000000000000000000000000000007657468636f726500000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000864616f636c61696d000000000000000000000000000000000000000000000000".from_hex().unwrap()
		);
		let urlhint = URLHintContract::new(Arc::new(Box::new(registrar)));

		// when
		let res = urlhint.resolve("test");

		// then
		assert_eq!(res, Some(GithubApp {
			account: "ethcore".into(),
			repo: "daoclaim".into(),
			commit: GithubApp::commit(&"ec4c1fe06c808fe3739858c347109b1f5f1ed4b5".from_hex().unwrap()).unwrap(),
			owner: Address::from_str("deadcafebeefbeefcafedeaddeedfeedffffffff").unwrap(),
		}))
	}

	#[test]
	fn should_return_valid_url() {
		// given
		let app = GithubApp {
			account: "test".into(),
			repo: "xyz".into(),
			commit: [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19],
			owner: Address::default(),
		};

		// when
		let url = app.url();

		// then
		assert_eq!(url, "http://github.todr.me/test/xyz/zip/000102030405060708090a0b0c0d0e0f10111213".to_owned());
	}
}
