use serde::{Serialize, Serializer, Deserialize, Deserializer, Error as SerdeError};
use serde::de::Visitor;
use super::{Error, H128};

#[derive(Debug, PartialEq)]
pub enum CipherSer {
	Aes128Ctr,
}

impl Serialize for CipherSer {
	fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error> 
	where S: Serializer {
		match *self {
			CipherSer::Aes128Ctr => serializer.serialize_str("aes-128-ctr"),
		}
	}
}

impl Deserialize for CipherSer {
	fn deserialize<D>(deserializer: &mut D) -> Result<Self, D::Error>
	where D: Deserializer {
		deserializer.deserialize(CipherSerVisitor)
	}
}

struct CipherSerVisitor;

impl Visitor for CipherSerVisitor {
	type Value = CipherSer;

	fn visit_str<E>(&mut self, value: &str) -> Result<Self::Value, E> where E: SerdeError {
		match value {
			"aes-128-ctr" => Ok(CipherSer::Aes128Ctr),
			_ => Err(SerdeError::custom(Error::UnsupportedCipher))
		}
	}

	fn visit_string<E>(&mut self, value: String) -> Result<Self::Value, E> where E: SerdeError {
		self.visit_str(value.as_ref())
	}
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Aes128Ctr {
	pub iv: H128,
}

#[derive(Debug, PartialEq)]
pub enum CipherSerParams {
	Aes128Ctr(Aes128Ctr),
}

impl Serialize for CipherSerParams {
	fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error> 
	where S: Serializer {
		match *self {
			CipherSerParams::Aes128Ctr(ref params) => params.serialize(serializer),
		}
	}
}

impl Deserialize for CipherSerParams {
	fn deserialize<D>(deserializer: &mut D) -> Result<Self, D::Error>
	where D: Deserializer {
		Aes128Ctr::deserialize(deserializer)
			.map(CipherSerParams::Aes128Ctr)
			.map_err(|_| Error::InvalidCipherParams)
			.map_err(SerdeError::custom)
	}
}

#[derive(Debug, PartialEq)]
pub enum Cipher {
	Aes128Ctr(Aes128Ctr),
}
