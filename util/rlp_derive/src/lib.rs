extern crate proc_macro;
extern crate syn;
#[macro_use]
extern crate quote;

mod en;
mod de;

use proc_macro::TokenStream;
use en::impl_encodable;
use de::impl_decodable;

#[proc_macro_derive(RlpEncodable)]
pub fn encodable(input: TokenStream) -> TokenStream {
	let s = input.to_string();
	let ast = syn::parse_derive_input(&s).unwrap();
	let gen = impl_encodable(&ast);
	gen.parse().unwrap()
}

#[proc_macro_derive(RlpDecodable)]
pub fn decodable(input: TokenStream) -> TokenStream {
	let s = input.to_string();
	let ast = syn::parse_derive_input(&s).unwrap();
	let gen = impl_decodable(&ast);
	gen.parse().unwrap()
}
