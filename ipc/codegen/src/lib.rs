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

//! Codegen for IPC RPC

#![cfg_attr(feature = "nightly-testing", plugin(clippy))]
#![cfg_attr(feature = "nightly-testing", feature(plugin))]
#![cfg_attr(feature = "nightly-testing", allow(used_underscore_binding))]
#![cfg_attr(not(feature = "with-syntex"), feature(rustc_private, plugin))]
#![cfg_attr(not(feature = "with-syntex"), plugin(quasi_macros))]

extern crate aster;
extern crate quasi;

#[cfg(feature = "with-syntex")]
extern crate syntex;

#[cfg(feature = "with-syntex")]
extern crate syntex_syntax as syntax;

#[cfg(not(feature = "with-syntex"))]
#[macro_use]
extern crate syntax;

#[cfg(not(feature = "with-syntex"))]
extern crate rustc_plugin;

#[cfg(not(feature = "with-syntex"))]
use syntax::feature_gate::AttributeType;

#[cfg(feature = "with-syntex")]
use syntax::{ast, fold};


#[cfg(feature = "with-syntex")]
include!(concat!(env!("OUT_DIR"), "/lib.rs"));

#[cfg(not(feature = "with-syntex"))]
include!("lib.rs.in");

#[cfg(feature = "with-syntex")]
pub fn expand(src: &std::path::Path, dst: &std::path::Path) {
	let mut registry = syntex::Registry::new();
	register(&mut registry);
	registry.expand("", src, dst).unwrap();
}

#[cfg(feature = "with-syntex")]
struct StripAttributeFolder<'a> {
	attr_title: &'a str,
}

#[cfg(feature = "with-syntex")]
impl<'a> fold::Folder for StripAttributeFolder<'a> {
	fn fold_attribute(&mut self, attr: ast::Attribute) -> Option<ast::Attribute> {
		match attr.node.value.node {
			ast::MetaItemKind::List(ref n, _) if n == self.attr_title => { return None; }
			ast::MetaItemKind::Word(ref n) if n == self.attr_title => { return None; }
			_ => {}
		}

		Some(attr)
	}

	fn fold_mac(&mut self, mac: ast::Mac) -> ast::Mac {
		fold::noop_fold_mac(mac, self)
	}
}

#[cfg(feature = "with-syntex")]
pub fn register_cleaner_ipc(reg: &mut syntex::Registry) {
	#[cfg(feature = "with-syntex")]
	fn strip_attributes(krate: ast::Crate) -> ast::Crate {
		let mut folder = StripAttributeFolder { attr_title: "ipc" };
		fold::Folder::fold_crate(&mut folder, krate)
	}

	reg.add_post_expansion_pass(strip_attributes);
}

#[cfg(feature = "with-syntex")]
pub fn register_cleaner_binary(reg: &mut syntex::Registry) {
	#[cfg(feature = "with-syntex")]
	fn strip_attributes(krate: ast::Crate) -> ast::Crate {
		let mut folder = StripAttributeFolder { attr_title: "binary" };
		fold::Folder::fold_crate(&mut folder, krate)
	}

	reg.add_post_expansion_pass(strip_attributes);
}

#[cfg(feature = "with-syntex")]
pub fn register(reg: &mut syntex::Registry) {
	reg.add_attr("feature(custom_derive)");
	reg.add_attr("feature(custom_attribute)");

	reg.add_decorator("ipc", codegen::expand_ipc_implementation);
	reg.add_decorator("binary", serialization::expand_serialization_implementation);

	register_cleaner_ipc(reg);
	register_cleaner_binary(reg);
}

#[cfg(not(feature = "with-syntex"))]
pub fn register(reg: &mut rustc_plugin::Registry) {
	reg.register_syntax_extension(
		syntax::parse::token::intern("ipc"),
		syntax::ext::base::MultiDecorator(
			Box::new(codegen::expand_ipc_implementation)));
	reg.register_syntax_extension(
		syntax::parse::token::intern("binary"),
		syntax::ext::base::MultiDecorator(
			Box::new(serialization::expand_serialization_implementation)));

	reg.register_attribute("ipc".to_owned(), AttributeType::Normal);
	reg.register_attribute("binary".to_owned(), AttributeType::Normal);
}

#[derive(Debug)]
pub enum Error { InvalidFileName, ExpandFailure, Io(std::io::Error) }

impl std::convert::From<std::io::Error> for Error {
	fn from(err: std::io::Error) -> Self {
		Error::Io(err)
	}
}

pub fn derive_ipc_cond(src_path: &str, has_feature: bool) -> Result<(), Error> {
	if has_feature { derive_ipc(src_path) }
	else { cleanup_ipc(src_path) }
}

pub fn cleanup_ipc(src_path: &str) -> Result<(), Error> {
	cleanup(src_path, AttributeKind::Ipc)
}

pub fn cleanup_binary(src_path: &str) -> Result<(), Error> {
	cleanup(src_path, AttributeKind::Binary)
}

enum AttributeKind {
	Ipc,
	Binary,
}

fn cleanup(src_path: &str, attr: AttributeKind) -> Result<(), Error> {
	use std::env;
	use std::path::{Path, PathBuf};

	let out_dir = env::var_os("OUT_DIR").unwrap();
	let file_name = PathBuf::from(src_path).file_name().ok_or(Error::InvalidFileName).map(|val| val.to_str().unwrap().to_owned())?;
	let mut registry = syntex::Registry::new();

	match attr {
		AttributeKind::Ipc => { register_cleaner_ipc(&mut registry); }
		AttributeKind::Binary => { register_cleaner_binary(&mut registry); }
	}

	if let Err(_) = registry.expand("", &Path::new(src_path), &Path::new(&out_dir).join(&file_name))
	{
		// will be reported by compiler
		return Err(Error::ExpandFailure)
	}
	Ok(())
}

pub fn derive_ipc(src_path: &str) -> Result<(), Error> {
	use std::env;
	use std::path::{Path, PathBuf};

	let out_dir = env::var_os("OUT_DIR").unwrap();
	let file_name = PathBuf::from(src_path).file_name().ok_or(Error::InvalidFileName).map(|val| val.to_str().unwrap().to_owned())?;

	let final_path = Path::new(&out_dir).join(&file_name);

	let mut intermediate_file_name = file_name.clone();
	intermediate_file_name.push_str(".rpc.in");
	let intermediate_path = Path::new(&out_dir).join(&intermediate_file_name);

	{
		let mut registry = syntex::Registry::new();
		register(&mut registry);
		if let Err(_) = registry.expand("", &Path::new(src_path), &intermediate_path) {
			// will be reported by compiler
			return Err(Error::ExpandFailure)
		}
	}

	{
		let mut registry = syntex::Registry::new();
		register(&mut registry);
		if let Err(_) = registry.expand("", &intermediate_path, &final_path) {
			// will be reported by compiler
			return Err(Error::ExpandFailure)
		}
	}

	Ok(())
}

pub fn derive_binary(src_path: &str) -> Result<(), Error> {
	use std::env;
	use std::path::{Path, PathBuf};

	let out_dir = env::var_os("OUT_DIR").unwrap();
	let file_name = PathBuf::from(src_path).file_name().ok_or(Error::InvalidFileName).map(|val| val.to_str().unwrap().to_owned())?;
	let final_path = Path::new(&out_dir).join(&file_name);

	let mut registry = syntex::Registry::new();
	register(&mut registry);
	if let Err(_) = registry.expand("", &Path::new(src_path), &final_path) {
		// will be reported by compiler
		return Err(Error::ExpandFailure)
	}

	Ok(())
}

pub fn derive_binary_cond(src_path: &str, has_feature: bool) -> Result<(), Error> {
	if has_feature { derive_binary(src_path) }
	else { cleanup_binary(src_path) }
}
