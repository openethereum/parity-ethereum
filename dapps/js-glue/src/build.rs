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

#[cfg(feature = "with-syntex")]
pub mod inner {
	use syntex;
	use codegen;
	use syntax::{ast, fold};
	use std::env;
	use std::path::Path;

	fn strip_attributes(krate: ast::Crate) -> ast::Crate {
		/// Helper folder that strips the serde attributes after the extensions have been expanded.
		struct StripAttributeFolder;

		impl fold::Folder for StripAttributeFolder {
			fn fold_attribute(&mut self, attr: ast::Attribute) -> Option<ast::Attribute> {
				if &*attr.value.name.as_str() == "webapp" {
					return None;
				}

				Some(attr)
			}

			fn fold_mac(&mut self, mac: ast::Mac) -> ast::Mac {
				fold::noop_fold_mac(mac, self)
			}
		}

		fold::Folder::fold_crate(&mut StripAttributeFolder, krate)
	}

	pub fn register(reg: &mut syntex::Registry) {
		reg.add_attr("feature(custom_derive)");
		reg.add_attr("feature(custom_attribute)");

		reg.add_decorator("derive_WebAppFiles", codegen::expand_webapp_implementation);
		reg.add_post_expansion_pass(strip_attributes);
	}

	pub fn generate() {
		let out_dir = env::var_os("OUT_DIR").unwrap();
		let mut registry = syntex::Registry::new();
		register(&mut registry);

		let src = Path::new("src/lib.rs.in");
		let dst = Path::new(&out_dir).join("lib.rs");

		registry.expand("", &src, &dst).unwrap();
	}
}

#[cfg(not(feature = "with-syntex"))]
pub mod inner {
	use codegen;

	pub fn register(reg: &mut rustc_plugin::Registry) {
		reg.register_syntax_extension(
			syntax::parse::token::intern("derive_WebAppFiles"),
			syntax::ext::base::MultiDecorator(
				Box::new(codegen::expand_webapp_implementation)));

		reg.register_attribute("webapp".to_owned(), AttributeType::Normal);
	}

	pub fn generate() {}
}
