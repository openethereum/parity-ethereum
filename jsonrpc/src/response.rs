//! jsonrpc response
use serde::{Serialize, Serializer};
use super::{Id, Value, Error};

#[derive(Debug, PartialEq, Serialize)]
pub struct Success {
	pub jsonrpc: String,
	pub result: Value,
	pub id: Id
}

#[derive(Debug, PartialEq, Serialize)]
pub struct Failure {
	pub jsonrpc: String,
	pub error: Error,
	pub id: Id
}

#[derive(Debug, PartialEq)]
pub enum ResponseBatchSlice {
	Success(Success),
	Failure(Failure)
}

impl Serialize for ResponseBatchSlice {
	fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error> 
	where S: Serializer {
		match self {
			&ResponseBatchSlice::Success(ref s) => s.serialize(serializer),
			&ResponseBatchSlice::Failure(ref f) => f.serialize(serializer)
		}
	}
}

#[derive(Debug, PartialEq)]
pub enum Response {
	Success(Success),
	Failure(Failure),
	Batch(Vec<ResponseBatchSlice>)
}

impl Serialize for Response {
	fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error> 
	where S: Serializer {
		match self {
			&Response::Success(ref s) => s.serialize(serializer),
			&Response::Failure(ref f) => f.serialize(serializer),
			&Response::Batch(ref b) => b.serialize(serializer)
		}
	}
}

