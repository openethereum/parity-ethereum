// Copyright 2015-2019 Parity Technologies (UK) Ltd.
// This file is part of Parity Ethereum.

// Parity Ethereum is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity Ethereum is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity Ethereum.  If not, see <http://www.gnu.org/licenses/>.

//! URLHint Contract

use std::sync::Weak;
use rustc_hex::ToHex;
use mime::{self, Mime};
use mime_guess;

use ethereum_types::{H256, Address};
use registrar::RegistrarClient;
use types::ids::BlockId;

use_contract!(urlhint, "res/urlhint.json");

const COMMIT_LEN: usize = 20;
const GITHUB_HINT: &'static str = "githubhint";
/// GithubHint entries with commit set as `0x0..01` should be treated
/// as Github Dapp, downloadable zip files, than can be extracted, containing
/// the manifest.json file along with the dapp
static GITHUB_DAPP_COMMIT: &[u8; COMMIT_LEN] = &[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1];

/// Github-hosted dapp.
#[derive(Debug, PartialEq)]
pub struct GithubApp {
	/// Github Account
	pub account: String,
	/// Github Repository
	pub repo: String,
	/// Commit on Github
	pub commit: [u8; COMMIT_LEN],
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
	/// GithubDapp
	GithubDapp(Content),
	/// Content
	Content(Content),
}

/// URLHint Contract interface
pub trait URLHint: Send + Sync {
	/// Resolves given id to registrar entry.
	fn resolve(&self, id: H256) -> Result<Option<URLHintResult>, String>;
}

/// `URLHintContract` API
pub struct URLHintContract {
	client: Weak<dyn RegistrarClient>,
}

impl URLHintContract {
	/// Creates new `URLHintContract`
	pub fn new(client: Weak<dyn RegistrarClient>) -> Self {
		URLHintContract {
			client: client,
		}
	}
}

fn get_urlhint_content(account_slash_repo: String, owner: Address) -> Content {
	let mime = guess_mime_type(&account_slash_repo).unwrap_or(mime::APPLICATION_JSON);
	Content {
		url: account_slash_repo,
		mime,
		owner,
	}
}

fn decode_urlhint_output(
	account_slash_repo: String,
	commit: [u8; 20],
	owner: Address
) -> Option<URLHintResult> {
	if owner == Address::zero() {
		return None;
	}

	let commit = GithubApp::commit(&commit);

	if commit == Some(Default::default()) {
		let content = get_urlhint_content(account_slash_repo, owner);
		return Some(URLHintResult::Content(content));
	}

	if commit == Some(*GITHUB_DAPP_COMMIT) {
		let content = get_urlhint_content(account_slash_repo, owner);
		return Some(URLHintResult::GithubDapp(content));
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
}

impl URLHint for URLHintContract {
	fn resolve(&self, id: H256) -> Result<Option<URLHintResult>, String>  {
		use urlhint::urlhint::functions::entries::{encode_input, decode_output};

		let client = self.client.clone().upgrade()
			.ok_or_else(|| "Registrar/contract client unavailable".to_owned())?;

		let returned_address = client.get_address(GITHUB_HINT, BlockId::Latest)?;

		if let Some(address) = returned_address {
			let data = encode_input(id);
			let output_bytes = client.call_contract(BlockId::Latest, address, data)?;
			let (account_slash_repo, commit, owner) = decode_output(&output_bytes)
				.map_err(|e| e.to_string())?;

			let url_hint = decode_urlhint_output(account_slash_repo, commit, owner);

			Ok(url_hint)
		} else {
			Ok(None)
		}
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

#[cfg(test)]
pub mod tests {
	use std::sync::Arc;
	use std::str::FromStr;
	use rustc_hex::FromHex;

	use super::*;
	use super::guess_mime_type;
	use parking_lot::Mutex;
	use ethereum_types::Address;
	use bytes::{Bytes, ToPretty};
	use call_contract::CallContract;

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

	impl CallContract for FakeRegistrar {
		fn call_contract(
			&self,
			_block: BlockId,
			address: Address,
			data: Bytes
		) -> Result<Bytes, String> {
			self.calls.lock().push((address.to_hex(), data.to_hex()));
			let res = self.responses.lock().remove(0);

			res
		}
	}

	impl RegistrarClient for FakeRegistrar {
		fn registrar_address(&self) -> Option<Address> {
			Some(REGISTRAR.parse().unwrap())
		}
	}

	fn h256_from_short_str(s: &str) -> H256 {
		let mut bytes = s.as_bytes().to_vec();
		bytes.resize(32usize, 0u8);
		H256::from_slice(bytes.as_ref())
	}

	#[test]
	fn should_call_registrar_and_urlhint_contracts() {
		// given
		let registrar = FakeRegistrar::new();
		let resolve_result = {
			use ethabi::{encode, Token};
			encode(&[Token::String(String::new()), Token::FixedBytes(vec![0; 20]), Token::Address([0; 20].into())])
		};
		registrar.responses.lock()[1] = Ok(resolve_result);

		let calls = registrar.calls.clone();

		let registrar = Arc::new(registrar) as Arc<dyn RegistrarClient>;
		let urlhint = URLHintContract::new(Arc::downgrade(&registrar));

		// when
		let res = urlhint.resolve(h256_from_short_str("test")).unwrap();
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

		let registrar = Arc::new(registrar) as Arc<dyn RegistrarClient>;
		let urlhint = URLHintContract::new(Arc::downgrade(&registrar));

		// when
		let res = urlhint.resolve(h256_from_short_str("test")).unwrap();

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
			Ok("00000000000000000000000000000000000000000000000000000000000000600000000000000000000000000000000000000000000000000000000000000000000000000000000000000000deadcafebeefbeefcafedeaddeedfeedffffffff000000000000000000000000000000000000000000000000000000000000003c68747470733a2f2f7061726974792e696f2f6173736574732f696d616765732f657468636f72652d626c61636b2d686f72697a6f6e74616c2e706e6700000000".from_hex().unwrap()),
		]);

		let registrar = Arc::new(registrar) as Arc<dyn RegistrarClient>;
		let urlhint = URLHintContract::new(Arc::downgrade(&registrar));

		// when
		let res = urlhint.resolve(h256_from_short_str("test")).unwrap();

		// then
		assert_eq!(res, Some(URLHintResult::Content(Content {
			url: "https://parity.io/assets/images/ethcore-black-horizontal.png".into(),
			mime: mime::IMAGE_PNG,
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
			owner: Address::zero(),
		};

		// when
		let url = app.url();

		// then
		assert_eq!(url, "https://codeload.github.com/test/xyz/zip/000102030405060708090a0b0c0d0e0f10111213".to_owned());
	}

	#[test]
	fn should_guess_mime_type_from_url() {
		let url1 = "https://parity.io/parity";
		let url2 = "https://parity.io/parity#content-type=image/png";
		let url3 = "https://parity.io/parity#something&content-type=image/png";
		let url4 = "https://parity.io/parity.png#content-type=image/jpeg";
		let url5 = "https://parity.io/parity.png";

		assert_eq!(guess_mime_type(url1), None);
		assert_eq!(guess_mime_type(url2), Some(mime::IMAGE_PNG));
		assert_eq!(guess_mime_type(url3), Some(mime::IMAGE_PNG));
		assert_eq!(guess_mime_type(url4), Some(mime::IMAGE_JPEG));
		assert_eq!(guess_mime_type(url5), Some(mime::IMAGE_PNG));
	}
}
