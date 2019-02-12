#![recursion_limit="128"]

extern crate proc_macro;
extern crate proc_macro2;
extern crate syn;
#[macro_use]
extern crate quote;



use self::proc_macro::TokenStream;

#[proc_macro_derive(SyncPackets, attributes(eth, par))]
pub fn sync_packets(input: TokenStream) -> TokenStream {
	let ast = syn::parse(input).unwrap();
	let gen = impl_sync_packets(&ast);
	gen.into()
}


fn impl_sync_packets(ast: &syn::DeriveInput) -> proc_macro2::TokenStream {
	let body = match ast.data {
		syn::Data::Enum(ref e) => e,
		_ => panic!("#[derive(RlpEncodable)] is only defined for enums."),
	};

	let eths: Vec<_> = body.variants.iter().filter(|v| v.attrs[0].path.is_ident("eth")).map(|v| &v.ident).collect();
	let pars: Vec<_> = body.variants.iter().filter(|v| v.attrs[0].path.is_ident("par")).map(|v| &v.ident).collect();

	let idents: Vec<_> = body.variants.iter().map(|v| &v.ident).collect();
	let values: Vec<_> = body.variants.iter().map(|v| v.discriminant.clone().unwrap().1).collect();

	quote!{
		impl SyncPacket {
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

		// The mechanism to match packet ids and protocol may be improved
		// through some macro magic, but for now this works.
		impl PacketInfo for SyncPacket {
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
