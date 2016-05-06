use serde::{Deserialize, Deserializer, Error as SerdeError};
use serde_json::Value;
use serde_json::value;
use super::{Function, Event};

#[derive(Debug, PartialEq)]
pub enum Operation {
	Constructor(Function),
	Function(Function),
	Event(Event),
}

impl Deserialize for Operation {
	fn deserialize<D>(deserializer: &mut D) -> Result<Self, D::Error>
		where D: Deserializer {
		let v = try!(Value::deserialize(deserializer));
		if let Value::Object(ref map) = v.clone() {
			if let Some(&Value::String(ref s) = map.get("type") {
				let result = match s.as_ref() {
					"constructor" => Deserialize::deserialize(&mut value::Deserializer::new(v)).map(Operation::Constructor),
					"function" => Deserialize::deserialize(&mut value::Deserializer::new(v)).map(Operation::Function),
					"event" => Deserialize::deserialize(&mut value::Deserializer::new(v)).map(Operation::Event),
					_ => Err(SerdeError::custom("Invalid operation type.")),
				};

				return result.map_err(|e| D::Error::custom(format!("{:?}", e).as_ref()));
			}
		}
		Err(D::Error::custom("Invalid operation"))
	}
}

impl Operation {
	pub fn constructor(&self) -> Option<&Function> {
		match *self {
			Operation::Constructor(ref f) => Some(f),
			_ => None
		}
	}

	pub fn function(&self) -> Option<&Function> {
		match *self {
			Operation::Function(ref f) => Some(f),
			_ => None
		}
	}

	pub fn event(&self) -> Option<&Event> {
		match *self {
			Operation::Event(ref e) => Some(e),
			_ => None
		}
	}
}

#[cfg(test)]
mod tests {
	use serde_json;
	use super::Operation;
	use spec::{ParamType, Function, Param};

	#[test]
	fn deserialize_operation() {
		let s = r#"{
			"type":"function",
			"inputs": [{
				"name":"a",
				"type":"address"
			}],
			"name":"foo",
			"outputs": []
		}"#;

		let deserialized: Operation = serde_json::from_str(s).unwrap();

		assert_eq!(deserialized, Operation::Function(Function {
			name: "foo".to_owned(),
			inputs: vec![
				Param {
					name: "a".to_owned(),
					kind: ParamType::Address,
				}
			],
			outputs: vec![]
		}));
	}
}
