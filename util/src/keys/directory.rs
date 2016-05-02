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

//! Keys Directory

use common::*;
use std::path::{PathBuf};

const CURRENT_DECLARED_VERSION: u64 = 3;
const MAX_KEY_FILE_LEN: u64 = 1024 * 80;
const MAX_CACHE_USAGE_TRACK: usize = 128;

/// Cipher type (currently only aes-128-ctr)
#[derive(PartialEq, Debug, Clone)]
pub enum CryptoCipherType {
	/// aes-128-ctr with 128-bit initialisation vector(iv)
	Aes128Ctr(H128)
}

#[derive(PartialEq, Debug, Clone)]
enum KeyFileVersion {
	V3(u64)
}

/// key generator function
#[derive(PartialEq, Debug, Clone)]
pub enum Pbkdf2CryptoFunction {
	/// keyed-hash generator (HMAC-256)
	HMacSha256
}

#[derive(Clone)]
/// Kdf of type `Pbkdf2`
/// https://en.wikipedia.org/wiki/PBKDF2
pub struct KdfPbkdf2Params {
	/// desired length of the derived key, in octets
	pub dk_len: u32,
	/// cryptographic salt
	pub salt: H256,
	/// number of iterations for derived key
	pub c: u32,
	/// pseudo-random 2-parameters function
	pub prf: Pbkdf2CryptoFunction
}

#[derive(Debug)]
enum Pbkdf2ParseError {
	InvalidParameter(&'static str),
	InvalidPrf(Mismatch<String>),
	InvalidSaltFormat(UtilError),
	MissingParameter(&'static str),
}

impl KdfPbkdf2Params {
	fn from_json(json: &BTreeMap<String, Json>) -> Result<KdfPbkdf2Params, Pbkdf2ParseError> {
		Ok(KdfPbkdf2Params{
			salt: match try!(json.get("salt").ok_or(Pbkdf2ParseError::MissingParameter("salt"))).as_string() {
				None => { return Err(Pbkdf2ParseError::InvalidParameter("salt")) },
				Some(salt_value) => match H256::from_str(salt_value) {
					Ok(salt_hex_value) => salt_hex_value,
					Err(from_hex_error) => { return Err(Pbkdf2ParseError::InvalidSaltFormat(from_hex_error)); },
				}
			},
			prf: match try!(json.get("prf").ok_or(Pbkdf2ParseError::MissingParameter("prf"))).as_string() {
				Some("hmac-sha256") => Pbkdf2CryptoFunction::HMacSha256,
				Some(unexpected_prf) => { return Err(Pbkdf2ParseError::InvalidPrf(Mismatch { expected: "hmac-sha256".to_owned(), found: unexpected_prf.to_owned() })); },
				None => { return Err(Pbkdf2ParseError::InvalidParameter("prf")); },
			},
			dk_len: try!(try!(json.get("dklen").ok_or(Pbkdf2ParseError::MissingParameter("dklen"))).as_u64().ok_or(Pbkdf2ParseError::InvalidParameter("dkLen"))) as u32,
			c: try!(try!(json.get("c").ok_or(Pbkdf2ParseError::MissingParameter("c"))).as_u64().ok_or(Pbkdf2ParseError::InvalidParameter("c"))) as u32,
		})
	}

	fn to_json(&self) -> Json {
		let mut map = BTreeMap::new();
		map.insert("dklen".to_owned(), json_from_u32(self.dk_len));
		map.insert("salt".to_owned(), Json::String(format!("{:?}", self.salt)));
		map.insert("prf".to_owned(), Json::String("hmac-sha256".to_owned()));
		map.insert("c".to_owned(), json_from_u32(self.c));
		Json::Object(map)
	}
}

#[derive(Clone)]
/// Kdf of type `Scrypt`.
/// https://en.wikipedia.org/wiki/Scrypt
pub struct KdfScryptParams {
	/// Desired length of the derived key, in octets.
	pub dk_len: u32,
	/// Parallelization parameter.
	pub p: u32,
	/// CPU/memory cost parameter.
	pub n: u32,
	/// TODO: comment
	pub r: u32,
	/// Cryptographic salt.
	pub salt: H256,
}

#[derive(Debug)]
enum ScryptParseError {
	InvalidParameter(&'static str),
	InvalidSaltFormat(UtilError),
	MissingParameter(&'static str),
}

fn json_from_u32(number: u32) -> Json { Json::U64(number as u64) }

impl KdfScryptParams {
	fn from_json(json: &BTreeMap<String, Json>) -> Result<KdfScryptParams, ScryptParseError> {
		Ok(KdfScryptParams{
			salt: match try!(json.get("salt").ok_or(ScryptParseError::MissingParameter("salt"))).as_string() {
				None => { return Err(ScryptParseError::InvalidParameter("salt")) },
				Some(salt_value) => match H256::from_str(salt_value) {
					Ok(salt_hex_value) => salt_hex_value,
					Err(from_hex_error) => { return Err(ScryptParseError::InvalidSaltFormat(from_hex_error)); },
				}
			},
			dk_len: try!(try!(json.get("dklen").ok_or(ScryptParseError::MissingParameter("dklen"))).as_u64().ok_or(ScryptParseError::InvalidParameter("dkLen"))) as u32,
			p: try!(try!(json.get("p").ok_or(ScryptParseError::MissingParameter("p"))).as_u64().ok_or(ScryptParseError::InvalidParameter("p"))) as u32,
			n: try!(try!(json.get("n").ok_or(ScryptParseError::MissingParameter("n"))).as_u64().ok_or(ScryptParseError::InvalidParameter("n"))) as u32,
			r: try!(try!(json.get("r").ok_or(ScryptParseError::MissingParameter("r"))).as_u64().ok_or(ScryptParseError::InvalidParameter("r"))) as u32,
		})
	}

	fn to_json(&self) -> Json {
		let mut map = BTreeMap::new();
		map.insert("dklen".to_owned(), json_from_u32(self.dk_len));
		map.insert("salt".to_owned(), Json::String(format!("{:?}", self.salt)));
		map.insert("p".to_owned(), json_from_u32(self.p));
		map.insert("n".to_owned(), json_from_u32(self.n));
		map.insert("r".to_owned(), json_from_u32(self.r));
		Json::Object(map)
	}
}

#[derive(Clone)]
/// Settings for password derived key geberator function.
pub enum KeyFileKdf {
	/// Password-Based Key Derivation Function 2 (PBKDF2) type.
	/// https://en.wikipedia.org/wiki/PBKDF2
	Pbkdf2(KdfPbkdf2Params),
	/// Scrypt password-based key derivation function.
	/// https://en.wikipedia.org/wiki/Scrypt
	Scrypt(KdfScryptParams)
}

#[derive(Clone)]
/// Encrypted password or other arbitrary message
/// with settings for password derived key generator for decrypting content.
pub struct KeyFileCrypto {
	/// Cipher type.
	pub cipher_type: CryptoCipherType,
	/// Cipher text (encrypted message).
	pub cipher_text: Bytes,
	/// Password derived key generator function settings.
	pub kdf: KeyFileKdf,
	/// Mac
	pub mac: H256
}

impl KeyFileCrypto {
	fn from_json(json: &Json) -> Result<KeyFileCrypto, CryptoParseError> {
		let as_object = match json.as_object() {
			None => { return Err(CryptoParseError::InvalidJsonFormat); }
			Some(obj) => obj
		};

		let cipher_type = match try!(as_object.get("cipher").ok_or(CryptoParseError::NoCipherType)).as_string() {
			None => { return Err(CryptoParseError::InvalidCipherType(Mismatch { expected: "aes-128-ctr".to_owned(), found: "not a json string".to_owned() })); }
			Some("aes-128-ctr") => CryptoCipherType::Aes128Ctr(
				match try!(as_object.get("cipherparams").ok_or(CryptoParseError::NoCipherParameters)).as_object() {
					None => { return Err(CryptoParseError::NoCipherParameters); },
					Some(cipher_param) => match H128::from_str(match cipher_param["iv"].as_string() {
							None => { return Err(CryptoParseError::NoInitialVector); },
							Some(iv_hex_string) => iv_hex_string
						})
					{
						Ok(iv_value) => iv_value,
						Err(hex_error) => { return Err(CryptoParseError::InvalidInitialVector(hex_error)); }
					}
				}
			),
			Some(other_cipher_type) => {
				return Err(CryptoParseError::InvalidCipherType(
					Mismatch { expected: "aes-128-ctr".to_owned(), found: other_cipher_type.to_owned() }));
			}
		};

		let kdf = match (try!(as_object.get("kdf").ok_or(CryptoParseError::NoKdf)).as_string(), try!(as_object.get("kdfparams").ok_or(CryptoParseError::NoKdfType)).as_object()) {
			(None, _) => { return Err(CryptoParseError::NoKdfType); },
			(Some("scrypt"), Some(kdf_params)) =>
				match KdfScryptParams::from_json(kdf_params) {
					Err(scrypt_params_error) => { return Err(CryptoParseError::Scrypt(scrypt_params_error)); },
					Ok(scrypt_params) => KeyFileKdf::Scrypt(scrypt_params)
				},
			(Some("pbkdf2"), Some(kdf_params)) =>
				match KdfPbkdf2Params::from_json(kdf_params) {
					Err(pbkdf2_params_error) => { return Err(CryptoParseError::KdfPbkdf2(pbkdf2_params_error)); },
					Ok(pbkdf2_params) => KeyFileKdf::Pbkdf2(pbkdf2_params)
				},
			(Some(other_kdf), _) => {
				return Err(CryptoParseError::InvalidKdfType(
					Mismatch { expected: "pbkdf2/scrypt".to_owned(), found: other_kdf.to_owned()}));
			}
		};

		let cipher_text = match try!(as_object.get("ciphertext").ok_or(CryptoParseError::NoCipherText)).as_string() {
			None => { return Err(CryptoParseError::InvalidCipherText); }
			Some(text) => text
		};

		let mac: H256 = match try!(as_object.get("mac").ok_or(CryptoParseError::NoMac)).as_string() {
			None => { return Err(CryptoParseError::InvalidMacFormat(None)) },
			Some(salt_value) => match H256::from_str(salt_value) {
				Ok(salt_hex_value) => salt_hex_value,
				Err(from_hex_error) => { return Err(CryptoParseError::InvalidMacFormat(Some(from_hex_error))); },
			}
		};

		Ok(KeyFileCrypto {
			cipher_text: match FromHex::from_hex(cipher_text) { Ok(bytes) => bytes, Err(_) => { return Err(CryptoParseError::InvalidCipherText); } },
			cipher_type: cipher_type,
			kdf: kdf,
			mac: mac,
		})
	}

	fn to_json(&self) -> Json {
		let mut map = BTreeMap::new();
		match self.cipher_type {
			CryptoCipherType::Aes128Ctr(ref iv) => {
				map.insert("cipher".to_owned(), Json::String("aes-128-ctr".to_owned()));
				let mut cipher_params = BTreeMap::new();
				cipher_params.insert("iv".to_owned(), Json::String(format!("{:?}", iv)));
				map.insert("cipherparams".to_owned(), Json::Object(cipher_params));
			}
		}
		map.insert("ciphertext".to_owned(), Json::String(
			self.cipher_text.iter().map(|b| format!("{:02x}", b)).collect::<Vec<String>>().join("")));

		map.insert("kdf".to_owned(), Json::String(match self.kdf {
			KeyFileKdf::Pbkdf2(_) => "pbkdf2".to_owned(),
			KeyFileKdf::Scrypt(_) => "scrypt".to_owned()
		}));

		map.insert("kdfparams".to_owned(), match self.kdf {
			KeyFileKdf::Pbkdf2(ref pbkdf2_params) => pbkdf2_params.to_json(),
			KeyFileKdf::Scrypt(ref scrypt_params) => scrypt_params.to_json()
		});

		map.insert("mac".to_owned(), Json::String(format!("{:?}", self.mac)));

		Json::Object(map)
	}

	/// New pbkdf2-type secret.
	/// `cipher-text` - encrypted cipher text.
	/// `dk-len` - desired length of the derived key, in octets.
	/// `c` - number of iterations for derived key.
	/// `salt` - cryptographic site, random 256-bit hash (ensure it's crypto-random).
	/// `iv` - initialisation vector.
	pub fn new_pbkdf2(cipher_text: Bytes, iv: H128, salt: H256, mac: H256, c: u32, dk_len: u32) -> KeyFileCrypto {
		KeyFileCrypto {
			cipher_type: CryptoCipherType::Aes128Ctr(iv),
			cipher_text: cipher_text,
			kdf: KeyFileKdf::Pbkdf2(KdfPbkdf2Params {
				dk_len: dk_len,
				salt: salt,
				c: c,
				prf: Pbkdf2CryptoFunction::HMacSha256
			}),
			mac: mac,
		}
	}
}

/// Universally unique identifier
pub type Uuid = H128;

fn new_uuid() -> Uuid {
	H128::random()
}

fn uuid_to_string(uuid: &Uuid) -> String {
	let d1 = &uuid.as_slice()[0..4];
	let d2 = &uuid.as_slice()[4..6];
	let d3 = &uuid.as_slice()[6..8];
	let d4 = &uuid.as_slice()[8..10];
	let d5 = &uuid.as_slice()[10..16];
	format!("{}-{}-{}-{}-{}", d1.to_hex(), d2.to_hex(), d3.to_hex(), d4.to_hex(), d5.to_hex())
}

fn uuid_from_string(s: &str) -> Result<Uuid, UtilError> {
	let parts: Vec<&str> = s.split('-').collect();
	if parts.len() != 5 { return Err(UtilError::BadSize); }

	let mut uuid = H128::zero();

	if parts[0].len() != 8 { return Err(UtilError::BadSize); }
	uuid[0..4].clone_from_slice(&try!(FromHex::from_hex(parts[0])));
	if parts[1].len() != 4 { return Err(UtilError::BadSize); }
	uuid[4..6].clone_from_slice(&try!(FromHex::from_hex(parts[1])));
	if parts[2].len() != 4 { return Err(UtilError::BadSize); }
	uuid[6..8].clone_from_slice(&try!(FromHex::from_hex(parts[2])));
	if parts[3].len() != 4 { return Err(UtilError::BadSize); }
	uuid[8..10].clone_from_slice(&try!(FromHex::from_hex(parts[3])));
	if parts[4].len() != 12 { return Err(UtilError::BadSize); }
	uuid[10..16].clone_from_slice(&try!(FromHex::from_hex(parts[4])));

	Ok(uuid)
}


#[derive(Clone)]
/// Stored key file struct with encrypted message `(cipher_text)`
/// also contains password derivation function settings (PBKDF2/Scrypt)
pub struct KeyFileContent {
	version: KeyFileVersion,
	/// Holds cypher and decrypt function settings.
	pub crypto: KeyFileCrypto,
	/// The identifier.
	pub id: Uuid,
	/// Account (if present)
	pub account: Option<Address>,
}

#[derive(Debug)]
enum CryptoParseError {
	InvalidMacFormat(Option<UtilError>),
	NoMac,
	NoCipherText,
	InvalidCipherText,
	NoCipherType,
	InvalidJsonFormat,
	InvalidKdfType(Mismatch<String>),
	InvalidCipherType(Mismatch<String>),
	NoInitialVector,
	NoCipherParameters,
	InvalidInitialVector(UtilError),
	NoKdf,
	NoKdfType,
	Scrypt(ScryptParseError),
	KdfPbkdf2(Pbkdf2ParseError)
}

#[derive(Debug)]
enum KeyFileParseError {
	InvalidVersion,
	UnsupportedVersion(OutOfBounds<u64>),
	InvalidJsonFormat,
	InvalidJson,
	InvalidIdentifier,
	NoCryptoSection,
	Crypto(CryptoParseError),
}

impl KeyFileContent {
	/// New stored key file struct with encrypted message (`cipher_text`)
	/// also contains password derivation function settings (PBKDF2/Scrypt)
	/// to decrypt `cipher_text` given the password is provided.
	pub fn new(crypto: KeyFileCrypto) -> KeyFileContent {
		KeyFileContent {
			id: new_uuid(),
			version: KeyFileVersion::V3(3),
			crypto: crypto,
			account: None
		}
	}

	/// Loads key from valid json, returns error and records warning if key is mallformed
	pub fn load(json: &Json) -> Result<KeyFileContent, ()> {
		match Self::from_json(json) {
			Ok(key_file) => Ok(key_file),
			Err(e) => {
				warn!(target: "sstore", "Error parsing json for key: {:?}", e);
				Err(())
			}
		}
	}

	/// Returns key file version if it is known.
	pub fn version(&self) -> Option<u64> {
		match self.version {
			KeyFileVersion::V3(declared) => Some(declared)
		}
	}

	fn from_json(json: &Json) -> Result<KeyFileContent, KeyFileParseError> {
		let as_object = match json.as_object() {
			None => { return Err(KeyFileParseError::InvalidJsonFormat); },
			Some(obj) => obj
		};

		let version = match as_object["version"].as_u64() {
			None => { return Err(KeyFileParseError::InvalidVersion); },
			Some(json_version) => {
				if json_version <= 2 {
					return Err(KeyFileParseError::UnsupportedVersion(OutOfBounds { min: Some(3), max: None, found: json_version }))
				};
				KeyFileVersion::V3(json_version)
			}
		};

		let id_text = try!(as_object.get("id").and_then(|json| json.as_string()).ok_or(KeyFileParseError::InvalidIdentifier));
		let id = match uuid_from_string(&id_text) {
			Err(_) => { return Err(KeyFileParseError::InvalidIdentifier); },
			Ok(id) => id
		};

		let account = as_object.get("address").and_then(|json| json.as_string()).and_then(
			|account_text| match Address::from_str(account_text) { Ok(account) => Some(account), Err(_) => None });

		let crypto = match as_object.get("crypto") {
			None => { return Err(KeyFileParseError::NoCryptoSection); }
			Some(crypto_json) => match KeyFileCrypto::from_json(crypto_json) {
					Ok(crypto) => crypto,
					Err(crypto_error) => { return Err(KeyFileParseError::Crypto(crypto_error)); }
				}
		};

		Ok(KeyFileContent {
			version: version,
			id: id.clone(),
			crypto: crypto,
			account: account
		})
	}

	fn to_json(&self) -> Json {
		let mut map = BTreeMap::new();
		map.insert("id".to_owned(), Json::String(uuid_to_string(&self.id)));
		map.insert("version".to_owned(), Json::U64(CURRENT_DECLARED_VERSION));
		map.insert("crypto".to_owned(), self.crypto.to_json());
		if let Some(ref address) = self.account { map.insert("address".to_owned(), Json::String(format!("{:?}", address))); }
		Json::Object(map)
	}
}

#[derive(Debug)]
enum KeyFileLoadError {
	TooLarge(OutOfBounds<u64>),
	ParseError(KeyFileParseError),
	ReadError(::std::io::Error),
}

/// Represents directory for saving/loading key files.
pub struct KeyDirectory {
	/// Directory path for key management.
	path: String,
	cache: RwLock<HashMap<Uuid, KeyFileContent>>,
	cache_usage: RwLock<VecDeque<Uuid>>,
}

fn restrict_permissions_owner(file_path: &Path) -> Result<(), i32>  {
	let cstr = ::std::ffi::CString::new(file_path.to_str().unwrap()).unwrap();
	match unsafe { ::libc::chmod(cstr.as_ptr(), ::libc::S_IWUSR | ::libc::S_IRUSR) } {
		0 => Ok(()),
		x => Err(x),
	}
}

impl KeyDirectory {
	/// Initializes new cache directory context with a given `path`
	pub fn new(path: &Path) -> KeyDirectory {
		KeyDirectory {
			cache: RwLock::new(HashMap::new()),
			path: path.to_str().expect("Initialized key directory with empty path").to_owned(),
			cache_usage: RwLock::new(VecDeque::new()),
		}
	}

	/// saves (inserts or updates) given key
	pub fn save(&mut self, key_file: KeyFileContent) -> Result<(Uuid), ::std::io::Error> {
		{
			let mut file = try!(fs::File::create(self.key_path(&key_file.id)));
			let json = key_file.to_json();
			let json_text = format!("{}", json.pretty());
			let json_bytes = json_text.into_bytes();
			try!(file.write(&json_bytes));
		}
		if let Err(error_code) = restrict_permissions_owner(self.key_path(&key_file.id).as_path()) {
			fs::remove_file(self.key_path(&key_file.id)).unwrap();
			warn!(target: "sstore", "fatal: failed to modify permissions of the file (chmod: {})", error_code);
			return Err(::std::io::Error::last_os_error());
		}
		let mut cache = self.cache.write().unwrap();
		let id = key_file.id.clone();
		cache.insert(id.clone(), key_file);
		Ok(id.clone())
	}

	/// Returns key given by id if corresponding file exists and no load error occured.
	/// Warns if any error occured during the key loading
	pub fn get(&self, id: &Uuid) -> Option<KeyFileContent> {
		let path = self.key_path(id);
		{
			let mut usage = self.cache_usage.write().unwrap();
			usage.push_back(id.clone());
		}

		if !self.cache.read().unwrap().contains_key(id) {
			match KeyDirectory::load_key(&path) {
				Ok(loaded_key) => {
					self.cache.write().unwrap().insert(id.to_owned(), loaded_key);
				}
				Err(error) => {
					warn!(target: "sstore", "error loading key {:?}: {:?}", id, error);
					return None;
				}
			}
		}

		// todo: replace with Ref::map when it stabilized to avoid copies
		Some(self.cache.read().unwrap().get(id)
			.expect("Key should be there, we have just inserted or checked it.")
			.clone())
	}

	/// Returns current path to the directory with keys
	pub fn path(&self) -> &str {
		&self.path
	}

	/// Removes keys that never been requested during last `MAX_USAGE_TRACK` times
	pub fn collect_garbage(&mut self) {
		let mut cache_usage = self.cache_usage.write().unwrap();

		let total_usages = cache_usage.len();
		let untracked_usages = max(total_usages as i64 - MAX_CACHE_USAGE_TRACK as i64, 0) as usize;
		if untracked_usages > 0 {
			cache_usage.drain(..untracked_usages);
		}

		if self.cache.read().unwrap().len() <= MAX_CACHE_USAGE_TRACK { return; }

		let uniqs: HashSet<&Uuid> = cache_usage.iter().collect();
		let removes:Vec<Uuid> = {
			let cache = self.cache.read().unwrap();
			cache.keys().cloned().filter(|key| !uniqs.contains(key)).collect()
		};
		if removes.is_empty() { return; }
		let mut cache = self.cache.write().unwrap();
		for key in removes { cache.remove(&key); }

		cache.shrink_to_fit();
	}

	/// Reports how many keys are currently cached.
	pub fn cache_size(&self) -> usize {
		self.cache.read().unwrap().len()
	}

	/// Removes key file from key directory
	pub fn delete(&mut self, id: &Uuid) -> Result<(), ::std::io::Error> {
		let path = self.key_path(id);

		if !self.cache.read().unwrap().contains_key(id) {
			return match fs::remove_file(&path) {
				Ok(_) => {
					self.cache.write().unwrap().remove(&id);
					Ok(())
				},
				Err(e) => Err(e)
			};
		}
		Ok(())
	}

	/// Enumerates all keys in the directory
	pub fn list(&self) -> Result<Vec<Uuid>, ::std::io::Error> {
		let mut result = Vec::new();
		for entry in try!(fs::read_dir(&self.path)) {
			let entry = try!(entry);
			if !try!(fs::metadata(entry.path())).is_dir() {
				match entry.file_name().to_str() {
					Some(ref name) => {
						if let Ok(uuid) = uuid_from_string(name) { result.push(uuid); }
					},
					None => { continue; }
				};

			}
		}
		Ok(result)
	}

	fn key_path(&self, id: &Uuid) -> PathBuf {
		let mut path = PathBuf::new();
		path.push(self.path.clone());
		path.push(uuid_to_string(&id));
		path
	}

	fn load_key(path: &PathBuf) -> Result<KeyFileContent, KeyFileLoadError> {
		match fs::File::open(path.clone()) {
			Ok(mut open_file) => {
				match open_file.metadata() {
					Ok(metadata) =>
						if metadata.len() > MAX_KEY_FILE_LEN { Err(KeyFileLoadError::TooLarge(OutOfBounds { min: Some(2), max: Some(MAX_KEY_FILE_LEN), found: metadata.len() })) }
						else { KeyDirectory::load_from_file(&mut open_file) },
					Err(read_error) => Err(KeyFileLoadError::ReadError(read_error))
				}
			},
			Err(read_error) => Err(KeyFileLoadError::ReadError(read_error))
		}
	}

	fn load_from_file(file: &mut fs::File) -> Result<KeyFileContent, KeyFileLoadError> {
		let mut buf = String::new();
		match file.read_to_string(&mut buf) {
			Ok(_) => {},
			Err(read_error) => { return Err(KeyFileLoadError::ReadError(read_error)); }
		}
		match Json::from_str(&buf) {
			Ok(json) => match KeyFileContent::from_json(&json) {
				Ok(key_file_content) => Ok(key_file_content),
				Err(parse_error) => Err(KeyFileLoadError::ParseError(parse_error))
			},
			Err(_) => Err(KeyFileLoadError::ParseError(KeyFileParseError::InvalidJson))
		}
	}


}


#[cfg(test)]
mod file_tests {
	use super::{KeyFileContent, KeyFileVersion, KeyFileKdf, KeyFileParseError, CryptoParseError, uuid_from_string, uuid_to_string, KeyFileCrypto, KdfPbkdf2Params};
	use common::*;

	#[test]
	fn uuid_parses() {
		let uuid = uuid_from_string("3198bc9c-6672-5ab3-d995-4942343ae5b6").unwrap();
		assert!(uuid > H128::zero());
	}

	#[test]
	fn uuid_serializes() {
		let uuid = uuid_from_string("3198bc9c-6fff-5ab3-d995-4942343ae5b6").unwrap();
		assert_eq!(uuid_to_string(&uuid), "3198bc9c-6fff-5ab3-d995-4942343ae5b6");
	}

	#[test]
	fn can_read_keyfile() {
		let json = Json::from_str(
			r#"
				{
					"crypto" : {
						"cipher" : "aes-128-ctr",
						"cipherparams" : {
							"iv" : "6087dab2f9fdbbfaddc31a909735c1e6"
						},
						"ciphertext" : "5318b4d5bcd28de64ee5559e671353e16f075ecae9f99c7a79a38af5f869aa46",
						"kdf" : "pbkdf2",
						"kdfparams" : {
							"c" : 262144,
							"dklen" : 32,
							"prf" : "hmac-sha256",
							"salt" : "ae3cd4e7013836a3df6bd7241b12db061dbe2c6785853cce422d148a624ce0bd"
						},
						"mac" : "517ead924a9d0dc3124507e3393d175ce3ff7c1e96529c6c555ce9e51205e9b2"
					},
					"id" : "3198bc9c-6672-5ab3-d995-4942343ae5b6",
					"version" : 3
				}
			"#).unwrap();

		match KeyFileContent::from_json(&json) {
			Ok(key_file) => {
				assert_eq!(KeyFileVersion::V3(3), key_file.version)
			},
			Err(e) => panic!("Error parsing valid file: {:?}", e)
		}
	}

	#[test]
	fn can_read_scrypt_kdf() {
		let json = Json::from_str(
			r#"
				{
					"crypto" : {
						"cipher" : "aes-128-ctr",
						"cipherparams" : {
							"iv" : "83dbcc02d8ccb40e466191a123791e0e"
						},
						"ciphertext" : "d172bf743a674da9cdad04534d56926ef8358534d458fffccd4e6ad2fbde479c",
						"kdf" : "scrypt",
						"kdfparams" : {
							"dklen" : 32,
							"n" : 262144,
							"r" : 1,
							"p" : 8,
							"salt" : "ab0c7876052600dd703518d6fc3fe8984592145b591fc8fb5c6d43190334ba19"
						},
						"mac" : "2103ac29920d71da29f15d75b4a16dbe95cfd7ff8faea1056c33131d846e3097"
					},
					"id" : "3198bc9c-6672-5ab3-d995-4942343ae5b6",
					"version" : 3
				}
			"#).unwrap();

		match KeyFileContent::from_json(&json) {
			Ok(key_file) => {
				match key_file.crypto.kdf {
					KeyFileKdf::Scrypt(_) => {},
					_ => { panic!("expected kdf params of crypto to be of scrypt type" ); }
				}
			},
			Err(e) => panic!("Error parsing valid file: {:?}", e)
		}
	}

	#[test]
	fn can_read_scrypt_kdf_params() {
		let json = Json::from_str(
			r#"
				{
					"crypto" : {
						"cipher" : "aes-128-ctr",
						"cipherparams" : {
							"iv" : "83dbcc02d8ccb40e466191a123791e0e"
						},
						"ciphertext" : "d172bf743a674da9cdad04534d56926ef8358534d458fffccd4e6ad2fbde479c",
						"kdf" : "scrypt",
						"kdfparams" : {
							"dklen" : 32,
							"n" : 262144,
							"r" : 1,
							"p" : 8,
							"salt" : "ab0c7876052600dd703518d6fc3fe8984592145b591fc8fb5c6d43190334ba19"
						},
						"mac" : "2103ac29920d71da29f15d75b4a16dbe95cfd7ff8faea1056c33131d846e3097"
					},
					"id" : "3198bc9c-6672-5ab3-d995-4942343ae5b6",
					"version" : 3
				}
			"#).unwrap();

		match KeyFileContent::from_json(&json) {
			Ok(key_file) => {
				match key_file.crypto.kdf {
					KeyFileKdf::Scrypt(scrypt_params) => {
						assert_eq!(262144, scrypt_params.n);
						assert_eq!(1, scrypt_params.r);
						assert_eq!(8, scrypt_params.p);
					},
					_ => { panic!("expected kdf params of crypto to be of scrypt type" ); }
				}
			},
			Err(e) => panic!("Error parsing valid file: {:?}", e)
		}
	}

	#[test]
	fn can_return_error_no_id() {
		let json = Json::from_str(
			r#"
				{
					"crypto" : {
						"cipher" : "aes-128-ctr",
						"cipherparams" : {
							"iv" : "83dbcc02d8ccb40e466191a123791e0e"
						},
						"ciphertext" : "d172bf743a674da9cdad04534d56926ef8358534d458fffccd4e6ad2fbde479c",
						"kdf" : "scrypt",
						"kdfparams" : {
							"dklen" : 32,
							"n" : 262144,
							"r" : 1,
							"p" : 8,
							"salt" : "ab0c7876052600dd703518d6fc3fe8984592145b591fc8fb5c6d43190334ba19"
						},
						"mac" : "2103ac29920d71da29f15d75b4a16dbe95cfd7ff8faea1056c33131d846e3097"
					},
					"version" : 3
				}
			"#).unwrap();

		match KeyFileContent::from_json(&json) {
			Ok(_) => {
				panic!("Should be error of no crypto section, got ok");
			},
			Err(KeyFileParseError::InvalidIdentifier) => { },
			Err(other_error) => { panic!("should be error of no crypto section, got {:?}", other_error); }
		}
	}

	#[test]
	fn can_return_error_no_crypto() {
		let json = Json::from_str(
			r#"
				{
					"id" : "3198bc9c-6672-5ab3-d995-4942343ae5b6",
					"version" : 3
				}
			"#).unwrap();

		match KeyFileContent::from_json(&json) {
			Ok(_) => {
				panic!("Should be error of no identifier, got ok");
			},
			Err(KeyFileParseError::NoCryptoSection) => { },
			Err(other_error) => { panic!("should be error of no identifier, got {:?}", other_error); }
		}
	}

	#[test]
	fn can_return_error_unsupported_version() {
		let json = Json::from_str(
			r#"
				{
					"crypto" : {
						"cipher" : "aes-128-ctr",
						"cipherparams" : {
							"iv" : "83dbcc02d8ccb40e466191a123791e0e"
						},
						"ciphertext" : "d172bf743a674da9cdad04534d56926ef8358534d458fffccd4e6ad2fbde479c",
						"kdf" : "scrypt",
						"kdfparams" : {
							"dklen" : 32,
							"n" : 262144,
							"r" : 1,
							"p" : 8,
							"salt" : "ab0c7876052600dd703518d6fc3fe8984592145b591fc8fb5c6d43190334ba19"
						},
						"mac" : "2103ac29920d71da29f15d75b4a16dbe95cfd7ff8faea1056c33131d846e3097"
					},
					"id" : "3198bc9c-6672-5ab3-d995-4942343ae5b6",
					"version" : 1
				}
			"#).unwrap();

		match KeyFileContent::from_json(&json) {
			Ok(_) => {
				panic!("should be error of unsupported version, got ok");
			},
			Err(KeyFileParseError::UnsupportedVersion(_)) => { },
			Err(other_error) => { panic!("should be error of unsupported version, got {:?}", other_error); }
		}
	}


	#[test]
	fn can_return_error_initial_vector() {
		let json = Json::from_str(
			r#"
				{
					"crypto" : {
						"cipher" : "aes-128-ctr",
						"cipherparams" : {
							"iv" : "83dbcc02d8ccb40e4______66191a123791e0e"
						},
						"ciphertext" : "d172bf743a674da9cdad04534d56926ef8358534d458fffccd4e6ad2fbde479c",
						"kdf" : "scrypt",
						"kdfparams" : {
							"dklen" : 32,
							"n" : 262144,
							"r" : 1,
							"p" : 8,
							"salt" : "ab0c7876052600dd703518d6fc3fe8984592145b591fc8fb5c6d43190334ba19"
						},
						"mac" : "2103ac29920d71da29f15d75b4a16dbe95cfd7ff8faea1056c33131d846e3097"
					},
					"id" : "3198bc9c-6672-5ab3-d995-4942343ae5b6",
					"version" : 3
				}
			"#).unwrap();

		match KeyFileContent::from_json(&json) {
			Ok(_) => {
				panic!("should be error of invalid initial vector, got ok");
			},
			Err(KeyFileParseError::Crypto(CryptoParseError::InvalidInitialVector(_))) => { },
			Err(other_error) => { panic!("should be error of invalid initial vector, got {:?}", other_error); }
		}
	}

	#[test]
	fn can_return_error_for_invalid_scrypt_kdf() {
		let json = Json::from_str(
			r#"
				{
					"crypto" : {
						"cipher" : "aes-128-ctr",
						"cipherparams" : {
							"iv" : "83dbcc02d8ccb40e466191a123791e0e"
						},
						"ciphertext" : "d172bf743a674da9cdad04534d56926ef8358534d458fffccd4e6ad2fbde479c",
						"kdf" : "scrypt",
						"kdfparams" : {
							"dklen2" : 32,
							"n5" : "xx",
							"r" : 1,
							"p" : 8,
							"salt" : "ab0c7876052600dd703518d6fc3fe8984592145b591fc8fb5c6d43190334ba19"
						},
						"mac" : "2103ac29920d71da29f15d75b4a16dbe95cfd7ff8faea1056c33131d846e3097"
					},
					"id" : "3198bc9c-6672-5ab3-d995-4942343ae5b6",
					"version" : 3
				}
			"#).unwrap();

		match KeyFileContent::from_json(&json) {
			Ok(_) => {
				panic!("Should be error of no identifier, got ok");
			},
			Err(KeyFileParseError::Crypto(CryptoParseError::Scrypt(_))) => { },
			Err(other_error) => { panic!("should be scrypt parse error, got {:?}", other_error); }
		}
	}

	#[test]
	fn can_serialize_scrypt_back() {
		let json = Json::from_str(
			r#"
				{
					"crypto" : {
						"cipher" : "aes-128-ctr",
						"cipherparams" : {
							"iv" : "83dbcc02d8ccb40e466191a123791e0e"
						},
						"ciphertext" : "d172bf743a674da9cdad04534d56926ef8358534d458fffccd4e6ad2fbde479c",
						"kdf" : "scrypt",
						"kdfparams" : {
							"dklen" : 32,
							"n" : 262144,
							"r" : 1,
							"p" : 8,
							"salt" : "ab0c7876052600dd703518d6fc3fe8984592145b591fc8fb5c6d43190334ba19"
						},
						"mac" : "2103ac29920d71da29f15d75b4a16dbe95cfd7ff8faea1056c33131d846e3097"
					},
					"id" : "3198bc9c-6672-5ab3-d995-4942343ae5b6",
					"version" : 3
				}
			"#).unwrap();

		let key = KeyFileContent::from_json(&json).unwrap();
		let serialized = key.to_json();

		assert!(serialized.as_object().is_some());
	}

	#[test]
	fn can_create_key_with_new_id() {
		let cipher_text: Bytes = FromHex::from_hex("a0f05555").unwrap();
		let key = KeyFileContent::new(KeyFileCrypto::new_pbkdf2(cipher_text, H128::zero(), H256::random(), H256::random(), 32, 32));
		assert!(!uuid_to_string(&key.id).is_empty());
	}

	#[test]
	fn can_load_json_from_itself() {
		let cipher_text: Bytes = FromHex::from_hex("aaaaaaaaaaaaaaaaaaaaaaaaaaa22222222222222222222222").unwrap();
		let key = KeyFileContent::new(KeyFileCrypto::new_pbkdf2(cipher_text, H128::zero(), H256::random(), H256::random(), 32, 32));
		let json = key.to_json();

		let loaded_key = KeyFileContent::from_json(&json).unwrap();

		assert_eq!(loaded_key.id, key.id);
	}

	#[test]
	fn can_parse_kdf_params_fail() {
		let json = Json::from_str(
			r#"
			{
				"dklen" : 32,
				"n" : 262144,
				"r" : 1,
				"p" : 8,
				"salt" : "ab0c7876052600dd703518d6fc3fe8984592145b591fc8fb5c6d43190334ba19"
			}
			"#).unwrap();
		{
			let mut invalid_json = json.as_object().unwrap().clone();
			invalid_json.remove("dklen");
			let kdf = KdfPbkdf2Params::from_json(&invalid_json);
			assert!(!kdf.is_ok());
		}
		{
			let mut invalid_json = json.as_object().unwrap().clone();
			invalid_json.remove("n");
			let kdf = KdfPbkdf2Params::from_json(&invalid_json);
			assert!(!kdf.is_ok());
		}
		{
			let mut invalid_json = json.as_object().unwrap().clone();
			invalid_json.remove("r");
			let kdf = KdfPbkdf2Params::from_json(&invalid_json);
			assert!(!kdf.is_ok());
		}
		{
			let mut invalid_json = json.as_object().unwrap().clone();
			invalid_json.remove("p");
			let kdf = KdfPbkdf2Params::from_json(&invalid_json);
			assert!(!kdf.is_ok());
		}
		{
			let mut invalid_json = json.as_object().unwrap().clone();
			invalid_json.remove("salt");
			let kdf = KdfPbkdf2Params::from_json(&invalid_json);
			assert!(!kdf.is_ok());
		}

	}

	#[test]
	fn can_parse_kdf_params_scrypt_fail() {
		let json = Json::from_str(
			r#"
			{
				"dklen" : 32,
				"r" : 1,
				"p" : 8,
				"salt" : "ab0c7876052600dd703518d6fc3fe8984592145b591fc8fb5c6d43190334ba19"
			}
			"#).unwrap();
		{
			let mut invalid_json = json.as_object().unwrap().clone();
			invalid_json.remove("dklen");
			let kdf = KdfPbkdf2Params::from_json(&invalid_json);
			assert!(!kdf.is_ok());
		}
		{
			let mut invalid_json = json.as_object().unwrap().clone();
			invalid_json.remove("r");
			let kdf = KdfPbkdf2Params::from_json(&invalid_json);
			assert!(!kdf.is_ok());
		}
		{
			let mut invalid_json = json.as_object().unwrap().clone();
			invalid_json.remove("p");
			let kdf = KdfPbkdf2Params::from_json(&invalid_json);
			assert!(!kdf.is_ok());
		}
		{
			let mut invalid_json = json.as_object().unwrap().clone();
			invalid_json.remove("salt");
			let kdf = KdfPbkdf2Params::from_json(&invalid_json);
			assert!(!kdf.is_ok());
		}
	}

	#[test]
	fn can_parse_crypto_fails() {
		let json = Json::from_str(
			r#"
			{
				"cipher" : "aes-128-ctr",
				"cipherparams" : {
					"iv" : "83dbcc02d8ccb40e466191a123791e0e"
				},
				"ciphertext" : "d172bf743a674da9cdad04534d56926ef8358534d458fffccd4e6ad2fbde479c",
				"kdf" : "scrypt",
				"kdfparams" : {
					"dklen" : 32,
					"n" : 262144,
					"r" : 1,
					"p" : 8,
					"salt" : "ab0c7876052600dd703518d6fc3fe8984592145b591fc8fb5c6d43190334ba19"
				},
				"mac" : "2103ac29920d71da29f15d75b4a16dbe95cfd7ff8faea1056c33131d846e3097"
			}"#).unwrap();

		{
			let mut invalid_json = json.as_object().unwrap().clone();
			invalid_json.insert("cipher".to_owned(), Json::String("unknown".to_owned()));
			let crypto = KeyFileCrypto::from_json(&Json::Object(invalid_json));
			assert!(!crypto.is_ok());
		}

		{
			let mut invalid_json = json.as_object().unwrap().clone();
			invalid_json.insert("kdfparams".to_owned(), Json::String("122".to_owned()));
			let crypto = KeyFileCrypto::from_json(&Json::Object(invalid_json));
			assert!(!crypto.is_ok());
		}

		{
			let mut invalid_json = json.as_object().unwrap().clone();
			invalid_json.insert("kdf".to_owned(), Json::String("15522".to_owned()));
			let crypto = KeyFileCrypto::from_json(&Json::Object(invalid_json));
			assert!(!crypto.is_ok());
		}

	}

}

#[cfg(test)]
mod directory_tests {
	use super::{KeyDirectory, new_uuid, uuid_to_string, KeyFileContent, KeyFileCrypto, MAX_CACHE_USAGE_TRACK};
	use common::*;
	use devtools::*;

	#[test]
	fn key_directory_locates_keys() {
		let temp_path = RandomTempPath::create_dir();
		let directory = KeyDirectory::new(temp_path.as_path());
		let uuid = new_uuid();

		let path = directory.key_path(&uuid);

		assert!(path.to_str().unwrap().contains(&uuid_to_string(&uuid)));
	}

	#[test]
	fn loads_key() {
		let cipher_text: Bytes = FromHex::from_hex("a0f05555").unwrap();
		let temp_path = RandomTempPath::create_dir();
		let mut directory = KeyDirectory::new(&temp_path.as_path());
		let uuid = directory.save(KeyFileContent::new(KeyFileCrypto::new_pbkdf2(cipher_text, H128::zero(), H256::random(), H256::random(), 32, 32))).unwrap();
		let path = directory.key_path(&uuid);

		let key = KeyDirectory::load_key(&path).unwrap();

		assert_eq!(key.id, uuid);
	}

	#[test]
	fn caches_keys() {
		let temp_path = RandomTempPath::create_dir();
		let mut directory = KeyDirectory::new(&temp_path.as_path());

		let cipher_text: Bytes = FromHex::from_hex("a0f05555").unwrap();
		let mut keys = Vec::new();
		for _ in 0..1000 {
			let key = KeyFileContent::new(KeyFileCrypto::new_pbkdf2(cipher_text.clone(), H128::zero(), H256::random(), H256::random(), 32, 32));
			keys.push(directory.save(key).unwrap());
		}

		for key_id in keys {
			directory.get(&key_id).unwrap();
		}

		assert_eq!(1000, directory.cache_size())

	}

	#[test]
	fn collects_garbage() {
		let temp_path = RandomTempPath::create_dir();
		let mut directory = KeyDirectory::new(&temp_path.as_path());

		let cipher_text: Bytes = FromHex::from_hex("a0f05555").unwrap();
		let mut keys = Vec::new();
		for _ in 0..1000 {
			let key = KeyFileContent::new(KeyFileCrypto::new_pbkdf2(cipher_text.clone(), H128::zero(), H256::random(), H256::random(), 32, 32));
			keys.push(directory.save(key).unwrap());
		}

		for key_id in keys {
			directory.get(&key_id).unwrap();
		}

		directory.collect_garbage();
		// since all keys are different, should be exactly MAX_CACHE_USAGE_TRACK
		assert_eq!(MAX_CACHE_USAGE_TRACK, directory.cache_size())
	}

	#[test]
	fn collects_garbage_on_empty() {
		let temp_path = RandomTempPath::create_dir();
		let mut directory = KeyDirectory::new(&temp_path.as_path());
		directory.collect_garbage();
		assert_eq!(0, directory.cache_size())
	}
}

#[cfg(test)]
mod specs {
	use super::*;
	use common::*;
	use devtools::*;

	#[test]
	fn can_initiate_key_directory() {
		let temp_path = RandomTempPath::create_dir();
		let directory = KeyDirectory::new(&temp_path.as_path());
		assert!(directory.path().len() > 0);
	}

	#[test]
	fn can_save_key() {
		let cipher_text: Bytes = FromHex::from_hex("a0f05555").unwrap();
		let temp_path = RandomTempPath::create_dir();
		let mut directory = KeyDirectory::new(&temp_path.as_path());

		let uuid = directory.save(KeyFileContent::new(KeyFileCrypto::new_pbkdf2(cipher_text, H128::zero(), H256::random(), H256::random(), 32, 32)));

		assert!(uuid.is_ok());
	}

	#[test]
	fn can_load_key() {
		let cipher_text: Bytes = FromHex::from_hex("a0f05555").unwrap();
		let temp_path = RandomTempPath::create_dir();
		let mut directory = KeyDirectory::new(&temp_path.as_path());
		let uuid = directory.save(KeyFileContent::new(KeyFileCrypto::new_pbkdf2(cipher_text.clone(), H128::zero(), H256::random(), H256::random(), 32, 32))).unwrap();

		let key = directory.get(&uuid).unwrap();

		assert_eq!(key.crypto.cipher_text, cipher_text);
	}

	#[test]
	fn can_store_10_keys() {
		let temp_path = RandomTempPath::create_dir();
		let mut directory = KeyDirectory::new(&temp_path.as_path());

		let cipher_text: Bytes = FromHex::from_hex("a0f05555").unwrap();
		let mut keys = Vec::new();
		for _ in 0..10 {
			let key = KeyFileContent::new(KeyFileCrypto::new_pbkdf2(cipher_text.clone(), H128::zero(), H256::random(), H256::random(), 32, 32));
			keys.push(directory.save(key).unwrap());
		}

		assert_eq!(10, keys.len())
	}

	#[test]
	fn can_list_keys() {
		let temp_path = RandomTempPath::create_dir();
		let mut directory = KeyDirectory::new(&temp_path.as_path());

		let cipher_text: Bytes = FromHex::from_hex("a0f05555").unwrap();
		let mut keys = Vec::new();
		for _ in 0..33 {
			let key = KeyFileContent::new(KeyFileCrypto::new_pbkdf2(cipher_text.clone(), H128::zero(), H256::random(), H256::random(), 32, 32));
			keys.push(directory.save(key).unwrap());
		}

		assert_eq!(33, directory.list().unwrap().len());
	}
}
