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
	TyKind,
	Path,
};

use syntax::ast;
use syntax::codemap::Span;
use syntax::ext::base::{Annotatable, ExtCtxt};
use syntax::ext::build::AstBuilder;
use syntax::ptr::P;

use std::collections::{HashMap, HashSet};
use std::ops::Deref;

fn is_new_entry(builder: &aster::AstBuilder, path: &Path) -> Option<String> {
	let known = {
		if path.segments.len() > 1 {
			false
		} else {
			let ident = format!("{}", path.segments[0].identifier.name.as_str());

			ident == "u32"       ||
			ident == "u64"       ||
			ident == "usize"     ||
			ident == "i32"		 ||
			ident == "i64"       ||
			ident == "String"    ||
			ident == "Option"
		}
	};

	if known { None }
	else { Some(path_str(path)) }
}

fn path_str(path: &Path) -> String {
	let mut res: String = "_".to_owned();
	for segment in path.segments.iter() {
		res.push_str(&format!("{}_", segment.identifier.name.as_str()));
	}
	res
}

pub fn push_bin_box(
	cx: &ExtCtxt,
	builder: &aster::AstBuilder,
	ty: &Ty,
	bbox_name: &str,
	push: &mut FnMut(Annotatable),
) {
	let ident = builder.id(bbox_name);
	let bin_box_struct = quote_item!(cx,
		struct $ident ($ty);
	).unwrap();
	push(Annotatable::Item(bin_box_struct));
	push(Annotatable::Item(quote_item!(cx,
		impl From<$ty> for $ident {
			fn from(val: $ty) -> $ident {
				$ident(val)
			}
		}).unwrap()));

	push(Annotatable::Item(quote_item!(cx,
		impl Into<$ty> for $ident {
			fn into(self) -> $ty {
				let $ident(val) = self;
				val
			}
		}).unwrap()));


	let serialize_impl = quote_item!(cx,
		impl ::serde::ser::Serialize for $ident {
			fn serialize<__S>(&self, _serializer: &mut __S) -> ::std::result::Result<(), __S::Error>
				where __S: ::serde::ser::Serializer
			{
				let &$ident(val) = self;
				_serializer.serialize_bytes(val.as_slice())
			}
	 	}).unwrap();

	let ident_expr = builder.id(::syntax::print::pprust::ty_to_string(ty));

	let deserialize_impl = quote_item!(cx,
		impl ::serde::de::Deserialize for $ident {
    		fn deserialize<__D>(deserializer: &mut __D) -> ::std::result::Result<$ident, __D::Error>
				where __D: ::serde::de::Deserializer
			{
				struct __Visitor<__D: ::serde::de::Deserializer>(::std::marker::PhantomData<__D>);

				impl <__D: ::serde::de::Deserializer> ::serde::de::Visitor for __Visitor<__D> {
					type Value = $ident;
					#[inline]
					fn visit_seq<__V>(&mut self, mut visitor: __V) -> ::std::result::Result<$ident, __V::Error>
						where __V: ::serde::de::SeqVisitor
					{
						let raw_bytes: Vec<u8> = try!(visitor.visit()).unwrap_or_else(|| Vec::new());
						let inner = $ident_expr ::from_bytes(&raw_bytes).unwrap();
						Ok($ident (inner))
					}

				}
            	deserializer.deserialize_bytes(__Visitor::<__D>(::std::marker::PhantomData))
			}

		}).unwrap();

	push(Annotatable::Item(serialize_impl));
	push(Annotatable::Item(deserialize_impl));
}

pub fn match_unknown_tys(
	cx: &ExtCtxt,
	builder: &aster::AstBuilder,
	tys: &[P<Ty>],
	push: &mut FnMut(Annotatable),
) -> HashMap<String, P<Ty>>
{
	let mut hash_map = HashMap::new();
	let mut fringe = Vec::new();
	fringe.extend(tys);
	let mut stop_list = HashSet::new();
	let mut index = 0;

	loop {
		if fringe.len() == 0 { break; }
		let drained = fringe.drain(..1).collect::<Vec<&P<Ty>>>();
		let ty = drained[0];
		stop_list.insert(ty);

		match ty.node {
			TyKind::Vec(ref nested_ty) => {
				if !stop_list.contains(nested_ty) {
					fringe.push(nested_ty);
				}
			},
			TyKind::FixedLengthVec(ref nested_ty, _) => {
				if !stop_list.contains(nested_ty) {
					fringe.push(nested_ty);
				}
			},
			TyKind::Path(_, ref path) => {
				if path.segments.len() > 0 && path.segments[0].identifier.name.as_str() == "Option" ||
					path.segments[0].identifier.name.as_str() == "Result" {
					for extra_type in path.segments[0].parameters.types() {
						if !stop_list.contains(extra_type) {
							fringe.push(extra_type);
						}
					}
					continue;
				}

				match is_new_entry(builder, path) {
					Some(old_path) => {
						if hash_map.get(&old_path).is_some() {
							continue;
						}

						let bin_box_name = format!("BinBox{}", index);
						push_bin_box(cx, builder, &ty, &bin_box_name, push);
						hash_map.insert(old_path, builder.ty().id(&bin_box_name));
						index = index + 1;
					},
					None => {}
				}
			},
			_ => { panic!("bad parameter in input args: {:?}", ty) }
		}
	}

	hash_map
}
