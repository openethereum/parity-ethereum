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

use serde::Serialize;
use serde_json;
use endpoint::Handler;
use handlers::{ContentHandler, EchoHandler};

pub fn as_json<T: Serialize>(val: &T) -> Box<Handler> {
	let json = serde_json::to_string(val)
		.expect("serialization to string is infallible; qed");
	Box::new(ContentHandler::ok(json, mime!(Application/Json)))
}

pub fn as_json_error<T: Serialize>(val: &T) -> Box<Handler> {
	let json = serde_json::to_string(val)
		.expect("serialization to string is infallible; qed");
	Box::new(ContentHandler::not_found(json, mime!(Application/Json)))
}

pub fn ping_response(local_domain: &str) -> Box<Handler> {
	Box::new(EchoHandler::cors(vec![
		format!("http://{}", local_domain),
		// Allow CORS calls also for localhost
		format!("http://{}", local_domain.replace("127.0.0.1", "localhost")),
	]))
}
