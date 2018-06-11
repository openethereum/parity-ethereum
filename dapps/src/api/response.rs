// Copyright 2015-2018 Parity Technologies (UK) Ltd.
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
use hyper::{self, mime, StatusCode};

use handlers::{ContentHandler, EchoHandler};

pub fn empty() -> hyper::Response {
	ContentHandler::ok("".into(), mime::TEXT_PLAIN).into()
}

pub fn as_json<T: Serialize>(status: StatusCode, val: &T) -> hyper::Response {
	let json = serde_json::to_string(val)
		.expect("serialization to string is infallible; qed");
	ContentHandler::new(status, json, mime::APPLICATION_JSON).into()
}

pub fn ping(req: hyper::Request) -> hyper::Response {
	EchoHandler::new(req).into()
}

pub fn not_found() -> hyper::Response {
	as_json(StatusCode::NotFound, &::api::types::ApiError {
		code: "404".into(),
		title: "Not Found".into(),
		detail: "Resource you requested has not been found.".into(),
	})
}
