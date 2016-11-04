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

use endpoint::{Endpoints, Endpoint};
use page::PageEndpoint;
use proxypac::ProxyPac;
use parity_dapps::WebApp;

mod cache;
mod fs;
pub mod urlhint;
pub mod fetcher;
pub mod manifest;

extern crate parity_ui;

pub const HOME_PAGE: &'static str = "home";
pub const DAPPS_DOMAIN : &'static str = ".parity";
pub const RPC_PATH : &'static str =  "rpc";
pub const API_PATH : &'static str =  "api";
pub const UTILS_PATH : &'static str =  "parity-utils";

pub fn utils() -> Box<Endpoint> {
	Box::new(PageEndpoint::with_prefix(parity_ui::App::default(), UTILS_PATH.to_owned()))
}

pub fn all_endpoints(dapps_path: String, signer_port: Option<u16>) -> Endpoints {
	// fetch fs dapps at first to avoid overwriting builtins
	let mut pages = fs::local_endpoints(dapps_path, signer_port);

	// NOTE [ToDr] Dapps will be currently embeded on 8180
	insert::<parity_ui::App>(&mut pages, "ui", Embeddable::Yes(signer_port));
	pages.insert("proxy".into(), ProxyPac::boxed(signer_port));

	pages
}

fn insert<T : WebApp + Default + 'static>(pages: &mut Endpoints, id: &str, embed_at: Embeddable) {
	pages.insert(id.to_owned(), Box::new(match embed_at {
		Embeddable::Yes(port) => PageEndpoint::new_safe_to_embed(T::default(), port),
		Embeddable::No => PageEndpoint::new(T::default()),
	}));
}

enum Embeddable {
	Yes(Option<u16>),
	#[allow(dead_code)]
	No,
}
