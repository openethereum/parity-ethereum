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
	Ty,
	TyKind,
	Path,
	DUMMY_NODE_ID,
};

use syntax::ast;
use syntax::ext::base::{Annotatable, ExtCtxt};
use syntax::ext::build::AstBuilder;
use syntax::ptr::P;

use std::collections::{HashMap, HashSet};
use std::ops::Deref;

fn is_new_entry(path: &Path) -> Option<String> {
	let known = {
		if path.segments.len() > 1 {
			false
		} else {
			let ident = format!("{}", path.segments[0].identifier.name.as_str());
			ident == "u8"        ||
			ident == "i8"        ||
			ident == "u16"       ||
			ident == "i16"       ||
			ident == "u32"       ||
			ident == "u64"       ||
			ident == "usize"     ||
			ident == "i32"		 ||
			ident == "i64"       ||
			ident == "String"    ||
			ident == "bool"
		}
	};

	if known { None }
	else { Some(::syntax::print::pprust::path_to_string(path)) }
}

pub fn argument_replacement(
	builder: &aster::AstBuilder,
	replacements: &HashMap<String, P<Ty>>,
	ty: &P<Ty>,
) -> Option<P<Ty>> {
	match ty.node {
		TyKind::Vec(ref nested_ty) => {
			argument_replacement(builder, replacements, nested_ty).and_then(|replaced_with| {
				let mut inplace_ty = nested_ty.deref().clone();
				inplace_ty.node = TyKind::Vec(replaced_with);
				inplace_ty.id = DUMMY_NODE_ID;
				Some(P(inplace_ty))
			})
		},
		TyKind::FixedLengthVec(ref nested_ty, ref len_expr) => {
			argument_replacement(builder, replacements, nested_ty).and_then(|replaced_with| {
				let mut inplace_ty = nested_ty.deref().clone();
				inplace_ty.node = TyKind::FixedLengthVec(replaced_with, len_expr.clone());
				inplace_ty.id = DUMMY_NODE_ID;
				Some(P(inplace_ty))
			})
		},
		TyKind::Path(_, ref path) => {
			if path.segments.len() > 0 && path.segments[0].identifier.name.as_str() == "Option" ||
				path.segments[0].identifier.name.as_str() == "Result" {

				let nested_ty = &path.segments[0].parameters.types()[0];
				argument_replacement(builder, replacements, nested_ty).and_then(|replaced_with| {
					let mut inplace_path = path.clone();
					match inplace_path.segments[0].parameters {
						ast::PathParameters::AngleBracketed(ref mut data) => {
							data.types = data.types.map(|_| replaced_with.clone());
						},
						_ => {}
					}
					let mut inplace_ty = nested_ty.deref().deref().clone();
					inplace_ty.node = TyKind::Path(None, inplace_path);
					inplace_ty.id = DUMMY_NODE_ID;
					Some(P(inplace_ty))
				})
			}
			else {
				replacements.get(&::syntax::print::pprust::path_to_string(path)).and_then(|replaced_with| {
					Some(replaced_with.clone())
				})
			}
		}
		_ => { None }
	}
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
				let &$ident(ref val) = self;
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
				if path.segments.len() > 0 && {
					let first_segment = path.segments[0].identifier.name.as_str();
					first_segment == "Option" || first_segment  == "Result" || first_segment == "Vec"
				}
				{
					let extra_type = &path.segments[0].parameters.types()[0];
					if !stop_list.contains(extra_type) {
						fringe.push(extra_type);
					}
					continue;
				}

				match is_new_entry(path) {
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
			_ => { }
		}
	}

	hash_map
}
