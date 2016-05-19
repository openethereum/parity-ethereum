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
use parity_webapp::WebApp;

extern crate parity_status;
extern crate parity_idmanager;


pub const DAPPS_DOMAIN : &'static str = ".parity";
pub const RPC_PATH : &'static str =  "rpc";
pub const API_PATH : &'static str =  "api";
pub const UTILS_PATH : &'static str =  "parity-utils";

pub fn main_page() -> &'static str {
	"/home/"
}

pub fn utils() -> Box<Endpoint> {
	Box::new(PageEndpoint::with_prefix(parity_idmanager::App::default(), UTILS_PATH.to_owned()))
}

pub fn all_endpoints() -> Endpoints {
	let mut pages = Endpoints::new();
	pages.insert("proxy".to_owned(), ProxyPac::boxed());

	insert::<parity_status::App>(&mut pages, "status");
	insert::<parity_status::App>(&mut pages, "parity");
	insert::<parity_idmanager::App>(&mut pages, "home");

	wallet_page(&mut pages);
	daodapp_page(&mut pages);
	makerotc_page(&mut pages);
	pages
}

#[cfg(feature = "parity-wallet")]
fn wallet_page(pages: &mut Endpoints) {
	extern crate parity_wallet;
	insert::<parity_wallet::App>(pages, "wallet");
}
#[cfg(not(feature = "parity-wallet"))]
fn wallet_page(_pages: &mut Endpoints) {}

#[cfg(feature = "parity-daodapp")]
fn daodapp_page(pages: &mut Endpoints) {
	extern crate parity_daodapp;
	insert::<parity_daodapp::App>(pages, "dao");
}
#[cfg(not(feature = "parity-daodapp"))]
fn daodapp_page(_pages: &mut Endpoints) {}

#[cfg(feature = "parity-makerotc")]
fn makerotc_page(pages: &mut Endpoints) {
	extern crate parity_makerotc;
	insert::<parity_makerotc::App>(pages, "makerotc");
}
#[cfg(not(feature = "parity-makerotc"))]
fn makerotc_page(_pages: &mut Endpoints) {}

fn insert<T : WebApp + Default + 'static>(pages: &mut Endpoints, id: &str) {
	pages.insert(id.to_owned(), Box::new(PageEndpoint::new(T::default())));
}
