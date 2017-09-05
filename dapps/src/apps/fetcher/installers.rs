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

use zip;
use std::{fs, fmt};
use std::io::{self, Read, Write};
use std::path::PathBuf;
use fetch::{self, Mime};
use hash::keccak_buffer;
use bigint::hash::H256;

use page::{LocalPageEndpoint, PageCache};
use handlers::{ContentValidator, ValidatorResponse};
use apps::manifest::{MANIFEST_FILENAME, deserialize_manifest, serialize_manifest, Manifest};
use Embeddable;

type OnDone = Box<Fn(Option<LocalPageEndpoint>) + Send>;

fn write_response_and_check_hash(
	id: &str,
	mut content_path: PathBuf,
	filename: &str,
	response: fetch::Response
) -> Result<(fs::File, PathBuf), ValidationError> {
	// try to parse id
	let id = id.parse().map_err(|_| ValidationError::InvalidContentId)?;

	// check if content exists
	if content_path.exists() {
		warn!(target: "dapps", "Overwriting existing content at 0x{:?}", id);
		fs::remove_dir_all(&content_path)?
	}

	// create directory
	fs::create_dir_all(&content_path)?;

	// append filename
	content_path.push(filename);

	// Now write the response
	let mut file = io::BufWriter::new(fs::File::create(&content_path)?);
	let mut reader = io::BufReader::new(response);
	io::copy(&mut reader, &mut file)?;
	file.flush()?;

	// Validate hash
	// TODO [ToDr] calculate keccak in-flight while reading the response
	let mut file = io::BufReader::new(fs::File::open(&content_path)?);
	let hash = keccak_buffer(&mut file)?;
	if id == hash {
		Ok((file.into_inner(), content_path))
	} else {
		Err(ValidationError::HashMismatch {
			expected: id,
			got: hash,
		})
	}
}

pub struct Content {
	id: String,
	mime: Mime,
	content_path: PathBuf,
	on_done: OnDone,
}

impl Content {
	pub fn new(id: String, mime: Mime, content_path: PathBuf, on_done: OnDone) -> Self {
		Content {
			id: id,
			mime: mime,
			content_path: content_path,
			on_done: on_done,
		}
	}
}

impl ContentValidator for Content {
	type Error = ValidationError;

	fn validate_and_install(&self, response: fetch::Response) -> Result<ValidatorResponse, ValidationError> {
		let validate = |content_path: PathBuf| {
			// Create dir
			let (_, content_path) = write_response_and_check_hash(self.id.as_str(), content_path.clone(), self.id.as_str(), response)?;

			Ok(LocalPageEndpoint::single_file(content_path, self.mime.clone(), PageCache::Enabled))
		};

		// Prepare path for a file
		let content_path = self.content_path.join(&self.id);
		// Make sure to always call on_done (even in case of errors)!
		let result = validate(content_path.clone());
		// remove the file if there was an error
		if result.is_err() {
			// Ignore errors since the file might not exist
			let _ = fs::remove_dir_all(&content_path);
		}
		(self.on_done)(result.as_ref().ok().cloned());
		result.map(ValidatorResponse::Local)
	}
}

pub struct Dapp {
	id: String,
	dapps_path: PathBuf,
	on_done: OnDone,
	embeddable_on: Embeddable,
}

impl Dapp {
	pub fn new(id: String, dapps_path: PathBuf, on_done: OnDone, embeddable_on: Embeddable) -> Self {
		Dapp {
			id,
			dapps_path,
			on_done,
			embeddable_on,
		}
	}

	fn find_manifest(zip: &mut zip::ZipArchive<fs::File>) -> Result<(Manifest, PathBuf), ValidationError> {
		for i in 0..zip.len() {
			let mut file = zip.by_index(i)?;

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
}

impl ContentValidator for Dapp {
	type Error = ValidationError;

	fn validate_and_install(&self, response: fetch::Response) -> Result<ValidatorResponse, ValidationError> {
		let validate = |dapp_path: PathBuf| {
			let (file, zip_path) = write_response_and_check_hash(self.id.as_str(), dapp_path.clone(), &format!("{}.zip", self.id), response)?;
			trace!(target: "dapps", "Opening dapp bundle at {:?}", zip_path);
			// Unpack archive
			let mut zip = zip::ZipArchive::new(file)?;
			// First find manifest file
			let (mut manifest, manifest_dir) = Self::find_manifest(&mut zip)?;
			// Overwrite id to match hash
			manifest.id = self.id.clone();

			// Unpack zip
			for i in 0..zip.len() {
				let mut file = zip.by_index(i)?;
				let is_dir = file.name().chars().rev().next() == Some('/');

				let file_path = PathBuf::from(file.name());
				let location_in_manifest_base = file_path.strip_prefix(&manifest_dir);
				// Create files that are inside manifest directory
				if let Ok(location_in_manifest_base) = location_in_manifest_base {
					let p = dapp_path.join(location_in_manifest_base);
					// Check if it's a directory
					if is_dir {
						fs::create_dir_all(p)?;
					} else {
						let mut target = fs::File::create(p)?;
						io::copy(&mut file, &mut target)?;
					}
				}
			}

			// Remove zip
			fs::remove_file(&zip_path)?;

			// Write manifest
			let manifest_str = serialize_manifest(&manifest).map_err(ValidationError::ManifestSerialization)?;
			let manifest_path = dapp_path.join(MANIFEST_FILENAME);
			let mut manifest_file = fs::File::create(manifest_path)?;
			manifest_file.write_all(manifest_str.as_bytes())?;
			// Create endpoint
			let endpoint = LocalPageEndpoint::new(dapp_path, manifest.clone().into(), PageCache::Enabled, self.embeddable_on.clone());
			Ok(endpoint)
		};

		// Prepare directory for dapp
		let target = self.dapps_path.join(&self.id);
		// Validate the dapp
		let result = validate(target.clone());
		// remove the file if there was an error
		if result.is_err() {
			// Ignore errors since the file might not exist
			let _ = fs::remove_dir_all(&target);
		}
		(self.on_done)(result.as_ref().ok().cloned());
		result.map(ValidatorResponse::Local)
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
