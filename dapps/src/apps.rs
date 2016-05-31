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

extern crate parity_dapps_status;
extern crate parity_dapps_builtins;


pub const DAPPS_DOMAIN : &'static str = ".parity";
pub const RPC_PATH : &'static str =  "rpc";
pub const API_PATH : &'static str =  "api";
pub const UTILS_PATH : &'static str =  "parity-utils";

pub fn main_page() -> &'static str {
	"/home/"
}

pub fn utils() -> Box<Endpoint> {
	Box::new(PageEndpoint::with_prefix(parity_dapps_builtins::App::default(), UTILS_PATH.to_owned()))
}

pub fn all_endpoints() -> Endpoints {
	let mut pages = Endpoints::new();
	pages.insert("proxy".into(), ProxyPac::boxed());

	// Home page needs to be safe embed
	// because we use Cross-Origin LocalStorage.
	// TODO [ToDr] Account naming should be moved to parity.
	pages.insert("home".into(), Box::new(
		PageEndpoint::new_safe_to_embed(parity_dapps_builtins::App::default())
	));
	insert::<parity_dapps_status::App>(&mut pages, "status");
	insert::<parity_dapps_status::App>(&mut pages, "parity");

	wallet_page(&mut pages);
	daodapp_page(&mut pages);
	makerotc_page(&mut pages);
	pages
}

#[cfg(feature = "parity-dapps-wallet")]
fn wallet_page(pages: &mut Endpoints) {
	extern crate parity_dapps_wallet;
	insert::<parity_dapps_wallet::App>(pages, "wallet");
}
#[cfg(not(feature = "parity-dapps-wallet"))]
fn wallet_page(_pages: &mut Endpoints) {}

#[cfg(feature = "parity-dapps-daodapp")]
fn daodapp_page(pages: &mut Endpoints) {
	extern crate parity_dapps_daodapp;
	insert::<parity_dapps_daodapp::App>(pages, "dao");
}
#[cfg(not(feature = "parity-dapps-daodapp"))]
fn daodapp_page(_pages: &mut Endpoints) {}

#[cfg(feature = "parity-dapps-makerotc")]
fn makerotc_page(pages: &mut Endpoints) {
	extern crate parity_dapps_makerotc;
	insert::<parity_dapps_makerotc::App>(pages, "makerotc");
}
#[cfg(not(feature = "parity-dapps-makerotc"))]
fn makerotc_page(_pages: &mut Endpoints) {}

fn insert<T : WebApp + Default + 'static>(pages: &mut Endpoints, id: &str) {
	pages.insert(id.to_owned(), Box::new(PageEndpoint::new(T::default())));
}
