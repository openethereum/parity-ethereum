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

use std::net::SocketAddr;
use endpoint::Endpoints;
use page::PageEndpoint;
use proxypac::ProxyPac;
use parity_webapp::WebApp;

extern crate parity_status;
#[cfg(feature = "parity-wallet")]
extern crate parity_wallet;

pub fn main_page() -> &'static str {
	"/status/"
}

pub fn all_endpoints(addr: &SocketAddr) -> Endpoints {
	let mut pages = Endpoints::new();
	pages.insert("proxy".to_owned(), ProxyPac::new(addr));

	insert::<parity_status::App>(&mut pages, "status");
	insert::<parity_status::App>(&mut pages, "parity");

	wallet_page(&mut pages);
	pages
}

#[cfg(feature = "parity-wallet")]
fn wallet_page(pages: &mut Endpoints) {
	insert::<parity_wallet::App>(pages, "wallet");
}

#[cfg(not(feature = "parity-wallet"))]
fn wallet_page(_pages: &mut Endpoints) {}

fn insert<T : WebApp + Default + 'static>(pages: &mut Endpoints, id: &str) {
	pages.insert(id.to_owned(), Box::new(PageEndpoint::new(T::default())));
}
