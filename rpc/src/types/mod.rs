use serde::{Serialize, Deserialize, de};
use serde_json::value::{Value, Serializer, Deserializer};

mod block;

pub fn to_value<S>(s: &S) -> Value where S: Serialize {
	let mut serializer = Serializer::new();
	// should never panic!
	s.serialize(&mut serializer).unwrap();
	serializer.unwrap()
}

pub fn from_value<D>(value: Value) -> Result<D, <Deserializer as de::Deserializer>::Error> where D: Deserialize {
	let mut deserialier = Deserializer::new(value);
	Deserialize::deserialize(&mut deserialier)
}

pub use self::block::Block;
