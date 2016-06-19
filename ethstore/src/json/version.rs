use serde::{Serialize, Serializer, Deserialize, Deserializer, Error as SerdeError};
use serde::de::Visitor;
use super::Error;

#[derive(Debug, PartialEq)]
pub enum Version {
	V3,
}

impl Serialize for Version {
	fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error> 
	where S: Serializer {
		match *self {
			Version::V3 => serializer.serialize_u64(3)
		}
	}
}

impl Deserialize for Version {
	fn deserialize<D>(deserializer: &mut D) -> Result<Version, D::Error>
	where D: Deserializer {
		deserializer.deserialize(VersionVisitor)
	}
}

struct VersionVisitor;

impl Visitor for VersionVisitor {
	type Value = Version;

	fn visit_u64<E>(&mut self, value: u64) -> Result<Self::Value, E> where E: SerdeError {
		match value {
			3 => Ok(Version::V3),
			_ => Err(SerdeError::custom(Error::UnsupportedVersion))
		}
	}
}

