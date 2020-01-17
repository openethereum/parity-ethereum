// Copyright 2015-2020 Parity Technologies (UK) Ltd.
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

//! Parity upgrade logic

use semver::{Version, SemVerError};
use std::collections::*;
use std::fs::{self, File, create_dir_all};
use std::io;
use std::io::{Read, Write};
use std::path::{PathBuf, Path};
use dir::{DatabaseDirectories, default_data_path, home_dir};
use dir::helpers::replace_home;
use journaldb::Algorithm;

#[derive(Debug)]
pub enum Error {
	CannotCreateConfigPath(io::Error),
	CannotWriteVersionFile(io::Error),
	CannotUpdateVersionFile(io::Error),
	SemVer(SemVerError),
}

impl From<SemVerError> for Error {
	fn from(err: SemVerError) -> Self {
		Error::SemVer(err)
	}
}

const CURRENT_VERSION: &'static str = env!("CARGO_PKG_VERSION");

#[derive(Hash, PartialEq, Eq)]
struct UpgradeKey {
	pub old_version: Version,
	pub new_version: Version,
}

type UpgradeList = HashMap<UpgradeKey, fn() -> Result<(), Error>>;

impl UpgradeKey {
	// given the following config exist
	// ver.lock 1.1 (`previous_version`)
	//
	//  current_version 1.4 (`current_version`)
	//
	//
	//upgrades (set of `UpgradeKey`)
	//	1.0 -> 1.1 (u1)
	//	1.1 -> 1.2 (u2)
	//	1.2 -> 1.3 (u3)
	//	1.3 -> 1.4 (u4)
	//	1.4 -> 1.5 (u5)
	//
	// then the following upgrades should be applied:
	// u2, u3, u4
	fn is_applicable(&self, previous_version: &Version, current_version: &Version) -> bool {
		self.old_version >= *previous_version && self.new_version <= *current_version
	}
}

// dummy upgrade (remove when the first one is in)
fn dummy_upgrade() -> Result<(), Error> {
	Ok(())
}

fn push_upgrades(upgrades: &mut UpgradeList)
{
	// dummy upgrade (remove when the first one is in)
	upgrades.insert(
		UpgradeKey { old_version: Version::new(0, 9, 0), new_version: Version::new(1, 0, 0)},
		dummy_upgrade);
}

fn upgrade_from_version(previous_version: &Version) -> Result<usize, Error> {
	let mut upgrades = HashMap::new();
	push_upgrades(&mut upgrades);

	let current_version = Version::parse(CURRENT_VERSION)?;

	let mut count = 0;
	for upgrade_key in upgrades.keys() {
		if upgrade_key.is_applicable(previous_version, &current_version) {
			let upgrade_script = upgrades[upgrade_key];
			upgrade_script()?;
			count += 1;
		}
	}
	Ok(count)
}

fn with_locked_version<F>(db_path: &str, script: F) -> Result<usize, Error>
	where F: Fn(&Version) -> Result<usize, Error>
{
	let mut path = PathBuf::from(db_path);
	create_dir_all(&path).map_err(Error::CannotCreateConfigPath)?;
	path.push("ver.lock");

	let version =
		File::open(&path).ok().and_then(|ref mut file|
			{
				let mut version_string = String::new();
				file.read_to_string(&mut version_string)
					.ok()
					.and_then(|_| Version::parse(&version_string).ok())
			})
			.unwrap_or(Version::new(0, 9, 0));

	let mut lock = File::create(&path).map_err(Error::CannotWriteVersionFile)?;
	let result = script(&version);

	let written_version = Version::parse(CURRENT_VERSION)?;
	lock.write_all(written_version.to_string().as_bytes()).map_err(Error::CannotUpdateVersionFile)?;
	result
}

pub fn upgrade(db_path: &str) -> Result<usize, Error> {
	with_locked_version(db_path, |ver| {
		upgrade_from_version(ver)
	})
}

fn file_exists(path: &Path) -> bool {
	match fs::metadata(&path) {
		Err(ref e) if e.kind() == io::ErrorKind::NotFound => false,
		_ => true,
	}
}

#[cfg(any(test, feature = "accounts"))]
pub fn upgrade_key_location(from: &PathBuf, to: &PathBuf) {
	match fs::create_dir_all(&to).and_then(|()| fs::read_dir(from)) {
		Ok(entries) => {
			let files: Vec<_> = entries.filter_map(|f| f.ok().and_then(|f| if f.file_type().ok().map_or(false, |f| f.is_file()) { f.file_name().to_str().map(|s| s.to_owned()) } else { None })).collect();
			let mut num: usize = 0;
			for name in files {
				let mut from = from.clone();
				from.push(&name);
				let mut to = to.clone();
				to.push(&name);
				if !file_exists(&to) {
					if let Err(e) = fs::rename(&from, &to) {
						debug!("Error upgrading key {:?}: {:?}", from, e);
					} else {
						num += 1;
					}
				} else {
					debug!("Skipped upgrading key {:?}", from);
				}
			}
			if num > 0 {
				info!("Moved {} keys from {} to {}", num, from.to_string_lossy(), to.to_string_lossy());
			}
		},
		Err(e) => {
			debug!("Error moving keys from {:?} to {:?}: {:?}", from, to, e);
		}
	}
}

fn upgrade_dir_location(source: &PathBuf, dest: &PathBuf) {
	if file_exists(&source) {
		if !file_exists(&dest) {
			let mut parent = dest.clone();
			parent.pop();
			if let Err(e) = fs::create_dir_all(&parent).and_then(|()| fs::rename(&source, &dest)) {
				debug!("Skipped path {:?} -> {:?} :{:?}", source, dest, e);
			} else {
				info!("Moved {} to {}", source.to_string_lossy(), dest.to_string_lossy());
			}
		} else {
			debug!("Skipped upgrading directory {:?}, Destination already exists at {:?}", source, dest);
		}
	}
}

fn upgrade_user_defaults(dirs: &DatabaseDirectories) {
	let source = dirs.legacy_user_defaults_path();
	let dest = dirs.user_defaults_path();
	if file_exists(&source) {
		if !file_exists(&dest) {
			if let Err(e) = fs::rename(&source, &dest) {
				debug!("Skipped upgrading user defaults {:?}:{:?}", dest, e);
			}
		} else {
			debug!("Skipped upgrading user defaults {:?}, File exists at {:?}", source, dest);
		}
	}
}

pub fn upgrade_data_paths(base_path: &str, dirs: &DatabaseDirectories, pruning: Algorithm) {
	if home_dir().is_none() {
		return;
	}

	let legacy_root_path = replace_home("", "$HOME/.parity");
	let default_path = default_data_path();
	if legacy_root_path != base_path && base_path == default_path {
		upgrade_dir_location(&PathBuf::from(legacy_root_path), &PathBuf::from(&base_path));
	}
	upgrade_dir_location(&dirs.legacy_version_path(pruning), &dirs.db_path(pruning));
	upgrade_dir_location(&dirs.legacy_snapshot_path(), &dirs.snapshot_path());
	upgrade_dir_location(&dirs.legacy_network_path(), &dirs.network_path());
	upgrade_user_defaults(&dirs);
}
