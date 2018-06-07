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

use std::path::PathBuf;
use std::sync::Arc;

use endpoint::Endpoints;
use futures_cpupool::CpuPool;
use proxypac::ProxyPac;
use web::Web;
use fetch::Fetch;
use WebProxyTokens;

mod app;
mod cache;
pub mod fs;
pub mod fetcher;
pub mod manifest;

pub use self::app::App;

pub const HOME_PAGE: &'static str = "home";
pub const RPC_PATH: &'static str = "rpc";
pub const API_PATH: &'static str = "api";
pub const WEB_PATH: &'static str = "web";
pub const URL_REFERER: &'static str = "__referer=";

pub fn all_endpoints<F: Fetch>(
	dapps_path: PathBuf,
	extra_dapps: Vec<PathBuf>,
	dapps_domain: &str,
	web_proxy_tokens: Arc<WebProxyTokens>,
	fetch: F,
	pool: CpuPool,
) -> (Vec<String>, Endpoints) {
	// fetch fs dapps at first to avoid overwriting builtins
	let mut pages = fs::local_endpoints(dapps_path.clone(), pool.clone());
	let local_endpoints: Vec<String> = pages.keys().cloned().collect();
	for path in extra_dapps {
		if let Some((id, endpoint)) = fs::local_endpoint(path.clone(), pool.clone()) {
			pages.insert(id, endpoint);
		} else {
			warn!(target: "dapps", "Ignoring invalid dapp at {}", path.display());
		}
	}

	pages.insert(
		"proxy".into(),
		ProxyPac::boxed(dapps_domain.to_owned())
	);
	pages.insert(
		WEB_PATH.into(),
		Web::boxed(web_proxy_tokens.clone(), fetch.clone(), pool.clone())
	);

	(local_endpoints, pages)
}
