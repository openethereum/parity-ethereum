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

use endpoint::{Endpoints, Endpoint};
use futures_cpupool::CpuPool;
use page;
use proxypac::ProxyPac;
use web::Web;
use fetch::Fetch;
use {WebProxyTokens, ParentFrameSettings};

mod app;
mod cache;
mod ui;
pub mod fs;
pub mod fetcher;
pub mod manifest;

pub use self::app::App;

pub const HOME_PAGE: &'static str = "home";
pub const RPC_PATH: &'static str =  "rpc";
pub const API_PATH: &'static str =  "api";
pub const UTILS_PATH: &'static str =  "parity-utils";
pub const WEB_PATH: &'static str = "web";
pub const URL_REFERER: &'static str = "__referer=";

pub fn utils(pool: CpuPool) -> Box<Endpoint> {
	Box::new(page::builtin::Dapp::new(pool, ::parity_ui::App::default()))
}

pub fn ui(pool: CpuPool) -> Box<Endpoint> {
	Box::new(page::builtin::Dapp::with_fallback_to_index(pool, ::parity_ui::App::default()))
}

pub fn ui_deprecation(pool: CpuPool) -> Box<Endpoint> {
	Box::new(page::builtin::Dapp::with_fallback_to_index(pool, ::parity_ui_deprecation::App::default()))
}

pub fn ui_redirection(embeddable: Option<ParentFrameSettings>) -> Box<Endpoint> {
	Box::new(ui::Redirection::new(embeddable))
}

pub fn all_endpoints<F: Fetch>(
	dapps_path: PathBuf,
	extra_dapps: Vec<PathBuf>,
	dapps_domain: &str,
	embeddable: Option<ParentFrameSettings>,
	web_proxy_tokens: Arc<WebProxyTokens>,
	fetch: F,
	pool: CpuPool,
) -> (Vec<String>, Endpoints) {
	// fetch fs dapps at first to avoid overwriting builtins
	let mut pages = fs::local_endpoints(dapps_path.clone(), embeddable.clone(), pool.clone());
	let local_endpoints: Vec<String> = pages.keys().cloned().collect();
	for path in extra_dapps {
		if let Some((id, endpoint)) = fs::local_endpoint(path.clone(), embeddable.clone(), pool.clone()) {
			pages.insert(id, endpoint);
		} else {
			warn!(target: "dapps", "Ignoring invalid dapp at {}", path.display());
		}
	}

	// NOTE [ToDr] Dapps will be currently embeded on 8180
	pages.insert(
		"ui".into(),
		Box::new(page::builtin::Dapp::new_safe_to_embed(pool.clone(), ::parity_ui::App::default(), embeddable.clone()))
	);
	// old version
	pages.insert(
		"v1".into(),
		Box::new({
			let mut page = page::builtin::Dapp::new_safe_to_embed(pool.clone(), ::parity_ui::old::App::default(), embeddable.clone());
			// allow JS eval on old Wallet
			page.allow_js_eval();
			page
		})
	);
	pages.insert(
		"proxy".into(),
		ProxyPac::boxed(embeddable.clone(), dapps_domain.to_owned())
	);
	pages.insert(
		WEB_PATH.into(),
		Web::boxed(embeddable.clone(), web_proxy_tokens.clone(), fetch.clone(), pool.clone())
	);

	(local_endpoints, pages)
}
