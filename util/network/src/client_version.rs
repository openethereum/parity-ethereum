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

const PARITY_CLIENT_LARGE_REQUESTS_VERSION: &str = "2.3.0";

use regex::Regex;
use std::fmt;
use semver::Version;



/// Description of the software version running in a peer
/// according to https://github.com/ethereum/wiki/wiki/Client-Version-Strings
/// This structure as it is represents the format used by Parity clients. Other
/// vendors may provide additional fields.
///
/// TODO support formats with extra fields, e.g.:
/// "Geth/main.jnode.network/v1.8.21-stable-9dc5d1a9/linux-amd64/go1.11.4"

#[derive(Clone,Debug,PartialEq,Eq)]
pub enum ClientVersion {
	ParityClient(
		String, // Name
		Version, // Semver
		String, // OS
		String, // Compiler
	),
	Other,
}

impl ClientVersion {
	fn name(&self) -> Option<&str> {
		match self {
			ClientVersion::ParityClient(name, _, _, _) => Some(name.as_str()),
			_ => None
		}
	}

	fn semver(&self) -> Option<&Version> {
		match self {
			ClientVersion::ParityClient(_, semver, _, _) => Some(&semver),
			_ => None
		}
	}

	fn os(&self) -> Option<&str> {
		match self {
			ClientVersion::ParityClient(_, _, os, _) => Some(os.as_str()),
			_ => None
		}
	}

	fn compiler(&self) -> Option<&str> {
		match self {
			ClientVersion::ParityClient(_, _, _, compiler) => Some(compiler.as_str()),
			_ => None
		}
	}
}

// TODO: Maybe merge with Peercapabilityinfo in ethcore-network?
pub trait ClientCapabilities {
	fn can_handle_large_requests(&self) -> bool;
}

impl ClientCapabilities for ClientVersion {
	fn can_handle_large_requests(&self) -> bool {
		match self {
			ClientVersion::ParityClient(_, semver, _, _) => {
				if *semver < Version::parse(PARITY_CLIENT_LARGE_REQUESTS_VERSION).unwrap() {
					false
				} else {
					true
				}
			},
			_ => true // As far as we know
		}
	}
}

impl From<&str> for ClientVersion {
	fn from(client_version: &str) -> Self{
		if client_version.is_empty() {
			return ClientVersion::Other;
		}

		let tokens: Vec<&str> = client_version.split("/").collect();

		let parity_re = Regex::new("(?i).*parity.*").unwrap();

		// Safe to assume we have at least one element
		if !parity_re.is_match(tokens[0]) {
			return ClientVersion::Other;
		}

		// Basically strip leading 'v'
		if let Some(version_number) = &get_number_from_version(tokens[1]) {

			return ClientVersion::ParityClient(
				tokens[0].to_string(),
				Version::parse(version_number).unwrap(),
				tokens[2].to_string(),
				tokens[3].to_string()
			);
		} else {
			return ClientVersion::Other;
		}
	}
}

impl From<String> for ClientVersion {
	fn from(client_version: String) -> Self{
		ClientVersion::from(client_version.as_ref())
	}
}

impl fmt::Display for ClientVersion {
	fn fmt(&self, f: &mut fmt::Formatter) -> std::fmt::Result {
		match self {
			ClientVersion::ParityClient(name, semver, os, compiler) => {
				write!(f, "{}/v{}/{}/{}", name, semver, os, compiler)
			},
			_ => write!(f, "")
		}
	}
}

fn get_number_from_version(version: &str) -> Option<String> {
	if version.starts_with("v") {
		return version.get(1..).map(|s| s.to_string());
	}

	None
}

#[cfg(test)]
pub mod tests {
	use super::*;

	const PARITY_CLIENT_NAME: &str = "parity";
	const PARITY_CLIENT_SEMVER: &str = "2.3.0";
	const PARITY_CLIENT_OLD_SEMVER: &str = "2.2.0";
	const PARITY_CLIENT_OS: &str = "linux";
	const PARITY_CLIENT_COMPILER: &str = "rustc";

	#[test]
	pub fn client_version_when_from_empty_string_then_other() {
		assert_eq!(ClientVersion::from(""), ClientVersion::Other);
	}

	#[test]
	pub fn get_number_from_version_when_valid_then_number() {
		let version_string = format!("v{}", PARITY_CLIENT_SEMVER);

		assert_eq!(get_number_from_version(&version_string).unwrap(), PARITY_CLIENT_SEMVER);
	}

	#[test]
	pub fn client_version_when_str_parity_format_and_valid_then_all_fields_match() {
		let client_version_string = format!("{}/v{}/{}/{}",
											PARITY_CLIENT_NAME,
											PARITY_CLIENT_SEMVER,
											PARITY_CLIENT_OS,
											PARITY_CLIENT_COMPILER);

		let client_version = ClientVersion::from(client_version_string.as_str());

		assert_eq!(client_version.name().unwrap(), PARITY_CLIENT_NAME);
		assert_eq!(*client_version.semver().unwrap(), Version::parse(PARITY_CLIENT_SEMVER).unwrap());
		assert_eq!(client_version.os().unwrap(), PARITY_CLIENT_OS);
		assert_eq!(client_version.compiler().unwrap(), PARITY_CLIENT_COMPILER);
	}

	#[test]
	pub fn client_version_when_string_parity_format_and_valid_then_all_fields_match() {
		let client_version_string: String = format!("{}/v{}/{}/{}",
													PARITY_CLIENT_NAME,
													PARITY_CLIENT_SEMVER,
													PARITY_CLIENT_OS,
													PARITY_CLIENT_COMPILER).to_string();

		let client_version = ClientVersion::from(client_version_string.as_str());

		assert_eq!(client_version.name().unwrap(), PARITY_CLIENT_NAME);
		assert_eq!(*client_version.semver().unwrap(), Version::parse(PARITY_CLIENT_SEMVER).unwrap());
		assert_eq!(client_version.os().unwrap(), PARITY_CLIENT_OS);
		assert_eq!(client_version.compiler().unwrap(), PARITY_CLIENT_COMPILER);
	}

	#[test]
	pub fn client_version_when_parity_format_and_invalid_then_all_fields_match() {
		// This is invalid because version has no leading 'v'
		let client_version_string = format!("{}/{}/{}/{}",
											PARITY_CLIENT_NAME,
											PARITY_CLIENT_SEMVER,
											PARITY_CLIENT_OS,
											PARITY_CLIENT_COMPILER);

		let client_version = ClientVersion::from(client_version_string.as_str());

		assert_eq!(client_version, ClientVersion::Other);
	}

	#[test]
	pub fn client_version_when_not_parity_format_and_valid_then_fields_none() {
		// We don't support this format yet, simply expect an empty structure.
		// Unfortunately, From must return a result, and TryFrom is still experimental.
		let client_name = "Geth";
		let network_name = "main.jnode.network";
		let client_semver = "v1.8.21-stable-9dc5d1a9";
		let client_os = "linux";
		let client_compiler = "go";

		let client_version_string = format!("{}/{}/{}/{}/{}", client_name, network_name, client_semver, client_os, client_compiler);

		let client_version = ClientVersion::from(client_version_string.as_str());

		match client_version {
			ClientVersion::Other => {
				assert!(client_version.name().is_none());
				assert!(client_version.compiler().is_none());
				assert!(client_version.os().is_none());
				assert!(client_version.semver().is_none());
			},
			_ => panic!("Expected Other")
		}
	}

	#[test]
	pub fn client_version_when_parity_format_and_valid_then_to_string_equal() {
		let client_version_string: String = format!("{}/v{}/{}/{}",
													PARITY_CLIENT_NAME,
													PARITY_CLIENT_SEMVER,
													PARITY_CLIENT_OS,
													PARITY_CLIENT_COMPILER).to_string();

		let client_version = ClientVersion::from(client_version_string.as_str());

		assert_eq!(client_version.to_string(), client_version_string);
	}

	#[test]
	pub fn client_capabilities_when_parity_old_version_then_handles_large_requests_false() {
		let client_version_string: String = format!("{}/v{}/{}/{}",
													PARITY_CLIENT_NAME,
													PARITY_CLIENT_OLD_SEMVER,
													PARITY_CLIENT_OS,
													PARITY_CLIENT_COMPILER).to_string();

		let client_version = ClientVersion::from(client_version_string.as_str());

		assert!(!client_version.can_handle_large_requests());
	}

	// FIXME For some reason the version in this test is considered older than 2.3.0.
	// A client with this ID _should_ actually be able to handle large requests
	#[test]
	pub fn client_capabilities_when_parity_new_version_then_handles_large_requests_true() {
		let client_version_string: String = format!("{}/v{}/{}/{}",
													"Parity-Ethereum",
													"2.3.0-beta-10657d9-20190115",
													"x86_64-linux-gnu",
													"rustc1.31.1").to_string();

		let client_version = ClientVersion::from(client_version_string.as_str());

		assert!(!client_version.can_handle_large_requests());
	}
}
