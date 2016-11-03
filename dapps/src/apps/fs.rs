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

use std::io;
use std::io::Read;
use std::fs;
use std::path::PathBuf;
use page::{LocalPageEndpoint, PageCache};
use endpoint::{Endpoints, EndpointInfo};
use apps::manifest::{MANIFEST_FILENAME, deserialize_manifest};

struct LocalDapp {
	id: String,
	path: PathBuf,
	info: EndpointInfo,
}

fn local_dapps(dapps_path: String) -> Vec<LocalDapp> {
	let files = fs::read_dir(dapps_path.as_str());
	if let Err(e) = files {
		warn!(target: "dapps", "Unable to load local dapps from: {}. Reason: {:?}", dapps_path, e);
		return vec![];
	}

	let files = files.expect("Check is done earlier");
	files.map(|dir| {
			let entry = try!(dir);
			let file_type = try!(entry.file_type());

			// skip files
			if file_type.is_file() {
				return Err(io::Error::new(io::ErrorKind::NotFound, "Not a file"));
			}

			// take directory name and path
			entry.file_name().into_string()
				.map(|name| (name, entry.path()))
				.map_err(|e| {
					info!(target: "dapps", "Unable to load dapp: {:?}. Reason: {:?}", entry.path(), e);
					io::Error::new(io::ErrorKind::NotFound, "Invalid name")
				})
		})
		.filter_map(|m| {
			if let Err(ref e) = m {
				debug!(target: "dapps", "Ignoring local dapp: {:?}", e);
			}
			m.ok()
		})
		.map(|(name, path)| {
			// try to get manifest file
			let info = read_manifest(&name, path.clone());
			LocalDapp {
				id: name,
				path: path,
				info: info,
			}
		})
		.collect()
}

fn read_manifest(name: &str, mut path: PathBuf) -> EndpointInfo {
	path.push(MANIFEST_FILENAME);

	fs::File::open(path.clone())
		.map_err(|e| format!("{:?}", e))
		.and_then(|mut f| {
			// Reat file
			let mut s = String::new();
			try!(f.read_to_string(&mut s).map_err(|e| format!("{:?}", e)));
			// Try to deserialize manifest
			deserialize_manifest(s)
		})
		.map(Into::into)
		.unwrap_or_else(|e| {
			warn!(target: "dapps", "Cannot read manifest file at: {:?}. Error: {:?}", path, e);

			EndpointInfo {
				name: name.into(),
				description: name.into(),
				version: "0.0.0".into(),
				author: "?".into(),
				icon_url: "icon.png".into(),
			}
		})
}

pub fn local_endpoints(dapps_path: String, signer_port: Option<u16>) -> Endpoints {
	let mut pages = Endpoints::new();
	for dapp in local_dapps(dapps_path) {
		pages.insert(
			dapp.id,
			Box::new(LocalPageEndpoint::new(dapp.path, dapp.info, PageCache::Disabled, signer_port))
		);
	}
	pages
}
