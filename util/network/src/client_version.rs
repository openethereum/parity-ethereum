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

#![warn(missing_docs)]

//! Parse ethereum client ID strings and provide querying functionality

use semver::Version;
use std::fmt;

/// Parity client string prefix
const LEGACY_CLIENT_ID_PREFIX: &str = "Parity";
const PARITY_CLIENT_ID_PREFIX: &str = "Parity-Ethereum";

lazy_static! {
/// Parity versions starting from this will accept block bodies requests
/// of 256 bodies
	static ref PARITY_CLIENT_LARGE_REQUESTS_VERSION: Version = Version::parse("2.4.0").unwrap();
}

/// Description of the software version running in a peer
/// according to https://github.com/ethereum/wiki/wiki/Client-Version-Strings
/// This structure as it is represents the format used by Parity clients. Other
/// vendors may provide additional fields.
#[derive(Clone,Debug,PartialEq,Eq,Serialize)]
pub struct ParityClientData {
	name: String,
	identity: Option<String>,
	semver: Version,
	os: String,
	compiler: String,

	// Capability flags, should be calculated in constructor
	can_handle_large_requests: bool,
}

/// Accessor methods for ParityClientData. This will probably
/// need to be abstracted away into a trait.
impl ParityClientData {
	fn new(
		name: String,
		identity: Option<String>,
		semver: Version,
		os: String,
		compiler: String,
	) -> Self {
		// Flags logic
		let can_handle_large_requests = &semver >= &PARITY_CLIENT_LARGE_REQUESTS_VERSION;

		// Instantiate and return
		ParityClientData {
			name: name,
			identity: identity,
			semver: semver,
			os: os,
			compiler: compiler,

			can_handle_large_requests: can_handle_large_requests,
		}
	}

	fn name(&self) -> &str {
		self.name.as_str()
	}

	fn identity(&self) -> Option<&str> {
		self.identity.as_ref().map(String::as_str)
	}

	fn semver(&self) -> &Version {
		&self.semver
	}

	fn os(&self) -> &str {
		self.os.as_str()
	}

	fn compiler(&self) -> &str {
		self.compiler.as_str()
	}

	fn can_handle_large_requests(&self) -> bool {
		self.can_handle_large_requests
	}
}

/// Enum describing the version of the software running on a peer.
#[derive(Clone,Debug,Eq,PartialEq,Serialize)]
pub enum ClientVersion {
	/// The peer runs software from parity and the string format is known
	ParityClient(
		/// The actual information fields: name, version, os, ...
		ParityClientData
	),
	/// The string ID is recognized as Parity but the overall format
	/// could not be parsed
	ParityUnknownFormat(String),
	/// Other software vendors than Parity
	Other(String),
}

impl Default for ClientVersion {
	fn default() -> Self {
		ClientVersion::Other("".to_owned())
	}
}

/// Provide information about what a particular version of a
/// peer software can do
pub trait ClientCapabilities {
	/// Parity versions before PARITY_CLIENT_LARGE_REQUESTS_VERSION would not
	/// check the accumulated size of a packet when building a response to a
	/// GET_BLOCK_BODIES request. If the packet was larger than a given limit,
	/// instead of sending fewer blocks no packet would get sent at all. Query
	/// if this version can handle requests for a large number of block bodies.
	fn can_handle_large_requests(&self) -> bool;

	/// Service transactions are specific to parity. Query if this version
	/// accepts them.
	fn accepts_service_transaction(&self) -> bool;
}

impl ClientCapabilities for ClientVersion {
	fn can_handle_large_requests(&self) -> bool {
		match self {
			ClientVersion::ParityClient(data) => data.can_handle_large_requests(),
			ClientVersion::ParityUnknownFormat(_) => false, // Play it safe
			ClientVersion::Other(_) => true // As far as we know
		}
	}

	fn accepts_service_transaction(&self) -> bool {
		match self {
			ClientVersion::ParityClient(_) => true,
			ClientVersion::ParityUnknownFormat(_) => true,
			ClientVersion::Other(_) => false
		}
	}

}

fn is_parity(client_id: &str) -> bool {
	client_id.starts_with(LEGACY_CLIENT_ID_PREFIX) || client_id.starts_with(PARITY_CLIENT_ID_PREFIX)
}

/// Parse known parity formats. Recognizes either a short format with four fields
/// or a long format which includes the same fields and an identity one.
fn parse_parity_format(client_version: &str) -> Result<ParityClientData, ()> {
	const PARITY_ID_STRING_MINIMUM_TOKENS: usize = 4;

	let tokens: Vec<&str> = client_version.split("/").collect();

	if tokens.len() < PARITY_ID_STRING_MINIMUM_TOKENS {
		return Err(())
	}

	let name = tokens[0];

	let identity = if tokens.len() - 3 > 1 {
		Some(tokens[1..(tokens.len() - 3)].join("/"))
	} else {
		None
	};

	let compiler = tokens[tokens.len() - 1];
	let os = tokens[tokens.len() - 2];

	// If version is in the right position and valid format return a valid
	// result. Otherwise return an error.
	get_number_from_version(tokens[tokens.len() - 3])
		.and_then(|v| Version::parse(v).ok())
		.map(|semver| ParityClientData::new(
			name.to_owned(),
			identity,
			semver,
			os.to_owned(),
			compiler.to_owned(),
		))
		.ok_or(())
}

/// Parse a version string and return the corresponding
/// ClientVersion. Only Parity clients are destructured right now, other
/// strings will just get wrapped in a variant so that the information is
/// not lost.
/// The parsing for parity may still fail, in which case return a ParityUnknownFormat with
/// the original version string. TryFrom would be a better trait to implement.
impl<T> From<T> for ClientVersion
where T: AsRef<str> {
	fn from(client_version: T) -> Self {
		let client_version_str: &str = client_version.as_ref();

		if !is_parity(client_version_str) {
			return ClientVersion::Other(client_version_str.to_owned());
		}

		if let Ok(data) = parse_parity_format(client_version_str) {
			ClientVersion::ParityClient(data)
		} else {
			ClientVersion::ParityUnknownFormat(client_version_str.to_owned())
		}
	}
}

fn format_parity_version_string(client_version: &ParityClientData, f: &mut fmt::Formatter) -> std::fmt::Result {
	let name = client_version.name();
	let semver = client_version.semver();
	let os = client_version.os();
	let compiler = client_version.compiler();

	match client_version.identity() {
		None => write!(f, "{}/v{}/{}/{}", name, semver, os, compiler),
		Some(identity) => write!(f, "{}/{}/v{}/{}/{}", name, identity, semver, os, compiler),
	}
}

impl fmt::Display for ClientVersion {
	fn fmt(&self, f: &mut fmt::Formatter) -> std::fmt::Result {
		match self {
			ClientVersion::ParityClient(data) => format_parity_version_string(data, f),
			ClientVersion::ParityUnknownFormat(id) => write!(f, "{}", id),
			ClientVersion::Other(id) => write!(f, "{}", id)
		}
	}
}

fn get_number_from_version(version: &str) -> Option<&str> {
	if version.starts_with("v") {
		return version.get(1..);
	}

	None
}

#[cfg(test)]
pub mod tests {
	use super::*;

	const PARITY_CLIENT_SEMVER: &str = "2.4.0";
	const PARITY_CLIENT_OLD_SEMVER: &str = "2.2.0";
	const PARITY_CLIENT_OS: &str = "linux";
	const PARITY_CLIENT_COMPILER: &str = "rustc";
	const PARITY_CLIENT_IDENTITY: &str = "ExpanseSOLO";
	const PARITY_CLIENT_MULTITOKEN_IDENTITY: &str = "ExpanseSOLO/abc/v1.2.3";


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
			PARITY_CLIENT_IDENTITY,
			PARITY_CLIENT_SEMVER,
			PARITY_CLIENT_OS,
			PARITY_CLIENT_COMPILER
		)
	}

	fn make_multitoken_identity_long_version_string() -> String {
		format!(
			"{}/{}/v{}/{}/{}",
			PARITY_CLIENT_ID_PREFIX,
			PARITY_CLIENT_MULTITOKEN_IDENTITY,
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
	pub fn client_version_when_from_empty_string_then_default() {
		let default = ClientVersion::default();

		assert_eq!(ClientVersion::from(""), default);
	}

	#[test]
	pub fn get_number_from_version_when_valid_then_number() {
		let version_string = format!("v{}", PARITY_CLIENT_SEMVER);

		assert_eq!(get_number_from_version(&version_string).unwrap(), PARITY_CLIENT_SEMVER);
	}

	#[test]
	pub fn client_version_when_str_parity_format_and_valid_then_all_fields_match() {
		let client_version_string = make_default_version_string();

		if let ClientVersion::ParityClient(client_version) = ClientVersion::from(client_version_string.as_str()) {
			assert_eq!(client_version.name(), PARITY_CLIENT_ID_PREFIX);
			assert_eq!(*client_version.semver(), Version::parse(PARITY_CLIENT_SEMVER).unwrap());
			assert_eq!(client_version.os(), PARITY_CLIENT_OS);
			assert_eq!(client_version.compiler(), PARITY_CLIENT_COMPILER);
		} else {
			panic!("shouldn't be here");
		}
	}

	#[test]
	pub fn client_version_when_str_parity_long_format_and_valid_then_all_fields_match() {
		let client_version_string = make_default_long_version_string();

		if let ClientVersion::ParityClient(client_version) = ClientVersion::from(client_version_string.as_str()) {
			assert_eq!(client_version.name(), PARITY_CLIENT_ID_PREFIX);
			assert_eq!(client_version.identity().unwrap(), PARITY_CLIENT_IDENTITY);
			assert_eq!(*client_version.semver(), Version::parse(PARITY_CLIENT_SEMVER).unwrap());
			assert_eq!(client_version.os(), PARITY_CLIENT_OS);
			assert_eq!(client_version.compiler(), PARITY_CLIENT_COMPILER);
		} else {
			panic!("shouldnt be here");
		}
	}

	#[test]
	pub fn client_version_when_str_parity_long_format_and_valid_and_identity_multiple_tokens_then_all_fields_match() {
		let client_version_string = make_multitoken_identity_long_version_string();

		if let ClientVersion::ParityClient(client_version) = ClientVersion::from(client_version_string.as_str()) {
			assert_eq!(client_version.name(), PARITY_CLIENT_ID_PREFIX);
			assert_eq!(client_version.identity().unwrap(), PARITY_CLIENT_MULTITOKEN_IDENTITY);
			assert_eq!(*client_version.semver(), Version::parse(PARITY_CLIENT_SEMVER).unwrap());
			assert_eq!(client_version.os(), PARITY_CLIENT_OS);
			assert_eq!(client_version.compiler(), PARITY_CLIENT_COMPILER);
		} else {
			panic!("shouldnt be here");
		}
	}

	#[test]
	pub fn client_version_when_string_parity_format_and_valid_then_all_fields_match() {
		let client_version_string: String = make_default_version_string();

		if let ClientVersion::ParityClient(client_version) = ClientVersion::from(client_version_string.as_str()) {
			assert_eq!(client_version.name(), PARITY_CLIENT_ID_PREFIX);
			assert_eq!(*client_version.semver(), Version::parse(PARITY_CLIENT_SEMVER).unwrap());
			assert_eq!(client_version.os(), PARITY_CLIENT_OS);
			assert_eq!(client_version.compiler(), PARITY_CLIENT_COMPILER);
		} else {
			panic!("shouldn't be here");
		}
	}

	#[test]
	pub fn client_version_when_parity_format_and_invalid_then_equals_parity_unknown_client_version_string() {
		// This is invalid because version has no leading 'v'
		let client_version_string = format!(
			"{}/{}/{}/{}",
			PARITY_CLIENT_ID_PREFIX,
			PARITY_CLIENT_SEMVER,
			PARITY_CLIENT_OS,
			PARITY_CLIENT_COMPILER);

		let client_version = ClientVersion::from(client_version_string.as_str());

		let parity_unknown = ClientVersion::ParityUnknownFormat(client_version_string.to_string());

		assert_eq!(client_version, parity_unknown);
	}

	#[test]
	pub fn client_version_when_parity_format_without_identity_and_missing_compiler_field_then_equals_parity_unknown_client_version_string() {
		let client_version_string = format!(
			"{}/v{}/{}",
			PARITY_CLIENT_ID_PREFIX,
			PARITY_CLIENT_SEMVER,
			PARITY_CLIENT_OS,
			);

		let client_version = ClientVersion::from(client_version_string.as_str());

		let parity_unknown = ClientVersion::ParityUnknownFormat(client_version_string.to_string());

		assert_eq!(client_version, parity_unknown);
	}

	#[test]
	pub fn client_version_when_parity_format_with_identity_and_missing_compiler_field_then_equals_parity_unknown_client_version_string() {
		let client_version_string = format!(
			"{}/{}/v{}/{}",
			PARITY_CLIENT_ID_PREFIX,
			PARITY_CLIENT_IDENTITY,
			PARITY_CLIENT_SEMVER,
			PARITY_CLIENT_OS,
			);

		let client_version = ClientVersion::from(client_version_string.as_str());

		let parity_unknown = ClientVersion::ParityUnknownFormat(client_version_string.to_string());

		assert_eq!(client_version, parity_unknown);
	}

	#[test]
	pub fn client_version_when_not_parity_format_and_valid_then_other_with_client_version_string() {
		let client_version_string = "Geth/main.jnode.network/v1.8.21-stable-9dc5d1a9/linux";

		let client_version = ClientVersion::from(client_version_string);

		assert_eq!(client_version, ClientVersion::Other(client_version_string.to_string()));
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
		let client_version_string: String = make_old_semver_version_string();

		let client_version = ClientVersion::from(client_version_string.as_str());

		assert!(!client_version.can_handle_large_requests());
	}

	#[test]
	pub fn client_capabilities_when_parity_beta_version_then_not_handles_large_requests_true() {
		let client_version_string: String = format!(
			"{}/v{}/{}/{}",
			"Parity-Ethereum",
			"2.4.0-beta",
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
