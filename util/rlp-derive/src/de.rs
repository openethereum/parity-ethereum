// Copyright 2015-2020 Parity Technologies (UK) Ltd.
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

use proc_macro2::TokenStream;
use quote::quote;

struct ParseQuotes {
	single: TokenStream,
	list: TokenStream,
	takes_index: bool,
}

fn decodable_parse_quotes() -> ParseQuotes {
	ParseQuotes {
		single: quote! { rlp.val_at },
		list: quote! { rlp.list_at },
		takes_index: true,
	}
}

fn decodable_wrapper_parse_quotes() -> ParseQuotes {
	ParseQuotes {
		single: quote! { rlp.as_val },
		list: quote! { rlp.as_list },
		takes_index: false,
	}
}

pub fn impl_decodable(ast: &syn::DeriveInput) -> TokenStream {
	let body = match ast.data {
		syn::Data::Struct(ref s) => s,
		_ => panic!("#[derive(RlpDecodable)] is only defined for structs."),
	};

	let mut default_attribute_encountered = false;
	let stmts: Vec<_> = body
		.fields
		.iter()
		.enumerate()
		.map(|(i, field)| decodable_field(
			i,
			field,
			decodable_parse_quotes(),
			&mut default_attribute_encountered,
		)).collect();
	let name = &ast.ident;

	let impl_block = quote! {
		impl rlp::Decodable for #name {
			fn decode(rlp: &rlp::Rlp) -> Result<Self, rlp::DecoderError> {
				let result = #name {
					#(#stmts)*
				};

				Ok(result)
			}
		}
	};

	quote! {
		const _: () = {
			extern crate rlp;
			#impl_block
		};
	}
}

pub fn impl_decodable_wrapper(ast: &syn::DeriveInput) -> TokenStream {
	let body = match ast.data {
		syn::Data::Struct(ref s) => s,
		_ => panic!("#[derive(RlpDecodableWrapper)] is only defined for structs."),
	};

	let stmt = {
		let fields: Vec<_> = body.fields.iter().collect();
		if fields.len() == 1 {
			let field = fields.first().expect("fields.len() == 1; qed");
			let mut default_attribute_encountered = false;
			decodable_field(
				0,
				field,
				decodable_wrapper_parse_quotes(),
				&mut default_attribute_encountered,
			)
		} else {
			panic!("#[derive(RlpEncodableWrapper)] is only defined for structs with one field.")
		}
	};

	let name = &ast.ident;

	let impl_block = quote! {
		impl rlp::Decodable for #name {
			fn decode(rlp: &rlp::Rlp) -> Result<Self, rlp::DecoderError> {
				let result = #name {
					#stmt
				};

				Ok(result)
			}
		}
	};

	quote! {
		const _: () = {
			extern crate rlp;
			#impl_block
		};
	}
}

fn decodable_field(
	index: usize,
	field: &syn::Field,
	quotes: ParseQuotes,
	default_attribute_encountered: &mut bool,
) -> TokenStream {
	let id = match field.ident {
		Some(ref ident) => quote! { #ident },
		None => {
			let index: syn::Index = index.into();
			quote! { #index }
		}
	};

	let index = index - *default_attribute_encountered as usize;
	let index = quote! { #index };

	let single = quotes.single;
	let list = quotes.list;

	let attributes = &field.attrs;
	let default = if let Some(attr) = attributes.iter().find(|attr| attr.path.is_ident("rlp")) {
		if *default_attribute_encountered {
			panic!("only 1 #[rlp(default)] attribute is allowed in a struct")
		}
		match attr.parse_args() {
			Ok(proc_macro2::TokenTree::Ident(ident)) if ident.to_string() == "default" => {},
			_ => panic!("only #[rlp(default)] attribute is supported"),
		}
		*default_attribute_encountered = true;
		true
	} else {
		false
	};

	match field.ty {
		syn::Type::Path(ref path) => {
			let ident = &path
				.path
				.segments
				.first()
				.expect("there must be at least 1 segment")
				.ident;
			let ident_type = ident.to_string();
			if &ident_type == "Vec" {
				if quotes.takes_index {
					if default {
						quote! { #id: #list(#index).unwrap_or_default(), }
					} else {
						quote! { #id: #list(#index)?, }
					}
				} else {
					quote! { #id: #list()?, }
				}
			} else {
				if quotes.takes_index {
					if default {
						quote! { #id: #single(#index).unwrap_or_default(), }
					} else {
						quote! { #id: #single(#index)?, }
					}
				} else {
					quote! { #id: #single()?, }
				}
			}
		}
		_ => panic!("rlp_derive not supported"),
	}
}
