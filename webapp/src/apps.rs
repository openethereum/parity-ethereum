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

use std::collections::HashMap;
use page::{Page, PageHandler};

extern crate parity_wallet;

pub type Pages = HashMap<String, Box<Page>>;

pub fn all_pages() -> Pages {
	let mut pages = Pages::new();
	wallet_page(&mut pages);
	pages
}

#[cfg(feature="parity-wallet")]
fn wallet_page(pages: &mut Pages) {
	pages.insert("wallet".to_owned(), Box::new(PageHandler { app: parity_wallet::App::default() }));
}

#[cfg(not(feature="parity-wallet"))]
fn wallet_page(_pages: &mut Pages) {}

