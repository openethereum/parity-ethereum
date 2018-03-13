use {syn, quote};

pub fn impl_encodable(ast: &syn::DeriveInput) -> quote::Tokens {
	let body = match ast.data {
		syn::Data::Struct(ref s) => s,
		_ => panic!("#[derive(RlpEncodable)] is only defined for structs."),
	};

	let stmts: Vec<_> = body.fields.iter().enumerate().map(encodable_field_map).collect();
	let name = &ast.ident;

	let stmts_len = stmts.len();
	let stmts_len = quote! { #stmts_len };
	let dummy_const: syn::Ident = format!("_IMPL_RLP_ENCODABLE_FOR_{}", name).into();
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
	let body = match ast.data {
		syn::Data::Struct(ref s) => s,
		_ => panic!("#[derive(RlpEncodableWrapper)] is only defined for structs."),
	};

	let stmt = {
		let fields: Vec<_> = body.fields.iter().collect();
		if fields.len() == 1 {
			let field = fields.first().expect("fields.len() == 1; qed");
			encodable_field(0, field)
		} else {
			panic!("#[derive(RlpEncodableWrapper)] is only defined for structs with one field.")
		}
	};

	let name = &ast.ident;

	let dummy_const: syn::Ident = format!("_IMPL_RLP_ENCODABLE_FOR_{}", name).into();
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
		Some(ref ident) => quote! { #ident },
		None => {
			let index: syn::Index = index.into();
			quote! { #index }
		}
	};

	let id = quote! { self.#ident };

	match field.ty {
		syn::Type::Path(ref path) => {
			let top_segment = path.path.segments.first().expect("there must be at least 1 segment");
			let ident = &top_segment.value().ident;
			if &ident.to_string() == "Vec" {
				let inner_ident = match top_segment.value().arguments {
					syn::PathArguments::AngleBracketed(ref angle) => {
						let ty = angle.args.first().expect("Vec has only one angle bracketed type; qed");
						match **ty.value() {
							syn::GenericArgument::Type(syn::Type::Path(ref path)) => &path.path.segments.first().expect("there must be at least 1 segment").value().ident,
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

