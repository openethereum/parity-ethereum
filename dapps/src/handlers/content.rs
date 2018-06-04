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

//! Simple Content Handler

use hyper::{self, mime, header};
use hyper::StatusCode;

use parity_version::version;

use handlers::add_security_headers;

#[derive(Debug, Clone)]
pub struct ContentHandler {
	code: StatusCode,
	content: String,
	mimetype: mime::Mime,
}

impl ContentHandler {
	pub fn ok(content: String, mimetype: mime::Mime) -> Self {
		Self::new(StatusCode::Ok, content, mimetype)
	}

	pub fn html(code: StatusCode, content: String) -> Self {
		Self::new(code, content, mime::TEXT_HTML)
	}

	pub fn error(
		code: StatusCode,
		title: &str,
		message: &str,
		details: Option<&str>,
	) -> Self {
		Self::html(code, format!(
			include_str!("../error_tpl.html"),
			title=title,
			message=message,
			details=details.unwrap_or_else(|| ""),
			version=version(),
		))
	}

	pub fn new(
		code: StatusCode,
		content: String,
		mimetype: mime::Mime,
	) -> Self {
		ContentHandler {
			code,
			content,
			mimetype,
		}
	}
}

impl Into<hyper::Response> for ContentHandler {
	fn into(self) -> hyper::Response {
		let mut res = hyper::Response::new()
			.with_status(self.code)
			.with_header(header::ContentType(self.mimetype))
			.with_body(self.content);
		add_security_headers(&mut res.headers_mut(), false);
		res
	}
}
