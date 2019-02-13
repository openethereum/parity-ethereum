#![recursion_limit="128"]

extern crate proc_macro;
extern crate proc_macro2;
extern crate syn;
#[macro_use]
extern crate quote;

use self::proc_macro::TokenStream;

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
		.filter(|v| v.attrs[0].path.is_ident("eth"))
		.map(|v| &v.ident).collect();

	let pars: Vec<_> = body.variants.iter()
		.filter(|v| v.attrs[0].path.is_ident("par"))
		.map(|v| &v.ident).collect();

	let idents: Vec<_> = body.variants.iter().map(|v| &v.ident).collect();
	let values: Vec<_> = body.variants.iter().map(|v| v.discriminant.clone().unwrap().1).collect();

	quote!{
		use api::{ETH_PROTOCOL, WARP_SYNC_PROTOCOL_ID};
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
