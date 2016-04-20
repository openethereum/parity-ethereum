// Copyright 2015, 2016 Ethcore (UK) Ltd.
// This file is part of Parity.

// Parity is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity.  If not, see <http://www.gnu.org/licenses/>.
use aster;

use syntax::ast::{
	MetaItem,
	Item,
	ImplItemKind,
	ImplItem,
	MethodSig,
	Arg,
	PatKind,
	FunctionRetTy,
	Ty,
	TraitRef,
	Ident,
	Generics,
};

use syntax::ast;
use syntax::codemap::Span;
use syntax::ext::base::{Annotatable, ExtCtxt};
use syntax::ext::build::AstBuilder;
use syntax::ptr::P;

pub struct Error;

pub fn expand_serialization_implementation(
	cx: &mut ExtCtxt,
	span: Span,
	meta_item: &MetaItem,
	annotatable: &Annotatable,
	push: &mut FnMut(Annotatable)
) {
	let item = match *annotatable {
		Annotatable::Item(ref item) => item,
		_ => {
			cx.span_err(meta_item.span, "`#[derive(Binary)]` may only be applied to structs and enums");
			return;
		}
	};

	let builder = aster::AstBuilder::new().span(span);

    let impl_item = match serialize_item(cx, &builder, &item) {
        Ok(item) => item,
        Err(Error) => {
            // An error occured, but it should have been reported already.
            return;
        }
    };

    push(Annotatable::Item(impl_item))
}

fn serialize_item(
    cx: &ExtCtxt,
    builder: &aster::AstBuilder,
    item: &Item,
) -> Result<P<ast::Item>, Error> {
	let generics = match item.node {
		ast::ItemKind::Struct(_, ref generics) => generics,
		ast::ItemKind::Enum(_, ref generics) => generics,
		_ => {
			cx.span_err(
				item.span,
				"`#[derive(Serialize)]` may only be applied to structs and enums");
			return Err(Error);
		}
	};

	let impl_generics = build_impl_generics(cx, builder, item, generics);

	let ty = builder.ty().path()
		.segment(item.ident).with_generics(impl_generics.clone()).build()
		.build();

	let where_clause = &impl_generics.where_clause;

	let binary_expressions = try!(binary_expr(cx,
		&builder,
		&item,
		&impl_generics,
		ty.clone()));

	Ok(quote_item!(cx,
		impl $impl_generics ::ipc::BinaryConvertable for $ty $where_clause {
			fn size(&self) -> usize {
				$size_expr
			}

			fn to_bytes(buffer: &mut [u8]) {
				$write_expr
			}

			fn from_bytes(buffer: &[u8]) -> Self {
				$create_expr
			}
        }
    ).unwrap())
}

fn binary_expr(
    cx: &ExtCtxt,
    builder: &aster::AstBuilder,
    item: &Item,
    impl_generics: &ast::Generics,
    ty: P<ast::Ty>,
) -> Result<BinaryExpressions, Error> {
	match item.node {
		ast::ItemKind::Struct(ref variant_data, _) => {
			binary_expr_item_struct(
				cx,
				builder,
				impl_generics,
				ty,
				item.span,
				variant_data,
			)
		}
		ast::ItemKind::Enum(ref enum_def, _) => {
			binary_expr_enum(
				cx,
				builder,
				item.ident,
				impl_generics,
				ty,
				enum_def,
			)
		}
		_ => {
			cx.span_bug(item.span,
						"expected ItemStruct or ItemEnum in #[derive(Serialize)]");
		}
	}
}

struct BinaryExpressions {
	pub size: P<ast::Expr>,
	pub write: P<ast::Stmt>,
	pub read: P<ast::Expr>,
}

fn binary_expr_struct(
	cx: &ExtCtxt,
	builder: &aster::AstBuilder,
	ty: P<ast::Ty>,
    fields: &[ast::StructField],
	value_ident: Option<P<ast::Ident>>,
) -> Result<BinaryExpressions, Error> {
    let type_name = field.ty;
	let id = field.id;

	let mut size_exprs = Vec::new();
	fields.iter().enumerate(|(index, field)| {
		let index_ident = builder.id(format!("{}", index));
		value_ident.and_then(|value_ident| size_exprs.push(quote_expr!(cx, $value_ident . $index_ident .size())))
			.unwrap_or_else(|| size_exprs.push(quote_expr!(cx, $index_ident .size())));
	});

	let mut total_size_expr = size_exprs[0].clone();
	for index in 1..size_expr.len() {
		let next_expr = size_exprs[i].clone();
		total_size_expr = quote_expr!(cx, $total_size_expr + $next_expr);
	}

	let mut write_stmts = Vec::new();
	write_stmts.push(quote_stmt!(cx, let mut offset = 0usize;));
	fields.iter().enumerate(|(index, field)| {
		let index_ident = builder.id(format!("{}", index));
		let size_expr = size_exprs[index];
		write_stmts.push(quote_stmt!(cx, let next_line = offset + $size_expr; ));
		value_ident.and_then(|x|
				write_stmts.push(quote_stmt!(cx, $x . $index_ident .write(&mut buffer[offset..next_line]);))
			).unwrap_or_else(|| write_stmts.push(quote_stmt!(cx, $index_ident .write(&mut buffer[offset..next_line]);)));
		write_stmts.push(quote_stmt!(cx, offset = next_line; ));
	});

    Ok(BinaryExpressions {
		size: total_size_expr,
		write: quote_stmt!(cx, { $write_stmts } ),
		read: quote_expr!(cx, $ty { }),
	});
}

fn binary_expr_item_struct(
	cx: &ExtCtxt,
	builder: &aster::AstBuilder,
	impl_generics: &ast::Generics,
	ty: P<ast::Ty>,
	span: Span,
	variant_data: &ast::VariantData,
) -> Result<BinaryExpressions, Error> {
	match *variant_data {
		ast::VariantData::Tuple(ref fields, _) => {
			binary_expr_struct(
				cx,
				&builder,
				ty,
				fields,
			)
		}
		ast::VariantData::Struct(ref fields, _) => {
			binary_expr_struct(
				cx,
				&builder,
				ty,
				fields,
			)
		}
	}
}

fn binary_expr_enum(
    cx: &ExtCtxt,
    builder: &aster::AstBuilder,
    type_ident: Ident,
    impl_generics: &ast::Generics,
    ty: P<ast::Ty>,
    enum_def: &ast::EnumDef,
) -> Result<BinaryExpressions, Error> {
	let arms: Vec<_> = try!(
		enum_def.variants.iter()
			.enumerate()
			.map(|(variant_index, variant)| {
				binary_expr_variant(
					cx,
					builder,
					type_ident,
					impl_generics,
					ty.clone(),
					variant,
					variant_index,
				)
			})
			.collect()
	);

	let (size_arms, write_arms, read_arms) = (
		arms.iter().map(|x| x.size).collect::<Vec<P<ast::Arm>>>(),
		arms.iter().map(|x| x.write).collect::<Vec<P<ast::Arm>>>(),
		arms.iter().map(|x| x.read).collect::<Vec<P<ast::Arm>>>());

	Ok(BinaryExpressions {
		size: quote_expr!(cx, match *self { $size_arms }),
		write: quote_stmt!(cx, match *self { $write_arms }; ),
		read: quote_expr!(cx, match *self { $read_arms }),
	});

}

struct BinaryArm {
	size: P<ast::Arm>,
	write: P<ast::Arm>,
	read: P<ast::Arm>,
}

fn binary_expr_variant(
    cx: &ExtCtxt,
    builder: &aster::AstBuilder,
    type_ident: Ident,
    generics: &ast::Generics,
    ty: P<ast::Ty>,
    variant: &ast::Variant,
    variant_index: usize,
) -> Result<BinaryArm, Error> {
	let type_name = ty.name;

	let variant_ident = variant.node.name;

	match variant.node.data {
		ast::VariantData::Tuple(ref fields, _) => {
			let field_names: Vec<ast::Ident> = (0 .. fields.len())
				.map(|i| builder.id(format!("__field{}", i)))
				.collect();

			let pat = builder.pat().enum_()
				.id(type_ident).id(variant_ident).build()
				.with_pats(
					field_names.iter()
						.map(|field| builder.pat().ref_id(field))
				)
				.build();

			let binary_expr = try!(binary_expr_struct(
				cx,
				&builder,
				ty,
				fields,
				None,
			));

			let (size_expr, write, read_expr) = (binary_expr.size, vec![binary_expr.write], binary_expr.read);

			Ok(BinaryArm {
				size: P(quote_arm!(cx, $pat => { $size_expr } )),
				write: P(quote_arm!(cx, $pat => { $write } )),
				read: P(quote_arm!(cx, $pat => { $read_expr } )),
			})
		}
		ast::VariantData::Struct(ref fields, _) => {
			let field_names: Vec<_> = (0 .. fields.len())
				.map(|i| builder.id(format!("__field{}", i)))
				.collect();

			let pat = builder.pat().struct_()
				.id(type_ident).id(variant_ident).build()
				.with_pats(
					field_names.iter()
						.zip(fields.iter())
						.map(|(id, field)|(name, builder.pat().ref_id(id))))
				.build();


			let binary_expr = try!(binary_expr_struct(
				cx,
				&builder,
				ty,
				fields,
				None,
			));

			let (size_expr, write, read_expr) = (binary_expr.size, vec![binary_expr.write], binary_expr.read);
			Ok(BinaryArm {
				size: P(quote_arm!(cx, $pat => { $size_expr } )),
				write: P(quote_arm!(cx, $pat => { $write } )),
				read: P(quote_arm!(cx, $pat => { $read_expr } )),
			})
		}
	}
}
