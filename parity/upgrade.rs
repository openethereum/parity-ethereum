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

//! Parity upgrade logic

use semver::Version;
use std::collections::*;
use std::fs::{File, create_dir_all};
use std::env;
use std::io::{Read, Write};

#[cfg_attr(feature="dev", allow(enum_variant_names))]
#[derive(Debug)]
pub enum Error {
	CannotCreateConfigPath,
	CannotWriteVersionFile,
	CannotUpdateVersionFile,
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
	println!("Adding ver.lock");
	Ok(())
}

fn push_upgrades(upgrades: &mut UpgradeList)
{
	// dummy upgrade (remove when the first one is in)
	upgrades.insert(
		UpgradeKey { old_version: Version::parse("0.9.0").unwrap(), new_version: Version::parse("1.0.0").unwrap() },
		dummy_upgrade);
}

fn upgrade_from_version(previous_version: &Version) -> Result<usize, Error> {
	let mut upgrades = HashMap::new();
	push_upgrades(&mut upgrades);

	let current_version = Version::parse(CURRENT_VERSION).unwrap();

	let mut count = 0;
	for upgrade_key in upgrades.keys() {
		if upgrade_key.is_applicable(previous_version, &current_version) {
			let upgrade_script = upgrades[upgrade_key];
			try!(upgrade_script());
			count = count + 1;
		}
	}
	Ok(count)
}

fn with_locked_version<F>(db_path: Option<&str>, script: F) -> Result<usize, Error>
	where F: Fn(&Version) -> Result<usize, Error>
{
	let mut path = db_path.map_or({
		let mut path = env::home_dir().expect("Applications should have a home dir");
		path.push(".parity");
		path
	}, |s| ::std::path::PathBuf::from(s));
	try!(create_dir_all(&path).map_err(|_| Error::CannotCreateConfigPath));
	path.push("ver.lock");

	let version =
		File::open(&path).ok().and_then(|ref mut file|
			{
				let mut version_string = String::new();
				file.read_to_string(&mut version_string)
					.ok()
					.and_then(|_| Version::parse(&version_string).ok())
			})
			.unwrap_or_else(|| Version::parse("0.9.0").unwrap());

	let mut lock = try!(File::create(&path).map_err(|_| Error::CannotWriteVersionFile));
	let result = script(&version);

	let written_version = Version::parse(CURRENT_VERSION).unwrap();
	try!(lock.write_all(written_version.to_string().as_bytes()).map_err(|_| Error::CannotUpdateVersionFile));
	result
}

pub fn upgrade(db_path: Option<&str>) -> Result<usize, Error> {
	with_locked_version(db_path, |ver| {
		upgrade_from_version(ver)
	})
}
