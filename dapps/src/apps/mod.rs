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
use parity_dapps::{self, WebApp};
use parity_dapps_glue::WebApp as NewWebApp;

mod cache;
mod fs;
pub mod urlhint;
pub mod fetcher;
pub mod manifest;

extern crate parity_dapps_home;
extern crate parity_ui;

pub const DAPPS_DOMAIN : &'static str = ".parity";
pub const RPC_PATH : &'static str =  "rpc";
pub const API_PATH : &'static str =  "api";
pub const UTILS_PATH : &'static str =  "parity-utils";

pub fn main_page() -> &'static str {
	"home"
}
pub fn redirection_address(using_dapps_domains: bool, app_id: &str) -> String {
	if using_dapps_domains {
		format!("http://{}{}/", app_id, DAPPS_DOMAIN)
	} else {
		format!("/{}/", app_id)
	}
}

pub fn utils() -> Box<Endpoint> {
	Box::new(PageEndpoint::with_prefix(parity_dapps_home::App::default(), UTILS_PATH.to_owned()))
}

pub fn all_endpoints(dapps_path: String, signer_port: Option<u16>) -> Endpoints {
	// fetch fs dapps at first to avoid overwriting builtins
	let mut pages = fs::local_endpoints(dapps_path);

	// NOTE [ToDr] Dapps will be currently embeded on 8180
	pages.insert("ui".into(), Box::new(
		PageEndpoint::new_safe_to_embed(NewUi::default(), signer_port)
	));

	pages.insert("proxy".into(), ProxyPac::boxed());
	insert::<parity_dapps_home::App>(&mut pages, "home");


	pages
}

fn insert<T : WebApp + Default + 'static>(pages: &mut Endpoints, id: &str) {
	pages.insert(id.to_owned(), Box::new(PageEndpoint::new(T::default())));
}

// TODO [ToDr] Temporary wrapper until we get rid of old built-ins.
use std::collections::HashMap;

struct NewUi {
	app: parity_ui::App,
	files: HashMap<&'static str, parity_dapps::File>,
}

impl Default for NewUi {
	fn default() -> Self {
		let app = parity_ui::App::default();
		let files = {
			let mut files = HashMap::new();
			for (k, v) in &app.files {
				files.insert(*k, parity_dapps::File {
					path: v.path,
					content: v.content,
					content_type: v.content_type,
				});
			}
			files
		};

		NewUi {
			app: app,
			files: files,
		}
	}
}

impl WebApp for NewUi {
	fn file(&self, path: &str) -> Option<&parity_dapps::File> {
		self.files.get(path)
	}

	fn info(&self) -> parity_dapps::Info {
		let info = self.app.info();
		parity_dapps::Info {
			name: info.name,
			version: info.version,
			author: info.author,
			description: info.description,
			icon_url: info.icon_url,
		}
	}
}
