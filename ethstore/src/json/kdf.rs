use serde::{Serialize, Serializer, Deserialize, Deserializer, Error as SerdeError};
use serde::de::Visitor;
use serde_json::{Value, value};
use super::{Error, H256};

#[derive(Debug, PartialEq)]
pub enum KdfSer {
	Pbkdf2,
	Scrypt,
}

impl Serialize for KdfSer {
	fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
	where S: Serializer {
		match *self {
			KdfSer::Pbkdf2 => serializer.serialize_str("pbkdf2"),
			KdfSer::Scrypt => serializer.serialize_str("scrypt"),
		}
	}
}

impl Deserialize for KdfSer {
	fn deserialize<D>(deserializer: &mut D) -> Result<Self, D::Error>
	where D: Deserializer {
		deserializer.deserialize(KdfSerVisitor)
	}
}

struct KdfSerVisitor;

impl Visitor for KdfSerVisitor {
	type Value = KdfSer;

	fn visit_str<E>(&mut self, value: &str) -> Result<Self::Value, E> where E: SerdeError {
		match value {
			"pbkdf2" => Ok(KdfSer::Pbkdf2),
			"scrypt" => Ok(KdfSer::Scrypt),
			_ => Err(SerdeError::custom(Error::UnsupportedKdf))
		}
	}

	fn visit_string<E>(&mut self, value: String) -> Result<Self::Value, E> where E: SerdeError {
		self.visit_str(value.as_ref())
	}
}

#[derive(Debug, PartialEq)]
pub enum Prf {
	HmacSha256,
}

impl Serialize for Prf {
	fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
	where S: Serializer {
		match *self {
			Prf::HmacSha256 => serializer.serialize_str("hmac-sha256"),
		}
	}
}

impl Deserialize for Prf {
	fn deserialize<D>(deserializer: &mut D) -> Result<Self, D::Error>
	where D: Deserializer {
		deserializer.deserialize(PrfVisitor)
	}
}

struct PrfVisitor;

impl Visitor for PrfVisitor {
	type Value = Prf;

	fn visit_str<E>(&mut self, value: &str) -> Result<Self::Value, E> where E: SerdeError {
		match value {
			"hmac-sha256" => Ok(Prf::HmacSha256),
			_ => Err(SerdeError::custom(Error::InvalidPrf)),
		}
	}

	fn visit_string<E>(&mut self, value: String) -> Result<Self::Value, E> where E: SerdeError {
		self.visit_str(value.as_ref())
	}
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Pbkdf2 {
	pub c: u32,
	pub dklen: u32,
	pub prf: Prf,
	pub salt: H256,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Scrypt {
	pub dklen: u32,
	pub p: u32,
	pub n: u32,
	pub r: u32,
	pub salt: H256,
}

#[derive(Debug, PartialEq)]
pub enum KdfSerParams {
	Pbkdf2(Pbkdf2),
	Scrypt(Scrypt),
}

impl Serialize for KdfSerParams {
	fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
	where S: Serializer {
		match *self {
			KdfSerParams::Pbkdf2(ref params) => params.serialize(serializer),
			KdfSerParams::Scrypt(ref params) => params.serialize(serializer),
		}
	}
}

impl Deserialize for KdfSerParams {
	fn deserialize<D>(deserializer: &mut D) -> Result<Self, D::Error>
	where D: Deserializer {
		let v = try!(Value::deserialize(deserializer));

		Deserialize::deserialize(&mut value::Deserializer::new(v.clone())).map(KdfSerParams::Pbkdf2)
			.or_else(|_| Deserialize::deserialize(&mut value::Deserializer::new(v)).map(KdfSerParams::Scrypt))
			.map_err(|e| D::Error::custom(format!("{}", e)))
	}
}

#[derive(Debug, PartialEq)]
pub enum Kdf {
	Pbkdf2(Pbkdf2),
	Scrypt(Scrypt),
}
