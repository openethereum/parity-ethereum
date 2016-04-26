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

const RESERVED_MESSAGE_IDS: u16 = 16;

pub fn expand_ipc_implementation(
	cx: &mut ExtCtxt,
	span: Span,
	meta_item: &MetaItem,
	annotatable: &Annotatable,
	push: &mut FnMut(Annotatable)
) {
	let item = match *annotatable {
		Annotatable::Item(ref item) => item,
		_ => {
			cx.span_err(meta_item.span, "`#[derive(Ipc)]` may only be applied to struct implementations");
			return;
		},
	};

	let builder = aster::AstBuilder::new().span(span);

	let interface_map = match implement_interface(cx, &builder, &item, push) {
		Ok(interface_map) => interface_map,
		Err(Error) => { return; },
	};

	push_client(cx, &builder, &interface_map, push);
	push_handshake_struct(cx, push);

	push(Annotatable::Item(interface_map.item));
}

fn push_handshake_struct(cx: &ExtCtxt, push: &mut FnMut(Annotatable)) {
	let handshake_item = quote_item!(cx,
		#[derive(Binary)]
		pub struct BinHandshake {
			api_version: String,
			protocol_version: String,
			reserved: Vec<u8>,
		}
	).unwrap();

	push(Annotatable::Item(handshake_item));
}

fn field_name(builder: &aster::AstBuilder, arg: &Arg) -> ast::Ident {
	match arg.pat.node {
		PatKind::Ident(_, ref ident, _) => builder.id(ident.node),
		_ => { panic!("unexpected param in interface: {:?}", arg.pat.node) }
	}
}

fn push_invoke_signature_aster(
	builder: &aster::AstBuilder,
	implement: &ImplItem,
	signature: &MethodSig,
	push: &mut FnMut(Annotatable),
) -> Dispatch {
	let inputs = &signature.decl.inputs;
	let (input_type_name, input_arg_names, input_arg_tys) = if inputs.len() > 0 {
		let first_field_name = field_name(builder, &inputs[0]).name.as_str();
		if first_field_name == "self" && inputs.len() == 1 { (None, vec![], vec![]) }
		else {
			let skip = if first_field_name == "self" { 2 } else { 1 };
			let name_str = format!("{}_input", implement.ident.name.as_str());

			let mut arg_names = Vec::new();
			let mut arg_tys = Vec::new();

			let arg_name = format!("{}", field_name(builder, &inputs[skip-1]).name);
			let arg_ty = &inputs[skip-1].ty;

			let mut tree = builder.item()
				.attr().word("derive(Binary)")
				.attr().word("allow(non_camel_case_types)")
				.struct_(name_str.as_str())
				.field(arg_name.as_str()).ty()
				.build(strip_ptr(arg_ty));

			arg_names.push(arg_name);
			arg_tys.push(arg_ty.clone());
			for arg in inputs.iter().skip(skip) {
				let arg_name = format!("{}", field_name(builder, &arg));
				let arg_ty = &arg.ty;

				tree = tree.field(arg_name.as_str()).ty().build(strip_ptr(arg_ty));
				arg_names.push(arg_name);
				arg_tys.push(arg_ty.clone());
			}

			push(Annotatable::Item(tree.build()));
			(Some(name_str.to_owned()), arg_names, arg_tys)
		}
	}
	else {
		(None, vec![], vec![])
	};

	let return_type_ty = match signature.decl.output {
		FunctionRetTy::Ty(ref ty) => {
			let name_str = format!("{}_output", implement.ident.name.as_str());
			let tree = builder.item()
				.attr().word("derive(Binary)")
				.attr().word("allow(non_camel_case_types)")
				.struct_(name_str.as_str())
				.field(format!("payload")).ty().build(ty.clone());
			push(Annotatable::Item(tree.build()));
			Some(ty.clone())
		}
		_ => None
	};

	Dispatch {
		function_name: format!("{}", implement.ident.name.as_str()),
		input_type_name: input_type_name,
		input_arg_names: input_arg_names,
		input_arg_tys: input_arg_tys,
		return_type_ty: return_type_ty,
	}
}

struct Dispatch {
	function_name: String,
	input_type_name: Option<String>,
	input_arg_names: Vec<String>,
	input_arg_tys: Vec<P<Ty>>,
	return_type_ty: Option<P<Ty>>,
}

//	This is the expanded version of this:
//
//	let invoke_serialize_stmt = quote_stmt!(cx, {
//		::bincode::serde::serialize(& $output_type_id { payload: self. $function_name ($hand_param_a, $hand_param_b) }, ::bincode::SizeLimit::Infinite).unwrap()
//  });
//
// But the above does not allow comma-separated expressions for arbitrary number
// of parameters ...$hand_param_a, $hand_param_b, ... $hand_param_n
fn implement_dispatch_arm_invoke_stmt(
	cx: &ExtCtxt,
	builder: &aster::AstBuilder,
	dispatch: &Dispatch,
) -> ast::Stmt
{
	let function_name = builder.id(dispatch.function_name.as_str());

	let input_args_exprs = dispatch.input_arg_names.iter().enumerate().map(|(arg_index, arg_name)| {
		let arg_ident = builder.id(arg_name);
		let expr = quote_expr!(cx, input. $arg_ident);
		if has_ptr(&dispatch.input_arg_tys[arg_index]) { quote_expr!(cx, & $expr) }
		else { expr }
	}).collect::<Vec<P<ast::Expr>>>();

	let ext_cx = &*cx;
	::quasi::parse_stmt_panic(&mut ::syntax::parse::new_parser_from_tts(
		ext_cx.parse_sess(),
		ext_cx.cfg(),
		{
			let _sp = ext_cx.call_site();
			let mut tt = ::std::vec::Vec::new();
			tt.push(::syntax::ast::TokenTree::Token(_sp, ::syntax::parse::token::OpenDelim(::syntax::parse::token::Brace)));
			tt.push(::syntax::ast::TokenTree::Token(_sp, ::syntax::parse::token::ModSep));
			tt.push(::syntax::ast::TokenTree::Token(_sp, ::syntax::parse::token::Ident(ext_cx.ident_of("ipc"), ::syntax::parse::token::ModName)));
			tt.push(::syntax::ast::TokenTree::Token(_sp, ::syntax::parse::token::ModSep));
			tt.push(::syntax::ast::TokenTree::Token(_sp, ::syntax::parse::token::Ident(ext_cx.ident_of("binary"), ::syntax::parse::token::ModName)));
			tt.push(::syntax::ast::TokenTree::Token(_sp, ::syntax::parse::token::ModSep));
			tt.push(::syntax::ast::TokenTree::Token(_sp, ::syntax::parse::token::Ident(ext_cx.ident_of("serialize"), ::syntax::parse::token::Plain)));
			tt.push(::syntax::ast::TokenTree::Token(_sp, ::syntax::parse::token::OpenDelim(::syntax::parse::token::Paren)));
			tt.push(::syntax::ast::TokenTree::Token(_sp, ::syntax::parse::token::BinOp(::syntax::parse::token::And)));
			tt.push(::syntax::ast::TokenTree::Token(_sp, ::syntax::parse::token::Ident(ext_cx.ident_of("self"), ::syntax::parse::token::Plain)));
			tt.push(::syntax::ast::TokenTree::Token(_sp, ::syntax::parse::token::Dot));
			tt.extend(::quasi::ToTokens::to_tokens(&function_name, ext_cx).into_iter());
			tt.push(::syntax::ast::TokenTree::Token(_sp, ::syntax::parse::token::OpenDelim(::syntax::parse::token::Paren)));

			for arg_expr in input_args_exprs {
				tt.extend(::quasi::ToTokens::to_tokens(&arg_expr, ext_cx).into_iter());
				tt.push(::syntax::ast::TokenTree::Token(_sp, ::syntax::parse::token::Comma));
			}

			tt.push(::syntax::ast::TokenTree::Token(_sp, ::syntax::parse::token::CloseDelim(::syntax::parse::token::Paren)));
			tt.push(::syntax::ast::TokenTree::Token(_sp, ::syntax::parse::token::CloseDelim(::syntax::parse::token::Paren)));
			tt.push(::syntax::ast::TokenTree::Token(_sp, ::syntax::parse::token::Dot));
			tt.push(::syntax::ast::TokenTree::Token(_sp, ::syntax::parse::token::Ident(ext_cx.ident_of("unwrap"), ::syntax::parse::token::Plain)));
			tt.push(::syntax::ast::TokenTree::Token(_sp, ::syntax::parse::token::OpenDelim(::syntax::parse::token::Paren)));
			tt.push(::syntax::ast::TokenTree::Token(_sp, ::syntax::parse::token::CloseDelim(::syntax::parse::token::Paren)));
			tt.push(::syntax::ast::TokenTree::Token(_sp, ::syntax::parse::token::CloseDelim(::syntax::parse::token::Brace)));
			tt
		})).unwrap()
}

fn implement_dispatch_arm_invoke(
	cx: &ExtCtxt,
	builder: &aster::AstBuilder,
	dispatch: &Dispatch,
	buffer: bool,
) -> P<ast::Expr>
{
	let deserialize_expr = if buffer {
		quote_expr!(cx, ::ipc::binary::deserialize(buf).expect("ipc deserialization error, aborting"))
	} else {
		quote_expr!(cx, ::ipc::binary::deserialize_from(r).expect("ipc deserialization error, aborting"))
	};

	let invoke_serialize_stmt = implement_dispatch_arm_invoke_stmt(cx, builder, dispatch);
	dispatch.input_type_name.as_ref().map(|val| {
			let input_type_id = builder.id(val.clone().as_str());
			quote_expr!(cx, {
				let input: $input_type_id = $deserialize_expr;
				$invoke_serialize_stmt
			})
		}).unwrap_or(quote_expr!(cx, { $invoke_serialize_stmt }))
}

/// generates dispatch match for method id
fn implement_dispatch_arm(
	cx: &ExtCtxt,
	builder: &aster::AstBuilder,
	index: u32,
	dispatch: &Dispatch,
	buffer: bool,
) -> ast::Arm
{
	let index_ident = builder.id(format!("{}", index + (RESERVED_MESSAGE_IDS as u32)).as_str());
	let invoke_expr = implement_dispatch_arm_invoke(cx, builder, dispatch, buffer);
	quote_arm!(cx, $index_ident => { $invoke_expr } )
}

fn implement_dispatch_arms(
	cx: &ExtCtxt,
	builder: &aster::AstBuilder,
	dispatches: &[Dispatch],
	buffer: bool,
) -> Vec<ast::Arm>
{
	let mut index = -1;
	dispatches.iter()
		.map(|dispatch| { index = index + 1; implement_dispatch_arm(cx, builder, index as u32, dispatch, buffer) }).collect()
}

pub fn strip_ptr(ty: &P<ast::Ty>) -> P<ast::Ty> {
	if let ast::TyKind::Rptr(_, ref ptr_mut) = ty.node {
		ptr_mut.ty.clone()
	}
	else { ty.clone() }
}

pub fn has_ptr(ty: &P<ast::Ty>) -> bool {
	if let ast::TyKind::Rptr(_, ref _ptr_mut) = ty.node {
		true
	}
	else { false }
}

/// returns an expression with the body for single operation that is being sent to server
/// operation itself serializes input, writes to socket and waits for socket to respond
/// (the latter only if original method signature returns anyting)
///
/// assuming expanded class contains method
///   fn commit(&self, f: u32) -> u32
///
/// the expanded implementation will generate method for the client like that
///    #[derive(Serialize)]
///    struct Request<'a> {
///	     f: &'a u32,
///    }
///    let payload = Request{f: &f,};
///    let mut socket_ref = self.socket.borrow_mut();
///    let mut socket = socket_ref.deref_mut();
///    let serialized_payload = ::bincode::serde::serialize(&payload, ::bincode::SizeLimit::Infinite).unwrap();
///    ::ipc::invoke(0, &Some(serialized_payload), &mut socket);
///    while !socket.ready().load(::std::sync::atomic::Ordering::Relaxed) { }
///    ::bincode::serde::deserialize_from::<_, u32>(&mut socket, ::bincode::SizeLimit::Infinite).unwrap()
fn implement_client_method_body(
	cx: &ExtCtxt,
	builder: &aster::AstBuilder,
	index: u16,
	interface_map: &InterfaceMap,
) -> P<ast::Expr>
{
	let dispatch = &interface_map.dispatches[index as usize];
	let index_ident = builder.id(format!("{}", index + RESERVED_MESSAGE_IDS).as_str());

	let request = if dispatch.input_arg_names.len() > 0 {

		let arg_name = dispatch.input_arg_names[0].as_str();
		let static_ty = strip_ptr(&dispatch.input_arg_tys[0]);
		let arg_ty = builder
			.ty().ref_()
			.lifetime("'a")
			.ty()
			.build(static_ty.clone());

		let mut tree = builder.item()
			.attr().word("derive(Binary)")
			.struct_("Request")
			.generics()
			.lifetime_name("'a")
			.build()
			.field(arg_name).ty()
			.build(arg_ty);

		for arg_idx in 1..dispatch.input_arg_names.len() {
			let arg_name = dispatch.input_arg_names[arg_idx].as_str();
			let static_ty = strip_ptr(&dispatch.input_arg_tys[arg_idx]);

			let arg_ty = builder
				.ty().ref_()
				.lifetime("'a")
				.ty()
				.build(static_ty);
			tree = tree.field(arg_name).ty().build(arg_ty);

		}
		let mut request_serialization_statements = Vec::new();

		let struct_tree = tree.build();
		let struct_stmt = quote_stmt!(cx, $struct_tree);
		request_serialization_statements.push(struct_stmt);

		// actually this is just expanded version of this:
		//   request_serialization_statements.push(quote_stmt!(cx, let payload = Request { p1: &p1, p2: &p2, ... pn: &pn, }));
		// again, cannot dynamically create expression with arbitrary number of comma-separated members
		request_serialization_statements.push({
			let ext_cx = &*cx;
			::quasi::parse_stmt_panic(&mut ::syntax::parse::new_parser_from_tts(
				ext_cx.parse_sess(),
				ext_cx.cfg(),
				{
					let _sp = ext_cx.call_site();
					let mut tt = ::std::vec::Vec::new();
					tt.push(::syntax::ast::TokenTree::Token(_sp, ::syntax::parse::token::Ident(ext_cx.ident_of("let"), ::syntax::parse::token::Plain)));
					tt.push(::syntax::ast::TokenTree::Token(_sp, ::syntax::parse::token::Ident(ext_cx.ident_of("payload"), ::syntax::parse::token::Plain)));
					tt.push(::syntax::ast::TokenTree::Token(_sp, ::syntax::parse::token::Eq));
					tt.push(::syntax::ast::TokenTree::Token(_sp, ::syntax::parse::token::Ident(ext_cx.ident_of("Request"), ::syntax::parse::token::Plain)));
					tt.push(::syntax::ast::TokenTree::Token(_sp, ::syntax::parse::token::OpenDelim(::syntax::parse::token::Brace)));

					for arg in dispatch.input_arg_names.iter() {
						tt.push(::syntax::ast::TokenTree::Token(_sp, ::syntax::parse::token::Ident(ext_cx.ident_of(arg.as_str()), ::syntax::parse::token::Plain)));
						tt.push(::syntax::ast::TokenTree::Token(_sp, ::syntax::parse::token::Colon));
						tt.push(::syntax::ast::TokenTree::Token(_sp, ::syntax::parse::token::BinOp(::syntax::parse::token::And)));

						tt.push(::syntax::ast::TokenTree::Token(_sp, ::syntax::parse::token::Ident(ext_cx.ident_of(arg.as_str()), ::syntax::parse::token::Plain)));
						tt.push(::syntax::ast::TokenTree::Token(_sp, ::syntax::parse::token::Comma));
					}

					tt.push(::syntax::ast::TokenTree::Token(_sp, ::syntax::parse::token::CloseDelim(::syntax::parse::token::Brace)));
					tt
				}))
			});

		request_serialization_statements.push(
			quote_stmt!(cx, let mut socket_ref = self.socket.borrow_mut()));

		request_serialization_statements.push(
			quote_stmt!(cx, let mut socket = socket_ref.deref_mut()));

		request_serialization_statements.push(
			quote_stmt!(cx, let serialized_payload = ::ipc::binary::serialize(&payload).unwrap()));

		request_serialization_statements.push(
			quote_stmt!(cx, ::ipc::invoke($index_ident, &Some(serialized_payload), &mut socket)));


		request_serialization_statements
	}
	else {
		let mut request_serialization_statements = Vec::new();
		request_serialization_statements.push(
			quote_stmt!(cx, let mut socket_ref = self.socket.borrow_mut()));
		request_serialization_statements.push(
			quote_stmt!(cx, let mut socket = socket_ref.deref_mut()));
		request_serialization_statements.push(
			quote_stmt!(cx, ::ipc::invoke($index_ident, Vec::new(), &mut socket)));
		request_serialization_statements
	};

	if let Some(ref return_ty) = dispatch.return_type_ty {
		let return_expr = quote_expr!(cx,
			::ipc::binary::deserialize_from::<$return_ty, _>(&mut socket).unwrap()
		);
		quote_expr!(cx, {
			$request
			$return_expr
		})
	}
	else {
		quote_expr!(cx, {
			$request
		})
	}
}

/// Generates signature and body (see `implement_client_method_body`)
/// for the client (signature is identical to the original method)
fn implement_client_method(
	cx: &ExtCtxt,
	builder: &aster::AstBuilder,
	index: u16,
	interface_map: &InterfaceMap,
)
	-> ast::ImplItem
{
	let dispatch = &interface_map.dispatches[index as usize];
	let method_name = builder.id(dispatch.function_name.as_str());
	let body = implement_client_method_body(cx, builder, index, interface_map);

	let ext_cx = &*cx;
	// expanded version of this
	//   pub fn $method_name(&self, p1: p1_ty, p2: p2_ty ... pn: pn_ty, ) [-> return_ty] { $body }
	// looks like it's tricky to build function declaration with aster if body already generated
	let signature = ::syntax::parse::parser::Parser::parse_impl_item(
		&mut ::syntax::parse::new_parser_from_tts(
			ext_cx.parse_sess(),
			ext_cx.cfg(),
			{
				let _sp = ext_cx.call_site();
				let mut tt = ::std::vec::Vec::new();
				tt.push(::syntax::ast::TokenTree::Token(_sp, ::syntax::parse::token::Ident(ext_cx.ident_of("fn"), ::syntax::parse::token::Plain)));
				tt.extend(::quasi::ToTokens::to_tokens(&method_name, ext_cx).into_iter());
				tt.push(::syntax::ast::TokenTree::Token(_sp, ::syntax::parse::token::OpenDelim(::syntax::parse::token::Paren)));
				tt.push(::syntax::ast::TokenTree::Token(_sp, ::syntax::parse::token::BinOp(::syntax::parse::token::And)));
				tt.push(::syntax::ast::TokenTree::Token(_sp, ::syntax::parse::token::Ident(ext_cx.ident_of("self"), ::syntax::parse::token::Plain)));
				tt.push(::syntax::ast::TokenTree::Token(_sp, ::syntax::parse::token::Comma));

				for arg_idx in 0..dispatch.input_arg_names.len() {
					let arg_name = dispatch.input_arg_names[arg_idx].as_str();
					let arg_ty = dispatch.input_arg_tys[arg_idx].clone();

					tt.push(::syntax::ast::TokenTree::Token(_sp, ::syntax::parse::token::Ident(ext_cx.ident_of(arg_name), ::syntax::parse::token::Plain)));
					tt.push(::syntax::ast::TokenTree::Token(_sp, ::syntax::parse::token::Colon));
					tt.extend(::quasi::ToTokens::to_tokens(&arg_ty, ext_cx).into_iter());
					tt.push(::syntax::ast::TokenTree::Token(_sp, ::syntax::parse::token::Comma));
				}
				tt.push(::syntax::ast::TokenTree::Token(_sp, ::syntax::parse::token::CloseDelim(::syntax::parse::token::Paren)));

				if let Some(ref return_ty) = dispatch.return_type_ty {
					tt.push(::syntax::ast::TokenTree::Token(_sp, ::syntax::parse::token::RArrow));
					tt.extend(::quasi::ToTokens::to_tokens(return_ty, ext_cx).into_iter());
				}

				tt.push(::syntax::ast::TokenTree::Token(_sp, ::syntax::parse::token::OpenDelim(::syntax::parse::token::Brace)));
				tt.extend(::quasi::ToTokens::to_tokens(&body, ext_cx).into_iter());
				tt.push(::syntax::ast::TokenTree::Token(_sp, ::syntax::parse::token::CloseDelim(::syntax::parse::token::Brace)));

				tt
			}));

	signature.unwrap()
}

fn client_generics(builder: &aster::AstBuilder, interface_map: &InterfaceMap) -> Generics {
	let ty_param = aster::ty_param::TyParamBuilder::new(
		builder.id("S")).trait_bound(
			builder.path().global().ids(&["ipc", "IpcSocket"]).build()
		).build().build();

	builder.from_generics(interface_map.generics.clone())
		.with_ty_param(ty_param)
		.build()
}

fn client_qualified_ident(builder: &aster::AstBuilder, interface_map: &InterfaceMap) -> P<Ty> {
	let generics = client_generics(builder, interface_map);
	aster::ty::TyBuilder::new().path().segment(interface_map.ident_map.client_ident(builder))
		.with_generics(generics).build()
		.build()
}

fn client_phantom_ident(builder: &aster::AstBuilder, interface_map: &InterfaceMap) -> P<Ty> {
	let generics = client_generics(builder, interface_map);
	aster::ty::TyBuilder::new().phantom_data()
		.tuple().with_tys(generics.ty_params.iter().map(|x| aster::ty::TyBuilder::new().id(x.ident)))
		.build()
}

/// generates client type for specified server type
/// for say `Service` it generates `ServiceClient`
fn push_client_struct(cx: &ExtCtxt, builder: &aster::AstBuilder, interface_map: &InterfaceMap, push: &mut FnMut(Annotatable)) {
	let generics = client_generics(builder, interface_map);
	let client_short_ident = interface_map.ident_map.client_ident(builder);
	let phantom = client_phantom_ident(builder, interface_map);

	let client_struct_item = quote_item!(cx,
		pub struct $client_short_ident $generics {
			socket: ::std::cell::RefCell<S>,
			phantom: $phantom,
		});

	push(Annotatable::Item(client_struct_item.expect(&format!("could not generate client struct for {:?}", client_short_ident.name))));
}

/// pushes generated code for the client class (type declaration and method invocation implementations)
fn push_client(
	cx: &ExtCtxt,
	builder: &aster::AstBuilder,
	interface_map: &InterfaceMap,
	push: &mut FnMut(Annotatable),
) {
	push_client_struct(cx, builder, interface_map, push);
	push_client_implementation(cx, builder, interface_map, push);
	push_with_socket_client_implementation(cx, builder, interface_map, push);
}


fn push_with_socket_client_implementation(
	cx: &ExtCtxt,
	builder: &aster::AstBuilder,
	interface_map: &InterfaceMap,
	push: &mut FnMut(Annotatable))
{
	let generics = client_generics(builder, interface_map);
	let client_ident = client_qualified_ident(builder, interface_map);
	let where_clause = &generics.where_clause;
	let client_short_ident = interface_map.ident_map.client_ident(builder);

	let implement = quote_item!(cx,
		impl $generics ::ipc::WithSocket<S> for $client_ident $where_clause {
			fn init(socket: S) -> $client_ident {
				$client_short_ident {
					socket: ::std::cell::RefCell::new(socket),
					phantom: ::std::marker::PhantomData,
				}
			}
		}).unwrap();
	push(Annotatable::Item(implement));
}

/// pushes full client side code for the original class exposed via ipc
fn push_client_implementation(
	cx: &ExtCtxt,
	builder: &aster::AstBuilder,
	interface_map: &InterfaceMap,
	push: &mut FnMut(Annotatable),
) {
	let item_ident = interface_map.ident_map.qualified_ident(builder);

	let mut index = -1i32;
	let items = interface_map.dispatches.iter()
		.map(|_| { index = index + 1; P(implement_client_method(cx, builder, index as u16, interface_map)) })
		.collect::<Vec<P<ast::ImplItem>>>();

	let generics = client_generics(builder, interface_map);
	let client_ident = client_qualified_ident(builder, interface_map);
	let where_clause = &generics.where_clause;

	let handshake_item = quote_impl_item!(cx,
		pub fn handshake(&self) -> Result<(), ::ipc::Error> {
			let payload = BinHandshake {
				protocol_version: $item_ident::protocol_version().to_string(),
				api_version: $item_ident::api_version().to_string(),
				reserved: vec![0u8; 64],
			};

			let mut socket_ref = self.socket.borrow_mut();
			let mut socket = socket_ref.deref_mut();
			::ipc::invoke(
				0,
				&Some(::ipc::binary::serialize(&payload).unwrap()),
				&mut socket);

			let mut result = vec![0u8; 1];
			if try!(socket.read(&mut result).map_err(|_| ::ipc::Error::HandshakeFailed)) == 1 {
				match result[0] {
					1 => Ok(()),
					_ => Err(::ipc::Error::RemoteServiceUnsupported),
				}
			}
			else { Err(::ipc::Error::HandshakeFailed) }
		}).unwrap();

	let socket_item = quote_impl_item!(cx,
		#[cfg(test)]
		pub fn socket(&self) -> &::std::cell::RefCell<S> {
			&self.socket
		}).unwrap();

	let generic_items = vec![P(handshake_item), P(socket_item)];

	if interface_map.impl_trait.is_some() {
		let trait_ty = builder.id(
			::syntax::print::pprust::path_to_string(
				&interface_map.impl_trait.as_ref().unwrap().path));

		let implement_trait =
			quote_item!(cx,
				impl $generics $trait_ty for $client_ident $where_clause {
					$items
				}
			).unwrap();
		push(Annotatable::Item(implement_trait));

		let implement =
			quote_item!(cx,
				impl $generics $client_ident $where_clause {
					$generic_items
				}
			).unwrap();
		push(Annotatable::Item(implement));
	}
	else {
		let pub_items = items.iter().map(|item| {
			let pub_item = item.clone();
			pub_item.map(|mut val| { val.vis = ast::Visibility::Public; val })
		}).collect::<Vec<P<ast::ImplItem>>>();

		let implement = quote_item!(cx,
			impl $generics $client_ident $where_clause {
				$pub_items
				$generic_items
			}).unwrap();
		push(Annotatable::Item(implement));
	}

}

/// implements dispatching of system handshake invocation (method_num 0)
fn implement_handshake_arm(
	cx: &ExtCtxt,
) -> (ast::Arm, ast::Arm)
{
	let handshake_deserialize = quote_stmt!(&cx,
		let handshake_payload = ::ipc::binary::deserialize_from::<BinHandshake, _>(r).unwrap();
	);

	let handshake_deserialize_buf = quote_stmt!(&cx,
		let handshake_payload = ::ipc::binary::deserialize::<BinHandshake>(buf).unwrap();
	);

	let handshake_serialize = quote_expr!(&cx,
		::ipc::binary::serialize::<bool>(&Self::handshake(&::ipc::Handshake {
			api_version: ::semver::Version::parse(&handshake_payload.api_version).unwrap(),
			protocol_version: ::semver::Version::parse(&handshake_payload.protocol_version).unwrap(),
		})).unwrap()
	);

	(
		quote_arm!(&cx, 0 => {
			$handshake_deserialize
			$handshake_serialize
		}),
		quote_arm!(&cx, 0 => {
			$handshake_deserialize_buf
			$handshake_serialize
		}),
	)
}


struct InterfaceMap {
	pub original_item: Item,
	pub item: P<ast::Item>,
	pub dispatches: Vec<Dispatch>,
	pub generics: Generics,
	pub impl_trait: Option<TraitRef>,
	pub ident_map: IdentMap,
}

struct IdentMap {
	original_path: ast::Path,
}

impl IdentMap {
	fn ident(&self, builder: &aster::AstBuilder) -> Ident {
		builder.id(format!("{}", ::syntax::print::pprust::path_to_string(&self.original_path)))
	}

	fn client_ident(&self, builder: &aster::AstBuilder) -> Ident {
		builder.id(format!("{}Client", self.original_path.segments[0].identifier))
	}

	fn qualified_ident(&self, builder: &aster::AstBuilder) -> Ident {
		builder.id(format!("{}", ::syntax::print::pprust::path_to_string(&self.original_path).replace("<", "::<")))
	}
}

fn ty_ident_map(original_ty: &P<Ty>) -> IdentMap {
	let original_path = match original_ty.node {
		::syntax::ast::TyKind::Path(_, ref path) => path.clone(),
		_ => { panic!("incompatible implementation"); }
	};
	let ident_map = IdentMap { original_path: original_path };
	ident_map
}

/// implements `IpcInterface<C>` for the given class `C`
fn implement_interface(
	cx: &ExtCtxt,
	builder: &aster::AstBuilder,
	item: &Item,
	push: &mut FnMut(Annotatable),
) -> Result<InterfaceMap, Error> {
	let (generics, impl_trait, original_ty, impl_items) = match item.node {
		ast::ItemKind::Impl(_, _, ref generics, ref impl_trait, ref ty, ref impl_items) => (generics, impl_trait, ty, impl_items),
		_ => {
			cx.span_err(
				item.span,
				"`#[derive(Ipc)]` may only be applied to item implementations");
			return Err(Error);
		},
	};
	let impl_generics = builder.from_generics(generics.clone()).build();
	let where_clause = &impl_generics.where_clause;

	let mut method_signatures = Vec::new();
	for impl_item in impl_items {
		if let ImplItemKind::Method(ref signature, _) = impl_item.node {
			method_signatures.push((impl_item, signature))
		}
	}

	let dispatch_table = method_signatures.iter().map(|&(impl_item, signature)|
			push_invoke_signature_aster(builder, impl_item, signature, push))
		.collect::<Vec<Dispatch>>();

	let dispatch_arms = implement_dispatch_arms(cx, builder, &dispatch_table, false);
	let dispatch_arms_buffered = implement_dispatch_arms(cx, builder, &dispatch_table, true);

	let (handshake_arm, handshake_arm_buf) = implement_handshake_arm(cx);

	let ty = ty_ident_map(&original_ty).ident(builder);
	let ipc_item = quote_item!(cx,
		impl $impl_generics ::ipc::IpcInterface<$ty> for $ty $where_clause {
			fn dispatch<R>(&self, r: &mut R) -> Vec<u8>
				where R: ::std::io::Read
			{
				let mut method_num = vec![0u8;2];
				match r.read(&mut method_num) {
					Ok(size) if size == 0 => { panic!("method id not supplied" ); }
					Err(e) => { panic!("ipc read error: {:?}, aborting", e); }
					_ => { }
				}

				// method_num is a 16-bit little-endian unsigned number
				match method_num[1] as u16 + (method_num[0] as u16)*256 {
					// handshake
					$handshake_arm
					// user methods
					$dispatch_arms
					_ => vec![]
				}
			}

			fn dispatch_buf(&self, method_num: u16, buf: &[u8]) -> Vec<u8>
			{
				match method_num {
					$handshake_arm_buf
					$dispatch_arms_buffered
					_ => vec![]
				}
			}
		}
	).unwrap();

	Ok(InterfaceMap {
		ident_map: ty_ident_map(&original_ty),
		original_item: item.clone(),
		item: ipc_item,
		dispatches: dispatch_table,
		generics: generics.clone(),
		impl_trait: impl_trait.clone(),
	})
}
