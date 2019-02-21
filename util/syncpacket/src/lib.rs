// Copyright 2015-2019 Parity Technologies (UK) Ltd.
// This file is part of Parity Ethereum.

// Parity Ethereum is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity Ethereum is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity Ethereum.  If not, see <http://www.gnu.org/licenses/>.

#![recursion_limit="128"]

// Needs to be "extern crate" even in rust 2018:
// https://blog.rust-lang.org/2018/12/21/Procedural-Macros-in-Rust-2018.html
extern crate proc_macro;

use proc_macro2::Ident;
use quote::quote;
use syn::{Attribute, Data, DeriveInput, Meta, MetaList, NestedMeta, Result};

fn parse_protocol_arguments(args: &MetaList) -> Result<&Ident> {
	if args.nested.len() != 1 {
		return Err(syn::Error::new_spanned(args, "protocol attribute should have exactly one argument"));
	}

	// We have exactly one argument, should not panic
	match args.nested.first().expect("protocol attribute without value").value() {
		// Meta argument
		NestedMeta::Meta(meta) => match meta {
			Meta::Word(ident) => Ok(&ident),
			_ => return Err(syn::Error::new_spanned(meta, "nested arguments to protocol are not allowed")),
		},
		// Quoted string
		a @ _ => return Err(syn::Error::new_spanned(a, "protocol argument must be an unquoted identifier"))
	}
}

/// Helper function to parse arguments to the protocol attribute.
/// Syntax should be #[protocol(P)] PacketName = 0xNN,
fn parse_protocol_attribute(input: &Attribute) -> Result<Ident> {
	let argument = match input.parse_meta() {
		Ok(arg) => arg,
		Err(err) => return Err(err)
	};

	// Arguments to invocation attributes are delivered as a list
	match argument {
		Meta::Word(_) => Err(syn::Error::new_spanned(input, "protocol attribute without argument")),
		Meta::List(args) => parse_protocol_arguments(&args).map(|ok| ok.clone()),
		_ => Err(syn::Error::new_spanned(input, "unsupported syntax"))
	}
}

/// The SyncPackets derive-macro will provide an enum with this attribute:
///
/// * With a method "from_u8" which will optionally convert a u8 value to
///   one of the variants or None if the value is unknown.
///
/// * With an implementation of a trait PacketInfo to get the packet id and
///   the protocol from instances of the enum.
#[proc_macro_derive(SyncPackets, attributes(protocol))]
pub fn sync_packets(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
	// Can we ever panic here?
	let ast = syn::parse(input).expect("invalid enum syntax");

	match impl_sync_packets(&ast) {
		Ok(output) => output.into(),
		Err(err) => err.to_compile_error().into(),
	}
}

fn impl_sync_packets(ast: &DeriveInput) -> Result<proc_macro2::TokenStream> {
	let body = match ast.data {
		Data::Enum(ref e) => e,
		_ => return Err(syn::Error::new_spanned(ast, "#[derive(SyncPackets)] is only defined for enums.")),
	};

	let enum_name = &ast.ident;

	if body.variants.is_empty() {
		return Err(syn::Error::new_spanned(enum_name, format!("enum {} has no variants defined", enum_name)));
	}

	// Apparently quote! consumes interpolated variables. Clone ids
	// to use them twice.
	let idents_from_u8: Vec<_> = body.variants.iter().map(|v| &v.ident).collect();
	let idents_enum = idents_from_u8.clone();

	// Within each variant of the enum find the first "protocol" attribute
	// and extract its argument
	let protocols:Vec<_> = match body.variants.iter()
		.map(
			|v| v.attrs
				.iter()
				.find(|&x| x.path.is_ident("protocol"))
				.ok_or(syn::Error::new_spanned(v, format!("enum variant without protocol attribute {}", &v.ident)))
				.and_then(|ref a| parse_protocol_attribute(a))
		).collect() {
			Ok(v) => v,
			Err(err) => return Err(err),
		};


	// Values asigned to the variants in the enum
	let values: Vec<_> = match body.variants.iter()
		.map(
			|v| v.discriminant
				.as_ref()
				.map(|d| &d.1)
				.ok_or(syn::Error::new_spanned(v, "enum pattern is not discriminant; should have assigned a unique value such as Foo = 1"))
		)
		.collect() {
			Ok(v) => v,
			Err(err) => return Err(err),
		};

	Ok(quote!{
			use network::{PacketId, ProtocolId};

			impl #enum_name {
				pub fn from_u8(id: u8) -> Option<SyncPacket> {
					match id {
						#(#values => Some(#idents_from_u8)),*,
						_ => None

					}
				}
			}

			use self::SyncPacket::*;

			/// Provide both subprotocol and packet id information within the
			/// same object.
			pub trait PacketInfo {
				fn id(&self) -> PacketId;
				fn protocol(&self) -> ProtocolId;
			}

			impl PacketInfo for #enum_name {
				fn protocol(&self) -> ProtocolId {
					match self {
						#(#idents_enum => #protocols),*
					}
				}

				fn id(&self) -> PacketId {
					(*self) as PacketId
				}
			}
		}
	)
}
