extern crate proc_macro;
extern crate syn;
#[macro_use]
extern crate quote;

mod en;
mod de;

use proc_macro::TokenStream;
use en::{impl_encodable, impl_encodable_wrapper};
use de::{impl_decodable, impl_decodable_wrapper};

#[proc_macro_derive(RlpEncodable)]
pub fn encodable(input: TokenStream) -> TokenStream {
	let s = input.to_string();
	let ast = syn::parse_derive_input(&s).unwrap();
	let gen = impl_encodable(&ast);
	gen.parse().unwrap()
}

#[proc_macro_derive(RlpEncodableWrapper)]
pub fn encodable_wrapper(input: TokenStream) -> TokenStream {
	let s = input.to_string();
	let ast = syn::parse_derive_input(&s).unwrap();
	let gen = impl_encodable_wrapper(&ast);
	gen.parse().unwrap()
}

#[proc_macro_derive(RlpDecodable)]
pub fn decodable(input: TokenStream) -> TokenStream {
	let s = input.to_string();
	let ast = syn::parse_derive_input(&s).unwrap();
	let gen = impl_decodable(&ast);
	gen.parse().unwrap()
}

#[proc_macro_derive(RlpDecodableWrapper)]
pub fn decodable_wrapper(input: TokenStream) -> TokenStream {
	let s = input.to_string();
	let ast = syn::parse_derive_input(&s).unwrap();
	let gen = impl_decodable_wrapper(&ast);
	gen.parse().unwrap()
}
