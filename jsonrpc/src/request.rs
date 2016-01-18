//! jsonrpc request
use serde::de::{Deserialize, Deserializer, Visitor, SeqVisitor, MapVisitor};
use serde::de::impls::{VecVisitor};
use super::{Id, Params, Version};

#[derive(Debug, PartialEq, Deserialize)]
pub struct MethodCall {
	/// A String specifying the version of the JSON-RPC protocol. 
	/// MUST be exactly "2.0".
	pub jsonrpc: Version,
	/// A String containing the name of the method to be invoked.
	pub method: String,
	/// A Structured value that holds the parameter values to be used 
	/// during the invocation of the method. This member MAY be omitted.
	pub params: Option<Params>,
	/// An identifier established by the Client that MUST contain a String,
	/// Number, or NULL value if included. If it is not included it is assumed 
	/// to be a notification. 
	pub id: Id,
}

#[derive(Debug, PartialEq, Deserialize)]
pub struct Notification {
	/// A String specifying the version of the JSON-RPC protocol. 
	/// MUST be exactly "2.0".
	pub jsonrpc: Version,
	/// A String containing the name of the method to be invoked.
	pub method: String,
	/// A Structured value that holds the parameter values to be used 
	/// during the invocation of the method. This member MAY be omitted.
	pub params: Option<Params>
}

#[derive(Debug, PartialEq)]
pub enum RequestBatchSlice {
	MethodCall(MethodCall),
	Notification(Notification)
}

impl Deserialize for RequestBatchSlice {
	fn deserialize<D>(deserializer: &mut D) -> Result<RequestBatchSlice, D::Error>
	where D: Deserializer {
		ok!(MethodCall::deserialize(deserializer).map(RequestBatchSlice::MethodCall));
		Notification::deserialize(deserializer).map(RequestBatchSlice::Notification)
	}
}

#[derive(Debug, PartialEq)]
pub enum Request {
	MethodCall(MethodCall),
	Notification(Notification),
	Batch(Vec<RequestBatchSlice>)
}

impl Deserialize for Request {
	fn deserialize<D>(deserializer: &mut D) -> Result<Request, D::Error>
	where D: Deserializer {
		ok!(MethodCall::deserialize(deserializer).map(Request::MethodCall));
		ok!(Notification::deserialize(deserializer).map(Request::Notification));
		deserializer.visit(BatchVisitor)
	}
}

struct BatchVisitor;

impl Visitor for BatchVisitor {
	type Value = Request;

	fn visit_seq<V>(&mut self, visitor: V) -> Result<Self::Value, V::Error> 
	where V: SeqVisitor {
		VecVisitor::new().visit_seq(visitor).map(Request::Batch)
	}
}

#[test]
fn notification_deserialize() {
	use serde_json;
	use serde_json::Value;

	let s = r#"{"jsonrpc": "2.0", "method": "update", "params": [1,2]}"#;
	let deserialized: Notification = serde_json::from_str(s).unwrap();

	assert_eq!(deserialized, Notification {
		jsonrpc: Version::V2,
		method: "update".to_string(),
		params: Some(Params::Array(vec![Value::U64(1), Value::U64(2)]))
	});

	let s = r#"{"jsonrpc": "2.0", "method": "foobar"}"#;
	let deserialized: Notification = serde_json::from_str(s).unwrap();

	assert_eq!(deserialized, Notification {
		jsonrpc: Version::V2,
		method: "foobar".to_string(),
		params: None
	});

	let s = r#"{"jsonrpc": "2.0", "method": "update", "params": [1,2], "id": 1}"#;
	let deserialized: Result<Notification, _> = serde_json::from_str(s);
	assert!(deserialized.is_err())
}
