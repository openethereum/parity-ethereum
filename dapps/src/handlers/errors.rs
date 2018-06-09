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

//! Handler errors.

use handlers::{ContentHandler, FETCH_TIMEOUT};
use hyper::StatusCode;
use std::fmt;

pub fn streaming() -> ContentHandler {
	ContentHandler::error(
		StatusCode::BadGateway,
		"Streaming Error",
		"This content is being streamed in other place.",
		None,
	)
}

pub fn download_error<E: fmt::Debug>(e: E) -> ContentHandler {
	ContentHandler::error(
		StatusCode::BadGateway,
		"Download Error",
		"There was an error when fetching the content.",
		Some(&format!("{:?}", e)),
	)
}

pub fn invalid_content<E: fmt::Debug>(e: E) -> ContentHandler {
	ContentHandler::error(
		StatusCode::BadGateway,
		"Invalid Dapp",
		"Downloaded bundle does not contain a valid content.",
		Some(&format!("{:?}", e)),
	)
}

pub fn timeout_error() -> ContentHandler {
	ContentHandler::error(
		StatusCode::GatewayTimeout,
		"Download Timeout",
		&format!("Could not fetch content within {} seconds.", FETCH_TIMEOUT.as_secs()),
		None,
	)
}

pub fn method_not_allowed() -> ContentHandler {
	ContentHandler::error(
		StatusCode::MethodNotAllowed,
		"Method Not Allowed",
		"Only <code>GET</code> requests are allowed.",
		None,
	)
}
