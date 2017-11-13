// Copyright 2015-2017 Parity Technologies (UK) Ltd.
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

//! Serving web-based content (proxying)

use std::sync::Arc;

use base32;
use fetch::{self, Fetch};
use hyper::{mime, StatusCode};

use apps;
use endpoint::{Endpoint, EndpointPath, Request, Response};
use futures::future;
use handlers::{
	ContentFetcherHandler, ContentHandler, ContentValidator, ValidatorResponse,
	StreamingHandler,
};
use {Embeddable, WebProxyTokens};

pub struct Web<F> {
	embeddable_on: Embeddable,
	web_proxy_tokens: Arc<WebProxyTokens>,
	fetch: F,
}

impl<F: Fetch> Web<F> {
	pub fn boxed(
		embeddable_on: Embeddable,
		web_proxy_tokens: Arc<WebProxyTokens>,
		fetch: F,
	) -> Box<Endpoint> {
		Box::new(Web {
			embeddable_on,
			web_proxy_tokens,
			fetch,
		})
	}

	fn extract_target_url(&self, path: &EndpointPath) -> Result<String, ContentHandler> {
		let token_and_url = path.app_params.get(0)
			.map(|encoded| encoded.replace('.', ""))
			.and_then(|encoded| base32::decode(base32::Alphabet::Crockford, &encoded.to_uppercase()))
			.and_then(|data| String::from_utf8(data).ok())
			.ok_or_else(|| ContentHandler::error(
				StatusCode::BadRequest,
				"Invalid parameter",
				"Couldn't parse given parameter:",
				path.app_params.get(0).map(String::as_str),
				self.embeddable_on.clone()
			))?;

		let mut token_it = token_and_url.split('+');
		let token = token_it.next();
		let target_url = token_it.next();

		// Check if token supplied in URL is correct.
		let domain = match token.and_then(|token| self.web_proxy_tokens.domain(token)) {
			Some(domain) => domain,
			_ => {
				return Err(ContentHandler::error(
					StatusCode::BadRequest, "Invalid Access Token", "Invalid or old web proxy access token supplied.", Some("Try refreshing the page."), self.embeddable_on.clone()
				));
			}
		};

		// Validate protocol
		let mut target_url = match target_url {
			Some(url) if url.starts_with("http://") || url.starts_with("https://") => url.to_owned(),
			_ => {
				return Err(ContentHandler::error(
					StatusCode::BadRequest, "Invalid Protocol", "Invalid protocol used.", None, self.embeddable_on.clone()
				));
			}
		};

		if !target_url.starts_with(&*domain) {
			return Err(ContentHandler::error(
				StatusCode::BadRequest, "Invalid Domain", "Dapp attempted to access invalid domain.", Some(&target_url), self.embeddable_on.clone(),
			));
		}

		if !target_url.ends_with("/") {
			target_url = format!("{}/", target_url);
		}

		// Skip the token
		let query = path.query.as_ref().map_or_else(String::new, |query| format!("?{}", query));
		let path = path.app_params[1..].join("/");

		Ok(format!("{}{}{}", target_url, path, query))
	}
}

impl<F: Fetch> Endpoint for Web<F> {
	fn respond(&self, path: EndpointPath, req: Request) -> Response {
		// First extract the URL (reject invalid URLs)
		let target_url = match self.extract_target_url(&path) {
			Ok(url) => url,
			Err(response) => {
				return Box::new(future::ok(response.into()));
			}
		};

		let token = path.app_params.get(0)
			.expect("`target_url` is valid; app_params is not empty;qed")
			.to_owned();

		Box::new(ContentFetcherHandler::new(
			req.method(),
			&target_url,
			path,
			WebInstaller {
				embeddable_on: self.embeddable_on.clone(),
				token,
			},
			self.embeddable_on.clone(),
			self.fetch.clone(),
		))
	}
}

struct WebInstaller {
	embeddable_on: Embeddable,
	token: String,
}

impl ContentValidator for WebInstaller {
	type Error = String;

	fn validate_and_install(self, response: fetch::Response) -> Result<ValidatorResponse, String> {
		let status = response.status();
		let is_html = response.is_html();
		let mime = response.content_type().unwrap_or(mime::TEXT_HTML);
		let mut handler = StreamingHandler::new(
			response,
			status,
			mime,
			self.embeddable_on,
		);
		if is_html {
			handler.set_initial_content(&format!(
				r#"<script src="/{}/inject.js"></script><script>history.replaceState({{}}, "", "/?{}{}/{}")</script>"#,
				apps::UTILS_PATH,
				apps::URL_REFERER,
				apps::WEB_PATH,
				&self.token,
			));
		}
		Ok(ValidatorResponse::Streaming(handler))
	}
}

