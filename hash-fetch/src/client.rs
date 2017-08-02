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

//! Hash-addressed content resolver & fetcher.

use std::{io, fs};
use std::io::Write;
use std::sync::Arc;
use std::path::PathBuf;

use fetch::{Fetch, Response, Error as FetchError, Client as FetchClient};
use futures::Future;
use parity_reactor::Remote;
use urlhint::{ContractClient, URLHintContract, URLHint, URLHintResult};
use util::{H256, sha3};

/// API for fetching by hash.
pub trait HashFetch: Send + Sync + 'static {
	/// Fetch hash-addressed content.
	/// Parameters:
	/// 1. `hash` - content hash
	/// 2. `on_done` - callback function invoked when the content is ready (or there was error during fetch)
	///
	/// This function may fail immediately when fetch cannot be initialized or content cannot be resolved.
	fn fetch(&self, hash: H256, on_done: Box<Fn(Result<PathBuf, Error>) + Send>);
}

/// Hash-fetching error.
#[derive(Debug)]
pub enum Error {
	/// Hash could not be resolved to a valid content address.
	NoResolution,
	/// Downloaded content hash does not match.
	HashMismatch {
		/// Expected hash
		expected: H256,
		/// Computed hash
		got: H256,
	},
	/// Server didn't respond with OK status.
	InvalidStatus,
	/// IO Error while validating hash.
	IO(io::Error),
	/// Error during fetch.
	Fetch(FetchError),
}

#[cfg(test)]
impl PartialEq for Error {
	fn eq(&self, other: &Self) -> bool {
		use Error::*;
		match (self, other)  {
			(&HashMismatch { expected, got }, &HashMismatch { expected: e, got: g }) => {
				expected == e && got == g
			},
			(&NoResolution, &NoResolution) => true,
			(&InvalidStatus, &InvalidStatus) => true,
			(&IO(_), &IO(_)) => true,
			(&Fetch(_), &Fetch(_)) => true,
			_ => false,
		}
	}
}

impl From<FetchError> for Error {
	fn from(error: FetchError) -> Self {
		Error::Fetch(error)
	}
}

impl From<io::Error> for Error {
	fn from(error: io::Error) -> Self {
		Error::IO(error)
	}
}

fn validate_hash(path: PathBuf, hash: H256, result: Result<Response, FetchError>) -> Result<PathBuf, Error> {
	let response = result?;
	if !response.is_success() {
		return Err(Error::InvalidStatus);
	}

	// Read the response
	let mut reader = io::BufReader::new(response);
	let mut writer = io::BufWriter::new(fs::File::create(&path)?);
	io::copy(&mut reader, &mut writer)?;
	writer.flush()?;

	// And validate the hash
	let mut file_reader = io::BufReader::new(fs::File::open(&path)?);
	let content_hash = sha3(&mut file_reader)?;
	if content_hash != hash {
		Err(Error::HashMismatch{ got: content_hash, expected: hash })
	} else {
		Ok(path)
	}
}

/// Default Hash-fetching client using on-chain contract to resolve hashes to URLs.
pub struct Client<F: Fetch + 'static = FetchClient> {
	contract: URLHintContract,
	fetch: F,
	remote: Remote,
	random_path: Arc<Fn() -> PathBuf + Sync + Send>,
}

impl Client {
	/// Creates new instance of the `Client` given on-chain contract client and task runner.
	pub fn new(contract: Arc<ContractClient>, remote: Remote) -> Self {
		Client::with_fetch(contract, FetchClient::new().unwrap(), remote)
	}
}

impl<F: Fetch + 'static> Client<F> {
	/// Creates new instance of the `Client` given on-chain contract client, fetch service and task runner.
	pub fn with_fetch(contract: Arc<ContractClient>, fetch: F, remote: Remote) -> Self {
		Client {
			contract: URLHintContract::new(contract),
			fetch: fetch,
			remote: remote,
			random_path: Arc::new(random_temp_path),
		}
	}
}

impl<F: Fetch + 'static> HashFetch for Client<F> {
	fn fetch(&self, hash: H256, on_done: Box<Fn(Result<PathBuf, Error>) + Send>) {
		debug!(target: "fetch", "Fetching: {:?}", hash);

		let random_path = self.random_path.clone();
		let remote_fetch = self.fetch.clone();
		let future = self.contract.resolve(hash.to_vec())
			.map_err(|e| { warn!("Error resolving URL: {}", e); Error::NoResolution })
			.and_then(|maybe_url| maybe_url.ok_or(Error::NoResolution))
			.map(|content| match content {
					URLHintResult::Dapp(dapp) => {
						dapp.url()
					},
					URLHintResult::Content(content) => {
						content.url
					},
			})
			.and_then(move |url| {
				debug!(target: "fetch", "Resolved {:?} to {:?}. Fetching...", hash, url);
				let future = remote_fetch.fetch(&url).then(move |result| {
					debug!(target: "fetch", "Content fetched, validating hash ({:?})", hash);
					let path = random_path();
					let res = validate_hash(path.clone(), hash, result);
					if let Err(ref err) = res {
						trace!(target: "fetch", "Error: {:?}", err);
						// Remove temporary file in case of error
						let _ = fs::remove_file(&path);
					}
					res
				});
				remote_fetch.process(future)
			})
			.then(move |res| { on_done(res); Ok(()) as Result<(), ()> });

		self.remote.spawn(future);
	}
}

fn random_temp_path() -> PathBuf {
	use ::rand::Rng;
	use ::std::env;

	let mut rng = ::rand::OsRng::new().expect("Reliable random source is required to work.");
	let file: String = rng.gen_ascii_chars().take(12).collect();

	let mut path = env::temp_dir();
	path.push(file);
	path
}

#[cfg(test)]
mod tests {
	use rustc_hex::FromHex;
	use std::sync::{Arc, mpsc};
	use util::Mutex;
	use futures::future;
	use fetch::{self, Fetch};
	use parity_reactor::Remote;
	use urlhint::tests::{FakeRegistrar, URLHINT};
	use super::{Error, Client, HashFetch, random_temp_path};


	#[derive(Clone)]
	struct FakeFetch {
		return_success: bool
	}

	impl Fetch for FakeFetch {
		type Result = future::Ok<fetch::Response, fetch::Error>;

		fn new() -> Result<Self, fetch::Error> where Self: Sized {
			Ok(FakeFetch { return_success: true })
		}

		fn fetch_with_abort(&self, url: &str, _abort: fetch::Abort) -> Self::Result {
			assert_eq!(url, "https://parity.io/assets/images/ethcore-black-horizontal.png");
			future::ok(if self.return_success {
				let cursor = ::std::io::Cursor::new(b"result");
				fetch::Response::from_reader(cursor)
			} else {
				fetch::Response::not_found()
			})
		}
	}

	fn registrar() -> FakeRegistrar {
		let mut registrar = FakeRegistrar::new();
		registrar.responses = Mutex::new(vec![
			Ok(format!("000000000000000000000000{}", URLHINT).from_hex().unwrap()),
			Ok("00000000000000000000000000000000000000000000000000000000000000600000000000000000000000000000000000000000000000000000000000000000000000000000000000000000deadcafebeefbeefcafedeaddeedfeedffffffff000000000000000000000000000000000000000000000000000000000000003c68747470733a2f2f7061726974792e696f2f6173736574732f696d616765732f657468636f72652d626c61636b2d686f72697a6f6e74616c2e706e6700000000".from_hex().unwrap()),
		]);
		registrar
	}

	#[test]
	fn should_return_error_if_hash_not_found() {
		// given
		let contract = Arc::new(FakeRegistrar::new());
		let fetch = FakeFetch { return_success: false };
		let client = Client::with_fetch(contract.clone(), fetch, Remote::new_sync());

		// when
		let (tx, rx) = mpsc::channel();
		client.fetch(2.into(), Box::new(move |result| {
			tx.send(result).unwrap();
		}));

		// then
		let result = rx.recv().unwrap();
		assert_eq!(result.unwrap_err(), Error::NoResolution);
	}

	#[test]
	fn should_return_error_if_response_is_not_successful() {
		// given
		let registrar = Arc::new(registrar());
		let fetch = FakeFetch { return_success: false };
		let client = Client::with_fetch(registrar.clone(), fetch, Remote::new_sync());

		// when
		let (tx, rx) = mpsc::channel();
		client.fetch(2.into(), Box::new(move |result| {
			tx.send(result).unwrap();
		}));

		// then
		let result = rx.recv().unwrap();
		assert_eq!(result.unwrap_err(), Error::InvalidStatus);
	}

	#[test]
	fn should_return_hash_mismatch() {
		// given
		let registrar = Arc::new(registrar());
		let fetch = FakeFetch { return_success: true };
		let mut client = Client::with_fetch(registrar.clone(), fetch, Remote::new_sync());
		let path = random_temp_path();
		let path2 = path.clone();
		client.random_path = Arc::new(move || path2.clone());

		// when
		let (tx, rx) = mpsc::channel();
		client.fetch(2.into(), Box::new(move |result| {
			tx.send(result).unwrap();
		}));

		// then
		let result = rx.recv().unwrap();
		let hash = "0x06b0a4f426f6713234b2d4b2468640bc4e0bb72657a920ad24c5087153c593c8".into();
		assert_eq!(result.unwrap_err(), Error::HashMismatch { expected: 2.into(), got: hash });
		assert!(!path.exists(), "Temporary file should be removed.");
	}

	#[test]
	fn should_return_path_if_hash_matches() {
		// given
		let registrar = Arc::new(registrar());
		let fetch = FakeFetch { return_success: true };
		let client = Client::with_fetch(registrar.clone(), fetch, Remote::new_sync());

		// when
		let (tx, rx) = mpsc::channel();
		client.fetch("0x06b0a4f426f6713234b2d4b2468640bc4e0bb72657a920ad24c5087153c593c8".into(), Box::new(move |result| {
			tx.send(result).unwrap();
		}));

		// then
		let result = rx.recv().unwrap();
		assert!(result.is_ok(), "Should return path, got: {:?}", result);
	}
}
