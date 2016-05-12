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

//! Serving ProxyPac file

use std::net::SocketAddr;
use endpoint::{Endpoint, Handler, ContentHandler, HostInfo};
use DAPPS_DOMAIN;

pub struct ProxyPac {
	addr: SocketAddr,
}

impl ProxyPac {
	pub fn new(addr: &SocketAddr) -> Box<Endpoint> {
		Box::new(ProxyPac {
			addr: addr.clone(),
		})
	}
}

impl Endpoint for ProxyPac {
	fn to_handler(&self, _prefix: &str, host: Option<HostInfo>) -> Box<Handler> {
		let host = host.map_or_else(|| format!("{}", self.addr), |h| format!("{}:{}", h.host, h.port));
		let content = format!(
r#"
function FindProxyForURL(url, host) {{
	if (shExpMatch(host, "*{0}"))
	{{
		return "PROXY {1}";
	}}

	return "DIRECT";
}}
"#,
			DAPPS_DOMAIN, host);
		Box::new(ContentHandler::new(content, "application/javascript".to_owned()))
	}
}


