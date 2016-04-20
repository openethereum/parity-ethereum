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
			binary_expr_struct(
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
	size: P<ast::Expr>,
	write: P<ast::Stmt>,
	read: P<ast::Expr>,
}

fn serialize_tuple_struct(
	cx: &ExtCtxt,
	builder: &aster::AstBuilder,
	impl_generics: &ast::Generics,
	ty: P<ast::Ty>,
	fields: usize,
) -> Result<P<ast::Expr>, Error> {
    let type_name = field.ty;
	let id = field.id;

    Ok(BinaryExpressions {
		size: quote_expr!(cx, self. $id .size() ),
		write: quote_stmt!(cx, self. $id .write(buffer); ),
		read: quote_expr!(cx, Self { $id: $type_name ::from_bytes(buffer) }),
	});
}

fn binary_expr_struct(
	cx: &ExtCtxt,
	builder: &aster::AstBuilder,
	impl_generics: &ast::Generics,
	ty: P<ast::Ty>,
	span: Span,
	variant_data: &ast::VariantData,
) -> Result<BinaryExpressions, Error> {
	match *variant_data {
		ast::VariantData::Tuple(ref fields, _) => {
			binary_expr_struct_tuple(
				cx,
				&builder,
				impl_generics,
				ty,
				fields,
			)
		}
		ast::VariantData::Struct(ref fields, _) => {
			binary_expr_struct_inner(
				cx,
				&builder,
				impl_generics,
				ty,
				fields,
			)
		}
	}
}
