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
#[proc_macro_derive(SyncPackets, attributes(eth, par))]
pub fn sync_packets(input: TokenStream) -> TokenStream {
	let ast = syn::parse(input).unwrap();
	let gen = impl_sync_packets(&ast);
	gen.into()
}

fn impl_sync_packets(ast: &syn::DeriveInput) -> proc_macro2::TokenStream {
	let body = match ast.data {
		syn::Data::Enum(ref e) => e,
		_ => panic!("#[derive(SyncPackets)] is only defined for enums."),
	};

	let enum_name = &ast.ident;

	let eths: Vec<_> = body.variants.iter()
		.filter(|v| v.attrs.get(0).expect("attribute is missing; annotate your enum patterns with #[eth] or #[par]").path.is_ident("eth"))
		.map(|v| &v.ident).collect();

	let pars: Vec<_> = body.variants.iter()
		.filter(|v| v.attrs.get(0).expect("attribute is missing; annotate your enum patterns with #[eth] or #[par]").path.is_ident("par"))
		.map(|v| &v.ident).collect();

	let idents: Vec<_> = body.variants.iter().map(|v| &v.ident).collect();
	let values: Vec<_> = body.variants.iter().map(|v| v.discriminant.clone().unwrap().1).collect();

	quote!{
		use crate::api::{ETH_PROTOCOL, WARP_SYNC_PROTOCOL_ID};
		use network::{PacketId, ProtocolId};

		impl #enum_name {
			pub fn from_u8(id: u8) -> Option<SyncPacket> {
				match id {
					#(#values => Some(#idents)),*,
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
					#(#eths)|* => ETH_PROTOCOL,
					#(#pars)|* => WARP_SYNC_PROTOCOL_ID,
				}
			}

			fn id(&self) -> PacketId {
				(*self) as PacketId
			}
		}
	}
}
