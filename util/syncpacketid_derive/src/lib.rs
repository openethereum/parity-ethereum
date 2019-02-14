#![recursion_limit="128"]

// Needs to be "extern crate" even in rust 2018:
// https://blog.rust-lang.org/2018/12/21/Procedural-Macros-in-Rust-2018.html
extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;

/// The SyncPackets derive-macro will provide an enum with this attribute:
///
/// * With a method "from_u8" which will optionally convert a u8 value to
///   one of the variants or None if the value is unknown.
///
/// * With an implementation of a trait PacketInfo to get the packet id and
///   the protocol from instances of the enum.
#[proc_macro_derive(SyncPackets, attributes(protocol))]
pub fn sync_packets(input: TokenStream) -> TokenStream {
	let ast = syn::parse(input).unwrap();
	let gen = impl_sync_packets(&ast);
	gen.into()
}

fn parse_protocol_attribute(input: proc_macro2::TokenStream) -> proc_macro2::Ident {
	let groups: Vec<_> = input.into_iter().take(1).collect();

	let group: Vec<_> = match &groups[0] {
		proc_macro2::TokenTree::Group(g) => g.stream().into_iter().take(1).collect(),
		_ => panic!()
	};

	if let proc_macro2::TokenTree::Ident(ref i) = group[0] {
		proc_macro2::Ident::new(i.to_string().as_ref(), i.span())
	} else {
		panic!("Should be an Ident");
	}
}

fn impl_sync_packets(ast: &syn::DeriveInput) -> proc_macro2::TokenStream {
	let body = match ast.data {
		syn::Data::Enum(ref e) => e,
		_ => panic!("#[derive(SyncPackets)] is only defined for enums."),
	};

	let enum_name = &ast.ident;

	let idents1: Vec<_> = body.variants.iter().map(|v| &v.ident).collect();
	let idents2 = idents1.clone();

	let prots:Vec<_> = body.variants.iter()
		.filter(|v| v.attrs.get(0).expect("attribute is missing; annotate your enum patterns with #[eth] or #[par]").path.is_ident("protocol"))
		.map(|v| parse_protocol_attribute(v.attrs.get(0).unwrap().tts.clone())).collect();

	let values: Vec<_> = body.variants.iter().map(|v| v.discriminant.clone().expect("enum pattern is not discriminant; should have assigned unique value such as #[eth] Foo = 1").1).collect();

	quote!{
		use network::{PacketId, ProtocolId};

		impl #enum_name {
			pub fn from_u8(id: u8) -> Option<SyncPacket> {
				match id {
					#(#values => Some(#idents1)),*,
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
					#(#idents2 => #prots),*
				}
			}

			fn id(&self) -> PacketId {
				(*self) as PacketId
			}
		}
	}
}
