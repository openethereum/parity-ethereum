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

//! URLHint Contract

use std::fmt;
use std::sync::Arc;
use rustc_serialize::hex::ToHex;
use mime::Mime;
use mime_guess;

use ethabi::{Interface, Contract, Token};
use util::{Address, Bytes, Hashable};

const COMMIT_LEN: usize = 20;

/// RAW Contract interface.
/// Should execute transaction using current blockchain state.
pub trait ContractClient: Send + Sync {
	/// Get registrar address
	fn registrar(&self) -> Result<Address, String>;
	/// Call Contract
	fn call(&self, address: Address, data: Bytes) -> Result<Bytes, String>;
}

/// Github-hosted dapp.
#[derive(Debug, PartialEq)]
pub struct GithubApp {
	/// Github Account
	pub account: String,
	/// Github Repository
	pub repo: String,
	/// Commit on Github
	pub commit: [u8;COMMIT_LEN],
	/// Dapp owner address
	pub owner: Address,
}

impl GithubApp {
	/// Returns URL of this Github-hosted dapp package.
	pub fn url(&self) -> String {
		// Since https fetcher doesn't support redirections we use direct link
		// format!("https://github.com/{}/{}/archive/{}.zip", self.account, self.repo, self.commit.to_hex())
		format!("https://codeload.github.com/{}/{}/zip/{}", self.account, self.repo, self.commit.to_hex())
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

/// Hash-Addressed Content
#[derive(Debug, PartialEq)]
pub struct Content {
	/// URL of the content
	pub url: String,
	/// MIME type of the content
	pub mime: Mime,
	/// Content owner address
	pub owner: Address,
}

/// Result of resolving id to URL
#[derive(Debug, PartialEq)]
pub enum URLHintResult {
	/// Dapp
	Dapp(GithubApp),
	/// Content
	Content(Content),
}

/// URLHint Contract interface
pub trait URLHint {
	/// Resolves given id to registrar entry.
	fn resolve(&self, id: Bytes) -> Option<URLHintResult>;
}

/// `URLHintContract` API
pub struct URLHintContract {
	urlhint: Contract,
	registrar: Contract,
	client: Arc<ContractClient>,
}

impl URLHintContract {
	/// Creates new `URLHintContract`
	pub fn new(client: Arc<ContractClient>) -> Self {
		let urlhint = Interface::load(include_bytes!("../res/urlhint.json")).expect("urlhint.json is valid ABI");
		let registrar = Interface::load(include_bytes!("../res/registrar.json")).expect("registrar.json is valid ABI");

		URLHintContract {
			urlhint: Contract::new(urlhint),
			registrar: Contract::new(registrar),
			client: client,
		}
	}

	fn urlhint_address(&self) -> Option<Address> {
		let res = || {
			let get_address = self.registrar.function("getAddress".into()).map_err(as_string)?;
			let params = get_address.encode_call(
					vec![Token::FixedBytes((*"githubhint".sha3()).to_vec()), Token::String("A".into())]
			).map_err(as_string)?;
			let output = self.client.call(self.client.registrar()?, params)?;
			let result = get_address.decode_output(output).map_err(as_string)?;

			match result.get(0) {
				Some(&Token::Address(address)) if address != *Address::default() => Ok(address.into()),
				Some(&Token::Address(_)) => Err(format!("Contract not found.")),
				e => Err(format!("Invalid result: {:?}", e)),
			}
		};

		match res() {
			Ok(res) => Some(res),
			Err(e) => {
				warn!(target: "dapps", "Error while calling registrar: {:?}", e);
				None
			}
		}
	}

	fn encode_urlhint_call(&self, id: Bytes) -> Option<Bytes> {
		let call = self.urlhint
			.function("entries".into())
			.and_then(|f| f.encode_call(vec![Token::FixedBytes(id)]));

		match call {
			Ok(res) => {
				Some(res)
			},
			Err(e) => {
				warn!(target: "dapps", "Error while encoding urlhint call: {:?}", e);
				None
			}
		}
	}

	fn decode_urlhint_output(&self, output: Bytes) -> Option<URLHintResult> {
		trace!(target: "dapps", "Output: {:?}", output.to_hex());
		let output = self.urlhint
			.function("entries".into())
			.and_then(|f| f.decode_output(output));

		if let Ok(vec) = output {
			if vec.len() != 3 {
				warn!(target: "dapps", "Invalid contract output: {:?}", vec);
				return None;
			}

			let mut it = vec.into_iter();
			let account_slash_repo = it.next().expect("element 0 of 3-len vector known to exist; qed");
			let commit = it.next().expect("element 1 of 3-len vector known to exist; qed");
			let owner = it.next().expect("element 2 of 3-len vector known to exist qed");

			match (account_slash_repo, commit, owner) {
				(Token::String(account_slash_repo), Token::FixedBytes(commit), Token::Address(owner)) => {
					let owner = owner.into();
					if owner == Address::default() {
						return None;
					}

					let commit = GithubApp::commit(&commit);
					if commit == Some(Default::default()) {
						let mime = guess_mime_type(&account_slash_repo).unwrap_or(mime!(Application/_));
						return Some(URLHintResult::Content(Content {
							url: account_slash_repo,
							mime: mime,
							owner: owner,
						}));
					}

					let (account, repo) = {
						let mut it = account_slash_repo.split('/');
						match (it.next(), it.next()) {
							(Some(account), Some(repo)) => (account.into(), repo.into()),
							_ => return None,
						}
					};

					commit.map(|commit| URLHintResult::Dapp(GithubApp {
						account: account,
						repo: repo,
						commit: commit,
						owner: owner,
					}))
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
	fn resolve(&self, id: Bytes) -> Option<URLHintResult> {
		self.urlhint_address().and_then(|address| {
			// Prepare contract call
			self.encode_urlhint_call(id)
				.and_then(|data| {
					let call = self.client.call(address, data);
					if let Err(ref e) = call {
						warn!(target: "dapps", "Error while calling urlhint: {:?}", e);
					}
					call.ok()
				})
				.and_then(|output| self.decode_urlhint_output(output))
		})
	}
}

fn guess_mime_type(url: &str) -> Option<Mime> {
	const CONTENT_TYPE: &'static str = "content-type=";

	let mut it = url.split('#');
	// skip url
	let url = it.next();
	// get meta headers
	let metas = it.next();
	if let Some(metas) = metas {
		for meta in metas.split('&') {
			let meta = meta.to_lowercase();
			if meta.starts_with(CONTENT_TYPE) {
				return meta[CONTENT_TYPE.len()..].parse().ok();
			}
		}
	}
	url.and_then(|url| {
		url.split('.').last()
	}).and_then(|extension| {
		mime_guess::get_mime_type_opt(extension)
	})
}

fn as_string<T: fmt::Debug>(e: T) -> String {
	format!("{:?}", e)
}

#[cfg(test)]
pub mod tests {
	use std::sync::Arc;
	use std::str::FromStr;
	use rustc_serialize::hex::FromHex;

	use super::*;
	use super::guess_mime_type;
	use util::{Bytes, Address, Mutex, ToPretty};

	pub struct FakeRegistrar {
		pub calls: Arc<Mutex<Vec<(String, String)>>>,
		pub responses: Mutex<Vec<Result<Bytes, String>>>,
	}

	pub const REGISTRAR: &'static str = "8e4e9b13d4b45cb0befc93c3061b1408f67316b2";
	pub const URLHINT: &'static str = "deadbeefcafe0000000000000000000000000000";

	impl FakeRegistrar {
		pub fn new() -> Self {
			FakeRegistrar {
				calls: Arc::new(Mutex::new(Vec::new())),
				responses: Mutex::new(
					vec![
						Ok(format!("000000000000000000000000{}", URLHINT).from_hex().unwrap()),
						Ok(Vec::new())
					]
				),
			}
		}
	}

	impl ContractClient for FakeRegistrar {

		fn registrar(&self) -> Result<Address, String> {
			Ok(REGISTRAR.parse().unwrap())
		}

		fn call(&self, address: Address, data: Bytes) -> Result<Bytes, String> {
			self.calls.lock().push((address.to_hex(), data.to_hex()));
			self.responses.lock().remove(0)
		}
	}

	#[test]
	fn should_call_registrar_and_urlhint_contracts() {
		// given
		let registrar = FakeRegistrar::new();
		let calls = registrar.calls.clone();
		let urlhint = URLHintContract::new(Arc::new(registrar));

		// when
		let res = urlhint.resolve("test".bytes().collect());
		let calls = calls.lock();
		let call0 = calls.get(0).expect("Registrar resolve called");
		let call1 = calls.get(1).expect("URLHint Resolve called");

		// then
		assert!(res.is_none());
		assert_eq!(call0.0, REGISTRAR);
		assert_eq!(call0.1,
			"6795dbcd058740ee9a5a3fb9f1cfa10752baec87e09cc45cd7027fd54708271aca300c75000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000000014100000000000000000000000000000000000000000000000000000000000000".to_owned()
		);
		assert_eq!(call1.0, URLHINT);
		assert_eq!(call1.1,
			"267b69227465737400000000000000000000000000000000000000000000000000000000".to_owned()
		);
	}

	#[test]
	fn should_decode_urlhint_output() {
		// given
		let mut registrar = FakeRegistrar::new();
		registrar.responses = Mutex::new(vec![
			Ok(format!("000000000000000000000000{}", URLHINT).from_hex().unwrap()),
			Ok("0000000000000000000000000000000000000000000000000000000000000060ec4c1fe06c808fe3739858c347109b1f5f1ed4b5000000000000000000000000000000000000000000000000deadcafebeefbeefcafedeaddeedfeedffffffff0000000000000000000000000000000000000000000000000000000000000011657468636f72652f64616f2e636c61696d000000000000000000000000000000".from_hex().unwrap()),
		]);
		let urlhint = URLHintContract::new(Arc::new(registrar));

		// when
		let res = urlhint.resolve("test".bytes().collect());

		// then
		assert_eq!(res, Some(URLHintResult::Dapp(GithubApp {
			account: "ethcore".into(),
			repo: "dao.claim".into(),
			commit: GithubApp::commit(&"ec4c1fe06c808fe3739858c347109b1f5f1ed4b5".from_hex().unwrap()).unwrap(),
			owner: Address::from_str("deadcafebeefbeefcafedeaddeedfeedffffffff").unwrap(),
		})))
	}

	#[test]
	fn should_decode_urlhint_content_output() {
		// given
		let mut registrar = FakeRegistrar::new();
		registrar.responses = Mutex::new(vec![
			Ok(format!("000000000000000000000000{}", URLHINT).from_hex().unwrap()),
			Ok("00000000000000000000000000000000000000000000000000000000000000600000000000000000000000000000000000000000000000000000000000000000000000000000000000000000deadcafebeefbeefcafedeaddeedfeedffffffff000000000000000000000000000000000000000000000000000000000000003d68747470733a2f2f657468636f72652e696f2f6173736574732f696d616765732f657468636f72652d626c61636b2d686f72697a6f6e74616c2e706e67000000".from_hex().unwrap()),
		]);
		let urlhint = URLHintContract::new(Arc::new(registrar));

		// when
		let res = urlhint.resolve("test".bytes().collect());

		// then
		assert_eq!(res, Some(URLHintResult::Content(Content {
			url: "https://ethcore.io/assets/images/ethcore-black-horizontal.png".into(),
			mime: mime!(Image/Png),
			owner: Address::from_str("deadcafebeefbeefcafedeaddeedfeedffffffff").unwrap(),
		})))
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
		assert_eq!(url, "https://codeload.github.com/test/xyz/zip/000102030405060708090a0b0c0d0e0f10111213".to_owned());
	}

	#[test]
	fn should_guess_mime_type_from_url() {
		let url1 = "https://ethcore.io/parity";
		let url2 = "https://ethcore.io/parity#content-type=image/png";
		let url3 = "https://ethcore.io/parity#something&content-type=image/png";
		let url4 = "https://ethcore.io/parity.png#content-type=image/jpeg";
		let url5 = "https://ethcore.io/parity.png";


		assert_eq!(guess_mime_type(url1), None);
		assert_eq!(guess_mime_type(url2), Some(mime!(Image/Png)));
		assert_eq!(guess_mime_type(url3), Some(mime!(Image/Png)));
		assert_eq!(guess_mime_type(url4), Some(mime!(Image/Jpeg)));
		assert_eq!(guess_mime_type(url5), Some(mime!(Image/Png)));
	}
}
