//! Contract interface.

use serde_json;
use super::{Operation, Function, Event};

/// Contract interface.
#[derive(Debug, PartialEq, Deserialize)]
pub struct Interface(Vec<Operation>);

impl Interface {
	/// Loads interface from json.
	pub fn load(bytes: &[u8]) -> Result<Self, serde_json::Error> {
		serde_json::from_slice(bytes)
	}

	/// Returns contract constructor specification.
	pub fn constructor(&self) -> Option<Function> {
		self.0.iter()
			.filter_map(Operation::constructor)
			.next()
			.cloned()
	}

	/// Returns specification of contract function.
	pub fn function(&self, name: String) -> Option<Function> { 
		self.0.iter()
			.filter_map(Operation::function)
			.find(|f| f.name == name)
			.cloned()
	}

	/// Returns specification of contract event.
	pub fn event(&self, name: String) -> Option<Event> {
		self.0.iter()
			.filter_map(Operation::event)
			.find(|e| e.name == name)
			.cloned()
	}
}

#[cfg(test)]
mod tests {
	use serde_json;
	use super::Interface;
	use spec::{ParamType, Function, Param, Operation, Event, EventParam};

	#[test]
	fn deserialize_interface() {
		let s = r#"[{
			"type":"event",
			"inputs": [{
				"name":"a",
				"type":"uint256",
				"indexed":true
			},{
				"name":"b",
				"type":"bytes32",
				"indexed":false
			}],
			"name":"Event2",
			"anonymous": false
		}, {
			"type":"function",
			"inputs": [{
				"name":"a",
				"type":"uint256"
			}],
			"name":"foo",
			"outputs": []
		}]"#;
		
		let deserialized: Interface = serde_json::from_str(s).unwrap();

		assert_eq!(deserialized, Interface(vec![
			Operation::Event(Event {
				name: "Event2".to_owned(),
				inputs: vec![
					EventParam {
						name: "a".to_owned(),
						kind: ParamType::Uint(256),
						indexed: true,
					},
					EventParam {
						name: "b".to_owned(),
						kind: ParamType::FixedBytes(32),
						indexed: false,
					}
				],
				anonymous: false,
			}),
			Operation::Function(Function {
				name: "foo".to_owned(),
				inputs: vec![
					Param {
						name: "a".to_owned(),
						kind: ParamType::Uint(256),
					}
				],
				outputs: vec![]
			})
		]));
	}
}
