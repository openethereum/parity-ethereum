use {syn, quote};

pub fn impl_decodable(ast: &syn::DeriveInput) -> quote::Tokens {
	let body = match ast.body {
		syn::Body::Struct(ref s) => s,
		_ => panic!("#[derive(Decodable)] is only defined for structs."),
	};

	let stmts: Vec<_> = match *body {
		syn::VariantData::Struct(ref fields) => fields.iter().enumerate().map(decodable_field_map).collect(),
		syn::VariantData::Tuple(ref fields) => fields.iter().enumerate().map(decodable_field_map).collect(),
		syn::VariantData::Unit => panic!("#[derive(Decodable)] is not defined for Unit structs."),
	};

	let name = &ast.ident;

	let dummy_const = syn::Ident::new(format!("_IMPL_RLP_DECODABLE_FOR_{}", name));
	let impl_block = quote! {
		impl rlp::Decodable for #name {
			fn decode(rlp: &rlp::UntrustedRlp) -> Result<Self, rlp::DecoderError> {
				let result = #name {
					#(#stmts)*
				};

				Ok(result)
			}
		}
	};

	quote! {
		#[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
		const #dummy_const: () = {
			extern crate rlp;
			#impl_block
		};
	}
}

fn decodable_field_map(tuple: (usize, &syn::Field)) -> quote::Tokens {
	decodable_field(tuple.0, tuple.1)
}

fn decodable_field(index: usize, field: &syn::Field) -> quote::Tokens {
	let ident = match field.ident {
		Some(ref ident) => ident.to_string(),
		None => index.to_string(),
	};

	let id = syn::Ident::new(ident);
	let index = syn::Ident::new(index.to_string());

	match field.ty {
		syn::Ty::Path(_, ref path) => {
			let ident = &path.segments.first().expect("there must be at least 1 segment").ident;
			if &ident.to_string() == "Vec" {
				quote! { #id: rlp.list_at(#index)?, }
			} else {
				quote! { #id: rlp.val_at(#index)?, }
			}
		},
		_ => panic!("rlp_derive not supported"),
	}
}

