use {syn, quote};

pub fn impl_encodable(ast: &syn::DeriveInput) -> quote::Tokens {
	let body = match ast.body {
		syn::Body::Struct(ref s) => s,
		_ => panic!("#[derive(RlpEncodable)] is only defined for structs."),
	};

	let stmts: Vec<_> = match *body {
		syn::VariantData::Struct(ref fields) | syn::VariantData::Tuple(ref fields) =>
			fields.iter().enumerate().map(encodable_field_map).collect(),
		syn::VariantData::Unit => panic!("#[derive(RlpEncodable)] is not defined for Unit structs."),
	};

	let name = &ast.ident;

	let stmts_len = syn::Ident::new(stmts.len().to_string());
	let dummy_const = syn::Ident::new(format!("_IMPL_RLP_ENCODABLE_FOR_{}", name));
	let impl_block = quote! {
		impl rlp::Encodable for #name {
			fn rlp_append(&self, stream: &mut rlp::RlpStream) {
				stream.begin_list(#stmts_len);
				#(#stmts)*
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

pub fn impl_encodable_wrapper(ast: &syn::DeriveInput) -> quote::Tokens {
	let body = match ast.body {
		syn::Body::Struct(ref s) => s,
		_ => panic!("#[derive(RlpEncodableWrapper)] is only defined for structs."),
	};

	let stmt = match *body {
		syn::VariantData::Struct(ref fields) | syn::VariantData::Tuple(ref fields) => {
			if fields.len() == 1 {
				let field = fields.first().expect("fields.len() == 1; qed");
				encodable_field(0, field)
			} else {
				panic!("#[derive(RlpEncodableWrapper)] is only defined for structs with one field.")
			}
		},
		syn::VariantData::Unit => panic!("#[derive(RlpEncodableWrapper)] is not defined for Unit structs."),
	};

	let name = &ast.ident;

	let dummy_const = syn::Ident::new(format!("_IMPL_RLP_ENCODABLE_FOR_{}", name));
	let impl_block = quote! {
		impl rlp::Encodable for #name {
			fn rlp_append(&self, stream: &mut rlp::RlpStream) {
				#stmt
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

fn encodable_field_map(tuple: (usize, &syn::Field)) -> quote::Tokens {
	encodable_field(tuple.0, tuple.1)
}

fn encodable_field(index: usize, field: &syn::Field) -> quote::Tokens {
	let ident = match field.ident {
		Some(ref ident) => ident.to_string(),
		None => index.to_string(),
	};

	let id = syn::Ident::new(format!("self.{}", ident));

	match field.ty {
		syn::Ty::Path(_, ref path) => {
			let top_segment = path.segments.first().expect("there must be at least 1 segment");
			let ident = &top_segment.ident;
			if &ident.to_string() == "Vec" {
				let inner_ident = match top_segment.parameters {
					syn::PathParameters::AngleBracketed(ref angle) => {
						let ty = angle.types.first().expect("Vec has only one angle bracketed type; qed");
						match *ty {
							syn::Ty::Path(_, ref path) => &path.segments.first().expect("there must be at least 1 segment").ident,
							_ => panic!("rlp_derive not supported"),
						}
					},
					_ => unreachable!("Vec has only one angle bracketed type; qed"),
				};
				quote! { stream.append_list::<#inner_ident, _>(&#id); }
			} else {
				quote! { stream.append(&#id); }
			}
		},
		_ => panic!("rlp_derive not supported"),
	}
}

