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

use zip;
use zip::result::ZipError;
use std::fs;
use std::io::{self, Read};
use std::path::PathBuf;
use std::sync::Arc;
use std::collections::HashSet;
use rustc_serialize::hex::ToHex;

use hyper::Control;
use hyper::status::StatusCode;

use util::{Address, FromHex, Mutex};
use apps::manifest::{MANIFEST_FILENAME, deserialize_manifest, Manifest};
use handlers::{ContentHandler, AppFetcherHandler};
use endpoint::{EndpointPath, Handler};

#[derive(Debug)]
pub struct GithubApp {
	pub account: String,
	pub repo: String,
	pub commit: [u8;20],
	pub owner: Address,
}

impl GithubApp {
	pub fn url(&self) -> String {
		// format!("https://github.com/{}/{}/archive/{}.zip", self.account, self.repo, self.commit.to_hex())
		format!("http://github.todr.me/{}/{}/zip/{}", self.account, self.repo, self.commit.to_hex())
	}

	fn commit(bytes: &[u8]) -> [u8;20] {
		let mut commit = [0; 20];
		for i in 0..20 {
			commit[i] = bytes[i];
		}
		commit
	}
}

pub struct AppFetcher {
	dapps_path: PathBuf,
	in_progress: Arc<Mutex<HashSet<String>>>,
}

impl AppFetcher {

	pub fn new(dapps_path: &str) -> Self {
		AppFetcher {
			dapps_path: PathBuf::from(dapps_path),
			in_progress: Arc::new(Mutex::new(HashSet::new())),
		}
	}

	fn resolve(&self, app_id: &str) -> Option<GithubApp> {
		// TODO [todr] use GithubHint contract to check the details
		// For now we are just accepting patterns: <commithash>.<repo>.<account>.parity

		let mut app_parts = app_id.split('.');
		let hash = app_parts.next().and_then(|h| h.from_hex().ok());
		let repo = app_parts.next();
		let account = app_parts.next();

		match (hash, repo, account) {
			(Some(hash), Some(repo), Some(account)) => {
				Some(GithubApp {
					account: account.into(),
					repo: repo.into(),
					commit: GithubApp::commit(&hash),
					owner: Address::default(),
				})
			},
			_ => None,
		}
	}

	pub fn can_resolve(&self, app_id: &str) -> bool {
		self.resolve(app_id).is_some()
	}

	pub fn to_handler(&self, path: EndpointPath, control: Control) -> Box<Handler> {
		{
			let mut in_progress = self.in_progress.lock();
			if in_progress.contains(&path.app_id) {
				return Box::new(ContentHandler::html(
					StatusCode::ServiceUnavailable,
					"<h1>This dapp is already being downloaded.</h1>".into()
				));
			}
			in_progress.insert(path.app_id.clone());
		}

		let app = self.resolve(&path.app_id).expect("to_handler is called only when `can_resolve` returns true.");

		let in_progress = self.in_progress.clone();
		let app_id = path.app_id.clone();
		Box::new(AppFetcherHandler::new(
			app,
			self.dapps_path.clone(),
			control,
			move || {
				in_progress.lock().remove(&app_id);
			}
		))
	}

}

#[derive(Debug)]
pub enum ValidationError {
	ManifestNotFound,
	Io(io::Error),
	Zip(ZipError),
}

impl From<io::Error> for ValidationError {
	fn from(err: io::Error) -> Self {
		ValidationError::Io(err)
	}
}

impl From<ZipError> for ValidationError {
	fn from(err: ZipError) -> Self {
		ValidationError::Zip(err)
	}
}

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
	return Err(ValidationError::ManifestNotFound);
}

pub fn validate_and_install_app(mut target: PathBuf, app_path: PathBuf) -> Result<String, ValidationError> {
	trace!(target: "dapps", "Opening dapp bundle at {:?}", app_path);
	let file = try!(fs::File::open(app_path));
	// Unpack archive
	let mut zip = try!(zip::ZipArchive::new(file));
	// First find manifest file
	let (manifest, manifest_dir) = try!(find_manifest(&mut zip));
	target.push(&manifest.id);

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

	Ok(manifest.id)
}


// 1. [x] Wait for response (with some timeout)
// 2. Validate (Check hash)
// 3. [x] Unpack to ~/.parity/dapps
// 4. [x] Display errors or refresh to load again from memory / FS
// 5. Mark as volatile?
//    Keep a list of "installed" apps?
//    Serve from memory?
//
// 6. Hosts validation?
// 7. [x] Mutex on dapp
