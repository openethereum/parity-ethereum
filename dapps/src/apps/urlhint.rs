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

use rustc_serialize::hex::ToHex;

use util::{Address, FromHex};

const COMMIT_LEN: usize = 20;

#[derive(Debug)]
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

pub trait URLHint {
	fn resolve(&self, app_id: &str) -> Option<GithubApp>;
}

pub struct URLHintContract;

impl URLHint for URLHintContract {
	fn resolve(&self, app_id: &str) -> Option<GithubApp> {
		// TODO [todr] use GithubHint contract to check the details
		// For now we are just accepting patterns: <commithash>.<repo>.<account>.parity
		let mut app_parts = app_id.split('.');

		let hash = app_parts.next()
			.and_then(|h| h.from_hex().ok())
			.and_then(|h| GithubApp::commit(&h));
		let repo = app_parts.next();
		let account = app_parts.next();

		match (hash, repo, account) {
			(Some(hash), Some(repo), Some(account)) => {
				Some(GithubApp {
					account: account.into(),
					repo: repo.into(),
					commit: hash,
					owner: Address::default(),
				})
			},
			_ => None,
		}
	}
}

#[cfg(test)]
mod tests {
	use super::GithubApp;
	use util::Address;

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
