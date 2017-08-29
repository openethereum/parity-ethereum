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
use fetch::{self, Fetch};
use parity_reactor::Remote;

use base32;
use hyper::{self, server, net, Next, Encoder, Decoder};
use hyper::status::StatusCode;

use apps;
use endpoint::{Endpoint, Handler, EndpointPath};
use handlers::{
	ContentFetcherHandler, ContentHandler, ContentValidator, ValidatorResponse,
	StreamingHandler, extract_url,
};
use url::Url;
use {Embeddable, WebProxyTokens};

pub struct Web<F> {
	embeddable_on: Embeddable,
	web_proxy_tokens: Arc<WebProxyTokens>,
	remote: Remote,
	fetch: F,
}

impl<F: Fetch> Web<F> {
	pub fn boxed(
		embeddable_on: Embeddable,
		web_proxy_tokens: Arc<WebProxyTokens>,
		remote: Remote,
		fetch: F,
	) -> Box<Endpoint> {
		Box::new(Web {
			embeddable_on,
			web_proxy_tokens,
			remote,
			fetch,
		})
	}
}

impl<F: Fetch> Endpoint for Web<F> {
	fn to_async_handler(&self, path: EndpointPath, control: hyper::Control) -> Box<Handler> {
		Box::new(WebHandler {
			control: control,
			state: State::Initial,
			path: path,
			remote: self.remote.clone(),
			fetch: self.fetch.clone(),
			web_proxy_tokens: self.web_proxy_tokens.clone(),
			embeddable_on: self.embeddable_on.clone(),
		})
	}
}

struct WebInstaller {
	embeddable_on: Embeddable,
	referer: String,
}

impl ContentValidator for WebInstaller {
	type Error = String;

	fn validate_and_install(&self, response: fetch::Response) -> Result<ValidatorResponse, String> {
		let status = StatusCode::from_u16(response.status().to_u16());
		let is_html = response.is_html();
		let mime = response.content_type().unwrap_or(mime!(Text/Html));
		let mut handler = StreamingHandler::new(
			response,
			status,
			mime,
			self.embeddable_on.clone(),
		);
		if is_html {
			handler.set_initial_content(&format!(
				r#"<script src="/{}/inject.js"></script><script>history.replaceState({{}}, "", "/?{}{}/{}")</script>"#,
				apps::UTILS_PATH,
				apps::URL_REFERER,
				apps::WEB_PATH,
				&self.referer,
			));
		}
		Ok(ValidatorResponse::Streaming(handler))
	}
}

enum State<F: Fetch> {
	Initial,
	Error(ContentHandler),
	Fetching(ContentFetcherHandler<WebInstaller, F>),
}

struct WebHandler<F: Fetch> {
	control: hyper::Control,
	state: State<F>,
	path: EndpointPath,
	remote: Remote,
	fetch: F,
	web_proxy_tokens: Arc<WebProxyTokens>,
	embeddable_on: Embeddable,
}

impl<F: Fetch> WebHandler<F> {
	fn extract_target_url(&self, url: Option<Url>) -> Result<String, State<F>> {
		let token_and_url = self.path.app_params.get(0)
			.map(|encoded| encoded.replace('.', ""))
			.and_then(|encoded| base32::decode(base32::Alphabet::Crockford, &encoded.to_uppercase()))
			.and_then(|data| String::from_utf8(data).ok())
			.ok_or_else(|| State::Error(ContentHandler::error(
				StatusCode::BadRequest,
				"Invalid parameter",
				"Couldn't parse given parameter:",
				self.path.app_params.get(0).map(String::as_str),
				self.embeddable_on.clone()
			)))?;

		let mut token_it = token_and_url.split('+');
		let token = token_it.next();
		let target_url = token_it.next();

		// Check if token supplied in URL is correct.
		let domain = match token.and_then(|token| self.web_proxy_tokens.domain(token)) {
			Some(domain) => domain,
			_ => {
				return Err(State::Error(ContentHandler::error(
					StatusCode::BadRequest, "Invalid Access Token", "Invalid or old web proxy access token supplied.", Some("Try refreshing the page."), self.embeddable_on.clone()
				)));
			}
		};

		// Validate protocol
		let mut target_url = match target_url {
			Some(url) if url.starts_with("http://") || url.starts_with("https://") => url.to_owned(),
			_ => {
				return Err(State::Error(ContentHandler::error(
					StatusCode::BadRequest, "Invalid Protocol", "Invalid protocol used.", None, self.embeddable_on.clone()
				)));
			}
		};

		if !target_url.starts_with(&*domain) {
			return Err(State::Error(ContentHandler::error(
				StatusCode::BadRequest, "Invalid Domain", "Dapp attempted to access invalid domain.", Some(&target_url), self.embeddable_on.clone(),
			)));
		}

		if !target_url.ends_with("/") {
			target_url = format!("{}/", target_url);
		}

		// TODO [ToDr] Should just use `path.app_params`
		let (path, query) = match (&url, self.path.using_dapps_domains) {
			(&Some(ref url), true) => (&url.path[..], &url.query),
			(&Some(ref url), false) => (&url.path[2..], &url.query),
			_ => {
				return Err(State::Error(ContentHandler::error(
					StatusCode::BadRequest, "Invalid URL", "Couldn't parse URL", None, self.embeddable_on.clone()
				)));
			}
		};

		let query = match *query {
			Some(ref query) => format!("?{}", query),
			None => "".into(),
		};

		Ok(format!("{}{}{}", target_url, path.join("/"), query))
	}
}

impl<F: Fetch> server::Handler<net::HttpStream> for WebHandler<F> {
	fn on_request(&mut self, request: server::Request<net::HttpStream>) -> Next {
		let url = extract_url(&request);
		// First extract the URL (reject invalid URLs)
		let target_url = match self.extract_target_url(url) {
			Ok(url) => url,
			Err(error) => {
				self.state = error;
				return Next::write();
			}
		};

		let mut handler = ContentFetcherHandler::new(
			target_url,
			self.path.clone(),
			self.control.clone(),
			WebInstaller {
				embeddable_on: self.embeddable_on.clone(),
				referer: self.path.app_params.get(0)
					.expect("`target_url` is valid; app_params is not empty;qed")
					.to_owned(),
			},
			self.embeddable_on.clone(),
			self.remote.clone(),
			self.fetch.clone(),
		);
		let res = handler.on_request(request);
		self.state = State::Fetching(handler);

		res
	}

	fn on_request_readable(&mut self, decoder: &mut Decoder<net::HttpStream>) -> Next {
		match self.state {
			State::Initial => Next::end(),
			State::Error(ref mut handler) => handler.on_request_readable(decoder),
			State::Fetching(ref mut handler) => handler.on_request_readable(decoder),
		}
	}

	fn on_response(&mut self, res: &mut server::Response) -> Next {
		match self.state {
			State::Initial => Next::end(),
			State::Error(ref mut handler) => handler.on_response(res),
			State::Fetching(ref mut handler) => handler.on_response(res),
		}
	}

	fn on_response_writable(&mut self, encoder: &mut Encoder<net::HttpStream>) -> Next {
		match self.state {
			State::Initial => Next::end(),
			State::Error(ref mut handler) => handler.on_response_writable(encoder),
			State::Fetching(ref mut handler) => handler.on_response_writable(encoder),
		}
	}
}
