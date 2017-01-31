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

use std::path::PathBuf;
use std::sync::Arc;
use endpoint::{Endpoints, Endpoint};
use page::PageEndpoint;
use proxypac::ProxyPac;
use web::Web;
use fetch::Fetch;
use parity_dapps::WebApp;
use parity_reactor::Remote;
use {WebProxyTokens};

mod cache;
mod fs;
pub mod fetcher;
pub mod manifest;

extern crate parity_ui;

pub const HOME_PAGE: &'static str = "parity";
pub const DAPPS_DOMAIN: &'static str = ".web3.site";
pub const RPC_PATH: &'static str =  "rpc";
pub const API_PATH: &'static str =  "api";
pub const UTILS_PATH: &'static str =  "parity-utils";
pub const WEB_PATH: &'static str = "web";
pub const URL_REFERER: &'static str = "__referer=";

pub fn utils() -> Box<Endpoint> {
	Box::new(PageEndpoint::with_prefix(parity_ui::App::default(), UTILS_PATH.to_owned()))
}

pub fn all_endpoints<F: Fetch>(
	dapps_path: PathBuf,
	extra_dapps: Vec<PathBuf>,
	signer_address: Option<(String, u16)>,
	web_proxy_tokens: Arc<WebProxyTokens>,
	remote: Remote,
	fetch: F,
) -> Endpoints {
	// fetch fs dapps at first to avoid overwriting builtins
	let mut pages = fs::local_endpoints(dapps_path, signer_address.clone());
	for path in extra_dapps {
		if let Some((id, endpoint)) = fs::local_endpoint(path.clone(), signer_address.clone()) {
			pages.insert(id, endpoint);
		} else {
			warn!(target: "dapps", "Ignoring invalid dapp at {}", path.display());
		}
	}

	// NOTE [ToDr] Dapps will be currently embeded on 8180
	insert::<parity_ui::App>(&mut pages, "ui", Embeddable::Yes(signer_address.clone()));
	pages.insert("proxy".into(), ProxyPac::boxed(signer_address.clone()));
	pages.insert(WEB_PATH.into(), Web::boxed(signer_address.clone(), web_proxy_tokens.clone(), remote.clone(), fetch.clone()));

	pages
}

fn insert<T : WebApp + Default + 'static>(pages: &mut Endpoints, id: &str, embed_at: Embeddable) {
	pages.insert(id.to_owned(), Box::new(match embed_at {
		Embeddable::Yes(address) => PageEndpoint::new_safe_to_embed(T::default(), address),
		Embeddable::No => PageEndpoint::new(T::default()),
	}));
}

enum Embeddable {
	Yes(Option<(String, u16)>),
	#[allow(dead_code)]
	No,
}
