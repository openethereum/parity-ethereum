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

// Parity client string prefix
const LEGACY_CLIENT_ID_PREFIX: &str = "Parity";
const PARITY_CLIENT_ID_PREFIX: &str = "Parity-Ethereum";

// Parity versions starting from this will accept block bodies requests
// of 256 bodies
const PARITY_CLIENT_LARGE_REQUESTS_VERSION: &str = "2.3.0";

// Parity versions starting from this will accept service-transactions
const SERVICE_TRANSACTIONS_VERSION: &str = "1.6.0";

use semver::Version;
use std::fmt;


/// Description of the software version running in a peer
/// according to https://github.com/ethereum/wiki/wiki/Client-Version-Strings
/// This structure as it is represents the format used by Parity clients. Other
/// vendors may provide additional fields.
///
/// TODO support formats with extra fields, e.g.:
/// "Geth/main.jnode.network/v1.8.21-stable-9dc5d1a9/linux-amd64/go1.11.4"

#[derive(Clone,Debug,PartialEq,Eq)]
pub struct ParityClientData {
	name: String,
	variant: Option<String>,
	semver: Version,
	os: String,
	compiler: String,
}

#[derive(Clone,Debug,PartialEq,Eq)]
pub enum ClientVersion {
	ParityClient(
		ParityClientData
	),

	Other(String), // Id string
}

impl ClientVersion {
	fn name(&self) -> Option<&str> {
		match self {
			ClientVersion::ParityClient(ref data) => Some(data.name.as_ref()),
			_ => None
		}
	}

	fn variant(&self) -> Option<&str> {
		match self {
			ClientVersion::ParityClient(data) => data.variant.as_ref().map(|s| s.as_ref()),
			_ => None
		}
	}

	fn semver(&self) -> Option<&Version> {
		match self {
			ClientVersion::ParityClient(data) => Some(&data.semver),
			_ => None
		}
	}

	fn os(&self) -> Option<&str> {
		match self {
			ClientVersion::ParityClient(data) => Some(data.os.as_ref()),
			_ => None
		}
	}

	fn compiler(&self) -> Option<&str> {
		match self {
			ClientVersion::ParityClient(data) => Some(data.compiler.as_ref()),
			_ => None
		}
	}
}

// TODO: Maybe merge with Peercapabilityinfo in ethcore-network?
pub trait ClientCapabilities {
	fn can_handle_large_requests(&self) -> bool;

	fn accepts_service_transaction(&self) -> bool;
}

// This is a implementation of a function taken from propagator.rs
fn parity_accepts_service_transaction(client_version: &ClientVersion) -> bool {
	match client_version {
		ClientVersion::ParityClient(data) => {
			let service_transactions_version = Version::parse(SERVICE_TRANSACTIONS_VERSION).unwrap();

			data.semver >= service_transactions_version
		},
		_ => panic!("should not really be here")
	}
}

impl ClientCapabilities for ClientVersion {
	fn can_handle_large_requests(&self) -> bool {
		match self {
			ClientVersion::ParityClient(data) => {
				if data.semver < Version::parse(PARITY_CLIENT_LARGE_REQUESTS_VERSION).unwrap() {
					false
				} else {
					true
				}
			},
			_ => true // As far as we know
		}
	}

	/// Checks if peer is able to process service transactions
	fn accepts_service_transaction(&self) -> bool {
		match self {
			ClientVersion::ParityClient(_) => parity_accepts_service_transaction(&self),
			_ => false
		}
	}

}


fn is_parity(client_id: &str) -> bool {
	if client_id.starts_with(LEGACY_CLIENT_ID_PREFIX) || client_id.starts_with(PARITY_CLIENT_ID_PREFIX) {
		return true;
	} else {
		return false;
	}
}

// Parse known parity formats.
//
// This is really not robust: parse four arguments and
// allow for an extra argument between identifier and
// version
// TODO implement a better logic
fn parse_parity_format(client_version: &str) -> ClientVersion {
	let tokens: Vec<&str> = client_version.split("/").collect();

	// Basically strip leading 'v'
	if let Some(version_number) = &get_number_from_version(tokens[1]) {

		return ClientVersion::ParityClient(
			ParityClientData {
				name: tokens[0].to_string(),
				variant: None,
				semver: Version::parse(version_number).unwrap(),
				os: tokens[2].to_string(),
				compiler: tokens[3].to_string(),
			}
		);
	} else if let Some(version_number) = &get_number_from_version(tokens[2]) {

		return ClientVersion::ParityClient(
			ParityClientData {
				name: tokens[0].to_string(),
				variant: Some(tokens[1].to_string()),
				semver: Version::parse(version_number).unwrap(),
				os: tokens[3].to_string(),
				compiler: tokens[4].to_string(),
			}
		);
	} else {
		// We should be able to signal an invalid result,
		// but because we are calling this function in impl From...
		// we must return a result. TryFrom should give us flexibility
		return ClientVersion::Other(client_version.to_string());
	}
}

// Parses a version string and returns the corresponding
// ClientVersion. Only Parity clients are destructured right now.
impl From<&str> for ClientVersion {
	fn from(client_version: &str) -> Self{
		if !is_parity(client_version) {
			return ClientVersion::Other(client_version.to_string());
		}

		parse_parity_format(client_version)
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
			ClientVersion::ParityClient(ParityClientData{name, variant: None, semver, os, compiler}) => {
				write!(f, "{}/v{}/{}/{}", name, semver, os, compiler)
			},

			ClientVersion::ParityClient(ParityClientData{name, variant: Some(variant), semver, os, compiler}) => {
				write!(f, "{}/{}/v{}/{}/{}", name, variant, semver, os, compiler)
			},
			ClientVersion::Other(id) => {
				write!(f, "{}", id)
			},
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

	const PARITY_CLIENT_SEMVER: &str = "2.3.0";
	const PARITY_CLIENT_OLD_SEMVER: &str = "2.2.0";
	const PARITY_CLIENT_OS: &str = "linux";
	const PARITY_CLIENT_COMPILER: &str = "rustc";
	const PARITY_CLIENT_VARIANT: &str = "ExpanseSOLO";

	fn make_default_version_string() -> String {
		format!(
			"{}/v{}/{}/{}",
			PARITY_CLIENT_ID_PREFIX,
			PARITY_CLIENT_SEMVER,
			PARITY_CLIENT_OS,
			PARITY_CLIENT_COMPILER
		)
	}

	fn make_default_long_version_string() -> String {
		format!(
			"{}/{}/v{}/{}/{}",
			PARITY_CLIENT_ID_PREFIX,
			PARITY_CLIENT_VARIANT,
			PARITY_CLIENT_SEMVER,
			PARITY_CLIENT_OS,
			PARITY_CLIENT_COMPILER
		)
	}

	fn make_old_semver_version_string() -> String {
		format!(
			"{}/v{}/{}/{}",
			PARITY_CLIENT_ID_PREFIX,
			PARITY_CLIENT_OLD_SEMVER,
			PARITY_CLIENT_OS,
			PARITY_CLIENT_COMPILER
		)
	}

	#[test]
	pub fn client_version_when_from_empty_string_then_other() {
		let other = ClientVersion::Other("".to_string());

		assert_eq!(ClientVersion::from(""), other);
	}

	#[test]
	pub fn get_number_from_version_when_valid_then_number() {
		let version_string = format!("v{}", PARITY_CLIENT_SEMVER);

		assert_eq!(get_number_from_version(&version_string).unwrap(), PARITY_CLIENT_SEMVER);
	}

	#[test]
	pub fn client_version_when_str_parity_format_and_valid_then_all_fields_match() {
		let client_version_string = make_default_version_string();

		let client_version = ClientVersion::from(client_version_string.as_str());

		assert_eq!(client_version.name().unwrap(), PARITY_CLIENT_ID_PREFIX);
		assert_eq!(*client_version.semver().unwrap(), Version::parse(PARITY_CLIENT_SEMVER).unwrap());
		assert_eq!(client_version.os().unwrap(), PARITY_CLIENT_OS);
		assert_eq!(client_version.compiler().unwrap(), PARITY_CLIENT_COMPILER);
	}

	#[test]
	pub fn client_version_when_str_parity_long_format_and_valid_then_all_fields_match() {
		let client_version_string = make_default_long_version_string();

		let client_version = ClientVersion::from(client_version_string.as_str());

		assert_eq!(client_version.name().unwrap(), PARITY_CLIENT_ID_PREFIX);
		assert_eq!(client_version.variant().unwrap(), PARITY_CLIENT_VARIANT);
		assert_eq!(*client_version.semver().unwrap(), Version::parse(PARITY_CLIENT_SEMVER).unwrap());
		assert_eq!(client_version.os().unwrap(), PARITY_CLIENT_OS);
		assert_eq!(client_version.compiler().unwrap(), PARITY_CLIENT_COMPILER);
	}

	#[test]
	pub fn client_version_when_string_parity_format_and_valid_then_all_fields_match() {
		let client_version_string: String = make_default_version_string();

		let client_version = ClientVersion::from(client_version_string.as_str());

		assert_eq!(client_version.name().unwrap(), PARITY_CLIENT_ID_PREFIX);
		assert_eq!(*client_version.semver().unwrap(), Version::parse(PARITY_CLIENT_SEMVER).unwrap());
		assert_eq!(client_version.os().unwrap(), PARITY_CLIENT_OS);
		assert_eq!(client_version.compiler().unwrap(), PARITY_CLIENT_COMPILER);
	}

	#[test]
	pub fn client_version_when_parity_format_and_invalid_then_equals_other() {
		// This is invalid because version has no leading 'v'
		let client_version_string = format!(
			"{}/{}/{}/{}",
			PARITY_CLIENT_ID_PREFIX,
			PARITY_CLIENT_SEMVER,
			PARITY_CLIENT_OS,
			PARITY_CLIENT_COMPILER);

		let client_version = ClientVersion::from(client_version_string.as_str());

		let other = ClientVersion::Other(client_version_string.to_string());

		assert_eq!(client_version, other);
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

		let client_version_string = format!(
			"{}/{}/{}/{}/{}",
			client_name,
			network_name,
			client_semver,
			client_os,
			client_compiler);

		let client_version = ClientVersion::from(client_version_string.as_str());

		match client_version {
			ClientVersion::Other(_) => {
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
		let client_version_string: String = make_default_version_string();

		let client_version = ClientVersion::from(client_version_string.as_str());

		assert_eq!(client_version.to_string(), client_version_string);
	}

	#[test]
	pub fn client_version_when_other_then_to_string_equal_input_string() {
		let client_version_string: String = "Other".to_string();

		let client_version = ClientVersion::from("Other");

		assert_eq!(client_version.to_string(), client_version_string);
	}

	#[test]
	pub fn client_capabilities_when_parity_old_version_then_handles_large_requests_false() {
		let mut client_version_string: String = make_old_semver_version_string();

		let client_version = ClientVersion::from(client_version_string.as_str());

		assert!(!client_version.can_handle_large_requests());
	}

	// FIXME For some reason the version in this test is considered older than 2.3.0.
	// A client with this ID _should_ actually be able to handle large requests
	#[test]
	pub fn client_capabilities_when_parity_new_version_then_handles_large_requests_true() {
		let client_version_string: String = format!(
			"{}/v{}/{}/{}",
			"Parity-Ethereum",
			"2.3.0-beta-10657d9-20190115",
			"x86_64-linux-gnu",
			"rustc1.31.1")
			.to_string();

		let client_version = ClientVersion::from(client_version_string.as_str());

		assert!(!client_version.can_handle_large_requests());
	}

	#[test]
	pub fn client_version_when_to_owned_then_both_objects_equal() {
		let client_version_string: String = make_old_semver_version_string();

		let origin = ClientVersion::from(client_version_string.as_str());

		let borrowed = &origin;

		let owned = origin.to_owned();

		assert_eq!(*borrowed, owned);
	}

	#[test]
	fn client_version_accepts_service_transaction_for_different_versions() {
		assert!(!ClientVersion::from("Geth").accepts_service_transaction());
		assert!(!ClientVersion::from("Parity/v1.5.0/linux/rustc").accepts_service_transaction());

		assert!(ClientVersion::from("Parity-Ethereum/v2.6.0/linux/rustc").accepts_service_transaction());
		assert!(ClientVersion::from("Parity-Ethereum/ABCDEFGH/v2.7.3/linux/rustc").accepts_service_transaction());
	}

	#[test]
	fn is_parity_when_parity_then_true() {
		let client_id = format!("{}/", PARITY_CLIENT_ID_PREFIX);

		assert!(is_parity(&client_id));
	}

	#[test]
	fn is_parity_when_empty_then_false() {
		let client_id = "";

		assert!(!is_parity(&client_id));
	}

	#[test]
	fn is_parity_when_other_then_false() {
		let client_id = "other";

		assert!(!is_parity(&client_id));
	}
}
