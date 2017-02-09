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

//! Serving ProxyPac file

use endpoint::{Endpoint, Handler, EndpointPath};
use handlers::ContentHandler;
use apps::{HOME_PAGE, DAPPS_DOMAIN};
use address;

pub struct ProxyPac {
	signer_address: Option<(String, u16)>,
}

impl ProxyPac {
	pub fn boxed(signer_address: Option<(String, u16)>) -> Box<Endpoint> {
		Box::new(ProxyPac {
			signer_address: signer_address
		})
	}
}

impl Endpoint for ProxyPac {
	fn to_handler(&self, path: EndpointPath) -> Box<Handler> {
		let signer = self.signer_address
			.as_ref()
			.map(address)
			.unwrap_or_else(|| format!("{}:{}", path.host, path.port));

		let content = format!(
r#"
function FindProxyForURL(url, host) {{
	if (shExpMatch(host, "{0}{1}"))
	{{
		return "PROXY {4}";
	}}

	if (shExpMatch(host, "*{1}"))
	{{
		return "PROXY {2}:{3}";
	}}

	return "DIRECT";
}}
"#,
		HOME_PAGE, DAPPS_DOMAIN, path.host, path.port, signer);

		Box::new(ContentHandler::ok(content, mime!(Application/Javascript)))
	}
}


