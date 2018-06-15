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

//! Hyper handlers implementations.

mod content;
mod echo;
mod fetch;
mod reader;
mod redirect;
mod streaming;
mod errors;

pub use self::content::ContentHandler;
pub use self::echo::EchoHandler;
pub use self::fetch::{ContentFetcherHandler, ContentValidator, FetchControl, ValidatorResponse};
pub use self::reader::Reader;
pub use self::redirect::Redirection;
pub use self::streaming::StreamingHandler;

use hyper::header;
use std::time::Duration;

const FETCH_TIMEOUT: Duration = Duration::from_secs(300);

/// Adds security-related headers to the Response.
pub fn add_security_headers(headers: &mut header::Headers, allow_js_eval: bool) {
	headers.set_raw("X-XSS-Protection", "1; mode=block");
	headers.set_raw("X-Content-Type-Options", "nosniff");
	headers.set_raw("X-Frame-Options", "SAMEORIGIN");

	// Content Security Policy headers
	headers.set_raw("Content-Security-Policy", String::new()
		// Restrict everything to the same origin by default.
		+ "default-src 'self';"
		// Allow connecting to WS servers and HTTP(S) servers.
		// We could be more restrictive and allow only RPC server URL.
		+ "connect-src http: https: ws: wss:;"
		// Allow framing any content from HTTP(S).
		// Again we could only allow embedding from RPC server URL.
		// (deprecated)
		+ "frame-src 'self' http: https:;"
		// Allow framing and web workers from HTTP(S).
		+ "child-src 'self' http: https:;"
		// We allow data: blob: and HTTP(s) images.
		// We could get rid of wildcarding HTTP and only allow RPC server URL.
		// (http required for local dapps icons)
		+ "img-src 'self' 'unsafe-inline' data: blob: http: https:;"
		// Allow style from data: blob: and HTTPS.
		+ "style-src 'self' 'unsafe-inline' data: blob: https:;"
		// Allow fonts from data: and HTTPS.
		+ "font-src 'self' data: https:;"
		// Disallow objects
		+ "object-src 'none';"
		// Allow scripts
		+ {
			let script_src = "";
			let eval = if allow_js_eval { " 'unsafe-eval'" } else { "" };

			&format!(
				"script-src 'self' {}{};",
				script_src,
				eval
			)
		}
		// Same restrictions as script-src with additional
		// blob: that is required for camera access (worker)
		+ "worker-src 'self' https: blob:;"
		// Run in sandbox mode (although it's not fully safe since we allow same-origin and script)
		+ "sandbox allow-same-origin allow-forms allow-modals allow-popups allow-presentation allow-scripts;"
		// Disallow submitting forms from any dapps
		+ "form-action 'none';"
		// Never allow mixed content
		+ "block-all-mixed-content;"
		// Specify if the site can be embedded.
		+ "frame-ancestors 'self';"
	);
}
