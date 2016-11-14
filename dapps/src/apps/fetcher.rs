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

//! Fetchable Dapps support.
//! Manages downloaded (cached) Dapps and downloads them when necessary.
//! Uses `URLHint` to resolve addresses into Dapps bundle file location.

use zip;
use std::{fs, env, fmt};
use std::io::{self, Read, Write};
use std::path::PathBuf;
use std::sync::Arc;
use rustc_serialize::hex::FromHex;

use hyper;
use hyper::status::StatusCode;

use random_filename;
use SyncStatus;
use util::{Mutex, H256};
use util::sha3::sha3;
use page::{LocalPageEndpoint, PageCache};
use handlers::{ContentHandler, ContentFetcherHandler, ContentValidator};
use endpoint::{Endpoint, EndpointPath, Handler};
use apps::cache::{ContentCache, ContentStatus};
use apps::manifest::{MANIFEST_FILENAME, deserialize_manifest, serialize_manifest, Manifest};
use apps::urlhint::{URLHintContract, URLHint, URLHintResult};

/// Limit of cached dapps/content
const MAX_CACHED_DAPPS: usize = 20;

pub struct ContentFetcher<R: URLHint = URLHintContract> {
	dapps_path: PathBuf,
	resolver: R,
	cache: Arc<Mutex<ContentCache>>,
	sync: Arc<SyncStatus>,
	embeddable_on: Option<(String, u16)>,
}

impl<R: URLHint> Drop for ContentFetcher<R> {
	fn drop(&mut self) {
		// Clear cache path
		let _ = fs::remove_dir_all(&self.dapps_path);
	}
}

impl<R: URLHint> ContentFetcher<R> {

	pub fn new(resolver: R, sync_status: Arc<SyncStatus>, embeddable_on: Option<(String, u16)>) -> Self {
		let mut dapps_path = env::temp_dir();
		dapps_path.push(random_filename());

		ContentFetcher {
			dapps_path: dapps_path,
			resolver: resolver,
			sync: sync_status,
			cache: Arc::new(Mutex::new(ContentCache::default())),
			embeddable_on: embeddable_on,
		}
	}

	fn still_syncing(address: Option<(String, u16)>) -> Box<Handler> {
		Box::new(ContentHandler::error(
			StatusCode::ServiceUnavailable,
			"Sync In Progress",
			"Your node is still syncing. We cannot resolve any content before it's fully synced.",
			Some("<a href=\"javascript:window.location.reload()\">Refresh</a>"),
			address,
		))
	}

	#[cfg(test)]
	fn set_status(&self, content_id: &str, status: ContentStatus) {
		self.cache.lock().insert(content_id.to_owned(), status);
	}

	pub fn contains(&self, content_id: &str) -> bool {
		{
			let mut cache = self.cache.lock();
			// Check if we already have the app
			if cache.get(content_id).is_some() {
				return true;
			}
		}
		// fallback to resolver
		if let Ok(content_id) = content_id.from_hex() {
			// else try to resolve the app_id
			let has_content = self.resolver.resolve(content_id).is_some();
			// if there is content or we are syncing return true
			has_content || self.sync.is_major_importing()
		} else {
			false
		}
	}

	pub fn to_async_handler(&self, path: EndpointPath, control: hyper::Control) -> Box<Handler> {
		let mut cache = self.cache.lock();
		let content_id = path.app_id.clone();

		let (new_status, handler) = {
			let status = cache.get(&content_id);
			match status {
				// Just serve the content
				Some(&mut ContentStatus::Ready(ref endpoint)) => {
					(None, endpoint.to_async_handler(path, control))
				},
				// Content is already being fetched
				Some(&mut ContentStatus::Fetching(ref fetch_control)) => {
					trace!(target: "dapps", "Content fetching in progress. Waiting...");
					(None, fetch_control.to_handler(control))
				},
				// We need to start fetching the content
				None => {
					trace!(target: "dapps", "Content unavailable. Fetching... {:?}", content_id);
					let content_hex = content_id.from_hex().expect("to_handler is called only when `contains` returns true.");
					let content = self.resolver.resolve(content_hex);

					let cache = self.cache.clone();
					let on_done = move |id: String, result: Option<LocalPageEndpoint>| {
						let mut cache = cache.lock();
						match result {
							Some(endpoint) => {
								cache.insert(id, ContentStatus::Ready(endpoint));
							},
							// In case of error
							None => {
								cache.remove(&id);
							},
						}
					};

					match content {
						// Don't serve dapps if we are still syncing (but serve content)
						Some(URLHintResult::Dapp(_)) if self.sync.is_major_importing() => {
							(None, Self::still_syncing(self.embeddable_on.clone()))
						},
						Some(URLHintResult::Dapp(dapp)) => {
							let (handler, fetch_control) = ContentFetcherHandler::new(
								dapp.url(),
								control,
								DappInstaller {
									id: content_id.clone(),
									dapps_path: self.dapps_path.clone(),
									on_done: Box::new(on_done),
									embeddable_on: self.embeddable_on.clone(),
								},
								self.embeddable_on.clone(),
							);

							(Some(ContentStatus::Fetching(fetch_control)), Box::new(handler) as Box<Handler>)
						},
						Some(URLHintResult::Content(content)) => {
							let (handler, fetch_control) = ContentFetcherHandler::new(
								content.url,
								control,
								ContentInstaller {
									id: content_id.clone(),
									mime: content.mime,
									content_path: self.dapps_path.clone(),
									on_done: Box::new(on_done),
								},
								self.embeddable_on.clone(),
							);

							(Some(ContentStatus::Fetching(fetch_control)), Box::new(handler) as Box<Handler>)
						},
						None if self.sync.is_major_importing() => {
							(None, Self::still_syncing(self.embeddable_on.clone()))
						},
						None => {
							// This may happen when sync status changes in between
							// `contains` and `to_handler`
							(None, Box::new(ContentHandler::error(
								StatusCode::NotFound,
								"Resource Not Found",
								"Requested resource was not found.",
								None,
								self.embeddable_on.clone(),
							)) as Box<Handler>)
						},
					}
				},
			}
		};

		if let Some(status) = new_status {
			cache.clear_garbage(MAX_CACHED_DAPPS);
			cache.insert(content_id, status);
		}

		handler
	}
}

#[derive(Debug)]
pub enum ValidationError {
	Io(io::Error),
	Zip(zip::result::ZipError),
	InvalidContentId,
	ManifestNotFound,
	ManifestSerialization(String),
	HashMismatch { expected: H256, got: H256, },
}

impl fmt::Display for ValidationError {
	fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
		match *self {
			ValidationError::Io(ref io) => write!(f, "Unexpected IO error occured: {:?}", io),
			ValidationError::Zip(ref zip) => write!(f, "Unable to read ZIP archive: {:?}", zip),
			ValidationError::InvalidContentId => write!(f, "ID is invalid. It should be 256 bits keccak hash of content."),
			ValidationError::ManifestNotFound => write!(f, "Downloaded Dapp bundle did not contain valid manifest.json file."),
			ValidationError::ManifestSerialization(ref err) => {
				write!(f, "There was an error during Dapp Manifest serialization: {:?}", err)
			},
			ValidationError::HashMismatch { ref expected, ref got } => {
				write!(f, "Hash of downloaded content did not match. Expected:{:?}, Got:{:?}.", expected, got)
			},
		}
	}
}

impl From<io::Error> for ValidationError {
	fn from(err: io::Error) -> Self {
		ValidationError::Io(err)
	}
}

impl From<zip::result::ZipError> for ValidationError {
	fn from(err: zip::result::ZipError) -> Self {
		ValidationError::Zip(err)
	}
}

struct ContentInstaller {
	id: String,
	mime: String,
	content_path: PathBuf,
	on_done: Box<Fn(String, Option<LocalPageEndpoint>) + Send>,
}

impl ContentValidator for ContentInstaller {
	type Error = ValidationError;

	fn validate_and_install(&self, path: PathBuf) -> Result<(String, LocalPageEndpoint), ValidationError> {
		// Create dir
		try!(fs::create_dir_all(&self.content_path));

		// Validate hash
		let mut file_reader = io::BufReader::new(try!(fs::File::open(&path)));
		let hash = try!(sha3(&mut file_reader));
		let id = try!(self.id.as_str().parse().map_err(|_| ValidationError::InvalidContentId));
		if id != hash {
			return Err(ValidationError::HashMismatch {
				expected: id,
				got: hash,
			});
		}

		// And prepare path for a file
		let filename = path.file_name().expect("We always fetch a file.");
		let mut content_path = self.content_path.clone();
		content_path.push(&filename);

		if content_path.exists() {
			try!(fs::remove_dir_all(&content_path))
		}

		try!(fs::copy(&path, &content_path));

		Ok((self.id.clone(), LocalPageEndpoint::single_file(content_path, self.mime.clone(), PageCache::Enabled)))
	}

	fn done(&self, endpoint: Option<LocalPageEndpoint>) {
		(self.on_done)(self.id.clone(), endpoint)
	}
}


struct DappInstaller {
	id: String,
	dapps_path: PathBuf,
	on_done: Box<Fn(String, Option<LocalPageEndpoint>) + Send>,
	embeddable_on: Option<(String, u16)>,
}

impl DappInstaller {
	fn find_manifest(zip: &mut zip::ZipArchive<fs::File>) -> Result<(Manifest, PathBuf), ValidationError> {
		for i in 0..zip.len() {
			let mut file = try!(zip.by_index(i));

			if !file.name().ends_with(MANIFEST_FILENAME) {
				continue;
			}

			// try to read manifest
			let mut manifest = String::new();
			let manifest = file
				.read_to_string(&mut manifest).ok()
				.and_then(|_| deserialize_manifest(manifest).ok());

			if let Some(manifest) = manifest {
				let mut manifest_location = PathBuf::from(file.name());
				manifest_location.pop(); // get rid of filename
				return Ok((manifest, manifest_location));
			}
		}

		Err(ValidationError::ManifestNotFound)
	}

	fn dapp_target_path(&self, manifest: &Manifest) -> PathBuf {
		let mut target = self.dapps_path.clone();
		target.push(&manifest.id);
		target
	}
}

impl ContentValidator for DappInstaller {
	type Error = ValidationError;

	fn validate_and_install(&self, app_path: PathBuf) -> Result<(String, LocalPageEndpoint), ValidationError> {
		trace!(target: "dapps", "Opening dapp bundle at {:?}", app_path);
		let mut file_reader = io::BufReader::new(try!(fs::File::open(app_path)));
		let hash = try!(sha3(&mut file_reader));
		let id = try!(self.id.as_str().parse().map_err(|_| ValidationError::InvalidContentId));
		if id != hash {
			return Err(ValidationError::HashMismatch {
				expected: id,
				got: hash,
			});
		}
		let file = file_reader.into_inner();
		// Unpack archive
		let mut zip = try!(zip::ZipArchive::new(file));
		// First find manifest file
		let (mut manifest, manifest_dir) = try!(Self::find_manifest(&mut zip));
		// Overwrite id to match hash
		manifest.id = self.id.clone();

		let target = self.dapp_target_path(&manifest);

		// Remove old directory
		if target.exists() {
			warn!(target: "dapps", "Overwriting existing dapp: {}", manifest.id);
			try!(fs::remove_dir_all(target.clone()));
		}

		// Unpack zip
		for i in 0..zip.len() {
			let mut file = try!(zip.by_index(i));
			// TODO [todr] Check if it's consistent on windows.
			let is_dir = file.name().chars().rev().next() == Some('/');

			let file_path = PathBuf::from(file.name());
			let location_in_manifest_base = file_path.strip_prefix(&manifest_dir);
			// Create files that are inside manifest directory
			if let Ok(location_in_manifest_base) = location_in_manifest_base {
				let p = target.join(location_in_manifest_base);
				// Check if it's a directory
				if is_dir {
					try!(fs::create_dir_all(p));
				} else {
					let mut target = try!(fs::File::create(p));
					try!(io::copy(&mut file, &mut target));
				}
			}
		}

		// Write manifest
		let manifest_str = try!(serialize_manifest(&manifest).map_err(ValidationError::ManifestSerialization));
		let manifest_path = target.join(MANIFEST_FILENAME);
		let mut manifest_file = try!(fs::File::create(manifest_path));
		try!(manifest_file.write_all(manifest_str.as_bytes()));

		// Create endpoint
		let app = LocalPageEndpoint::new(target, manifest.clone().into(), PageCache::Enabled, self.embeddable_on.clone());

		// Return modified app manifest
		Ok((manifest.id.clone(), app))
	}

	fn done(&self, endpoint: Option<LocalPageEndpoint>) {
		(self.on_done)(self.id.clone(), endpoint)
	}
}

#[cfg(test)]
mod tests {
	use std::env;
	use std::sync::Arc;
	use util::Bytes;
	use endpoint::EndpointInfo;
	use page::LocalPageEndpoint;
	use apps::cache::ContentStatus;
	use apps::urlhint::{URLHint, URLHintResult};
	use super::ContentFetcher;

	struct FakeResolver;
	impl URLHint for FakeResolver {
		fn resolve(&self, _id: Bytes) -> Option<URLHintResult> {
			None
		}
	}

	#[test]
	fn should_true_if_contains_the_app() {
		// given
		let path = env::temp_dir();
		let fetcher = ContentFetcher::new(FakeResolver, Arc::new(|| false), None);
		let handler = LocalPageEndpoint::new(path, EndpointInfo {
			name: "fake".into(),
			description: "".into(),
			version: "".into(),
			author: "".into(),
			icon_url: "".into(),
		}, Default::default(), None);

		// when
		fetcher.set_status("test", ContentStatus::Ready(handler));
		fetcher.set_status("test2", ContentStatus::Fetching(Default::default()));

		// then
		assert_eq!(fetcher.contains("test"), true);
		assert_eq!(fetcher.contains("test2"), true);
		assert_eq!(fetcher.contains("test3"), false);
	}
}
