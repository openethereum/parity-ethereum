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
};

use syntax::ast;
use syntax::codemap::Span;
use syntax::ext::base::{Annotatable, ExtCtxt};
use syntax::ext::build::AstBuilder;
use syntax::ptr::P;

pub struct Error;

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
		}
	};

	let builder = aster::AstBuilder::new().span(span);

	let (impl_item, dispatches) = match implement_interface(cx, &builder, &item, push) {
		Ok((item, dispatches)) => (item, dispatches),
		Err(Error) => { return; }
	};

	push_proxy(cx, &builder, &item, &dispatches, push);

	push(Annotatable::Item(impl_item))
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
			let arg_ty = inputs[skip-1].ty.clone();
			let mut tree = builder.item()
				.attr().word("derive(Serialize, Deserialize)")
				.attr().word("allow(non_camel_case_types)")
				.struct_(name_str.as_str())
				.field(arg_name.as_str()).ty().build(arg_ty.clone());
			arg_names.push(arg_name);
			arg_tys.push(arg_ty.clone());
			for arg in inputs.iter().skip(skip) {
				let arg_name = format!("{}", field_name(builder, &arg));
				let arg_ty = arg.ty.clone();
				tree = tree.field(arg_name.as_str()).ty().build(arg_ty.clone());
				arg_names.push(arg_name);
				arg_tys.push(arg_ty);
			}

			push(Annotatable::Item(tree.build()));
			(Some(name_str.to_owned()), arg_names, arg_tys)
		}
	}
	else {
		(None, vec![], vec![])
	};

	let (return_type_name, return_type_ty) = match signature.decl.output {
		FunctionRetTy::Ty(ref ty) => {
			let name_str = format!("{}_output", implement.ident.name.as_str());
			let tree = builder.item()
				.attr().word("derive(Serialize, Deserialize)")
				.attr().word("allow(non_camel_case_types)")
				.struct_(name_str.as_str())
				.field(format!("payload")).ty().build(ty.clone());
			push(Annotatable::Item(tree.build()));
			(Some(name_str.to_owned()), Some(ty.clone()))
		}
		_ => (None, None)
	};

	Dispatch {
		function_name: format!("{}", implement.ident.name.as_str()),
		input_type_name: input_type_name,
		input_arg_names: input_arg_names,
		input_arg_tys: input_arg_tys,
		return_type_name: return_type_name,
		return_type_ty: return_type_ty,
	}
}

struct Dispatch {
	function_name: String,
	input_type_name: Option<String>,
	input_arg_names: Vec<String>,
	input_arg_tys: Vec<P<Ty>>,
	return_type_name: Option<String>,
	return_type_ty: Option<P<Ty>>,
}

fn implement_dispatch_arm_invoke(
	cx: &ExtCtxt,
	builder: &aster::AstBuilder,
	dispatch: &Dispatch,
) -> P<ast::Expr>
{
	let deserialize_expr = quote_expr!(cx, ::bincode::serde::deserialize_from(r, ::bincode::SizeLimit::Infinite).expect("ipc deserialization error, aborting"));
	let input_type_id = builder.id(dispatch.input_type_name.clone().unwrap().as_str());
	let function_name = builder.id(dispatch.function_name.as_str());
	let output_type_id = builder.id(dispatch.return_type_name.clone().unwrap().as_str());

	let input_args_exprs = dispatch.input_arg_names.iter().map(|ref arg_name| {
		let arg_ident = builder.id(arg_name);
		quote_expr!(cx, input. $arg_ident)
	}).collect::<Vec<P<ast::Expr>>>();

	//	This is the expanded version of this:
	//
	//	let invoke_serialize_stmt = quote_stmt!(cx, {
	//		::bincode::serde::serialize(& $output_type_id { payload: self. $function_name ($hand_param_a, $hand_param_b) }, ::bincode::SizeLimit::Infinite).unwrap()
	//  });
	//
	// But the above does not allow comma-separated expressions for arbitrary number
	// of parameters ...$hand_param_a, $hand_param_b, ... $hand_param_n
	let invoke_serialize_stmt = {
		let ext_cx = &*cx;
		::quasi::parse_stmt_panic(&mut ::syntax::parse::new_parser_from_tts(
			ext_cx.parse_sess(),
			ext_cx.cfg(),
			{
				let _sp = ext_cx.call_site();
				let mut tt = ::std::vec::Vec::new();
				tt.push(::syntax::ast::TokenTree::Token(_sp, ::syntax::parse::token::OpenDelim(::syntax::parse::token::Brace)));
				tt.push(::syntax::ast::TokenTree::Token(_sp, ::syntax::parse::token::ModSep));
				tt.push(::syntax::ast::TokenTree::Token(_sp, ::syntax::parse::token::Ident(ext_cx.ident_of("bincode"), ::syntax::parse::token::ModName)));
				tt.push(::syntax::ast::TokenTree::Token(_sp, ::syntax::parse::token::ModSep));
				tt.push(::syntax::ast::TokenTree::Token(_sp, ::syntax::parse::token::Ident(ext_cx.ident_of("serde"), ::syntax::parse::token::ModName)));
				tt.push(::syntax::ast::TokenTree::Token(_sp, ::syntax::parse::token::ModSep));
				tt.push(::syntax::ast::TokenTree::Token(_sp, ::syntax::parse::token::Ident(ext_cx.ident_of("serialize"), ::syntax::parse::token::Plain)));
				tt.push(::syntax::ast::TokenTree::Token(_sp, ::syntax::parse::token::OpenDelim(::syntax::parse::token::Paren)));
				tt.push(::syntax::ast::TokenTree::Token(_sp, ::syntax::parse::token::BinOp(::syntax::parse::token::And)));
				tt.extend(::quasi::ToTokens::to_tokens(&output_type_id, ext_cx).into_iter());
				tt.push(::syntax::ast::TokenTree::Token(_sp, ::syntax::parse::token::OpenDelim(::syntax::parse::token::Brace)));
				tt.push(::syntax::ast::TokenTree::Token(_sp, ::syntax::parse::token::Ident(ext_cx.ident_of("payload"), ::syntax::parse::token::Plain)));
				tt.push(::syntax::ast::TokenTree::Token(_sp, ::syntax::parse::token::Colon));
				tt.push(::syntax::ast::TokenTree::Token(_sp, ::syntax::parse::token::Ident(ext_cx.ident_of("self"), ::syntax::parse::token::Plain)));
				tt.push(::syntax::ast::TokenTree::Token(_sp, ::syntax::parse::token::Dot));
				tt.extend(::quasi::ToTokens::to_tokens(&function_name, ext_cx).into_iter());
				tt.push(::syntax::ast::TokenTree::Token(_sp, ::syntax::parse::token::OpenDelim(::syntax::parse::token::Paren)));

				for arg_expr in input_args_exprs {
					tt.extend(::quasi::ToTokens::to_tokens(&arg_expr, ext_cx).into_iter());
					tt.push(::syntax::ast::TokenTree::Token(_sp, ::syntax::parse::token::Comma));
				}

				tt.push(::syntax::ast::TokenTree::Token(_sp, ::syntax::parse::token::CloseDelim(::syntax::parse::token::Paren)));
				tt.push(::syntax::ast::TokenTree::Token(_sp, ::syntax::parse::token::CloseDelim(::syntax::parse::token::Brace)));
				tt.push(::syntax::ast::TokenTree::Token(_sp, ::syntax::parse::token::Comma));
				tt.push(::syntax::ast::TokenTree::Token(_sp, ::syntax::parse::token::ModSep));
				tt.push(::syntax::ast::TokenTree::Token(_sp, ::syntax::parse::token::Ident(ext_cx.ident_of("bincode"), ::syntax::parse::token::ModName)));
				tt.push(::syntax::ast::TokenTree::Token(_sp, ::syntax::parse::token::ModSep));
				tt.push(::syntax::ast::TokenTree::Token(_sp, ::syntax::parse::token::Ident(ext_cx.ident_of("SizeLimit"), ::syntax::parse::token::ModName)));
				tt.push(::syntax::ast::TokenTree::Token(_sp, ::syntax::parse::token::ModSep));
				tt.push(::syntax::ast::TokenTree::Token(_sp, ::syntax::parse::token::Ident(ext_cx.ident_of("Infinite"), ::syntax::parse::token::Plain)));
				tt.push(::syntax::ast::TokenTree::Token(_sp, ::syntax::parse::token::CloseDelim(::syntax::parse::token::Paren)));
				tt.push(::syntax::ast::TokenTree::Token(_sp, ::syntax::parse::token::Dot));
				tt.push(::syntax::ast::TokenTree::Token(_sp, ::syntax::parse::token::Ident(ext_cx.ident_of("unwrap"), ::syntax::parse::token::Plain)));
				tt.push(::syntax::ast::TokenTree::Token(_sp, ::syntax::parse::token::OpenDelim(::syntax::parse::token::Paren)));
				tt.push(::syntax::ast::TokenTree::Token(_sp, ::syntax::parse::token::CloseDelim(::syntax::parse::token::Paren)));
				tt.push(::syntax::ast::TokenTree::Token(_sp, ::syntax::parse::token::CloseDelim(::syntax::parse::token::Brace)));
				tt
			}))
	};
	quote_expr!(cx, {
		let input: $input_type_id = $deserialize_expr;
		$invoke_serialize_stmt
	})
}

fn implement_dispatch_arm(cx: &ExtCtxt, builder: &aster::AstBuilder, index: u32, dispatch: &Dispatch)
	-> ast::Arm
{
	let index_ident = builder.id(format!("{}", index).as_str());
	let invoke_expr = implement_dispatch_arm_invoke(cx, builder, dispatch);
	quote_arm!(cx, $index_ident => { $invoke_expr } )
}

fn push_proxy_struct(cx: &ExtCtxt, builder: &aster::AstBuilder, item: &Item, push: &mut FnMut(Annotatable)) {
	let proxy_ident = builder.id(format!("{}Proxy", item.ident.name.as_str()));

	let proxy_struct_item = quote_item!(cx,
		pub struct $proxy_ident <S: ::ipc::IpcSocket> {
			socket: ::std::cell::RefCell<S>,
			phantom: ::std::marker::PhantomData<S>,
		});

	push(Annotatable::Item(proxy_struct_item.expect(&format!("could not generate proxy struct for {:?}", proxy_ident.name))));
}

fn push_proxy(
	cx: &ExtCtxt,
	builder: &aster::AstBuilder,
	item: &Item,
	dispatches: &[Dispatch],
	push: &mut FnMut(Annotatable))
{
	push_proxy_struct(cx, builder, item, push);
	push_proxy_implementation(cx, builder, dispatches, push);
}

fn implement_proxy_method_body(
	cx: &ExtCtxt,
	builder: &aster::AstBuilder,
	dispatch: &Dispatch)
	-> P<ast::Expr>
{
	let request = if dispatch.input_arg_names.len() > 0 {
		let arg_name = dispatch.input_arg_names[0].as_str();
		let arg_ty = builder
			.ty().ref_()
			.lifetime("'a")
			.ty().build(dispatch.input_arg_tys[0].clone());

		let mut tree = builder.item()
			.attr().word("derive(Serialize)")
			.struct_("Request")
			.generics()
			.lifetime_name("'a")
			.build()
			.field(arg_name).ty().build(arg_ty);

		for arg_idx in 1..dispatch.input_arg_names.len() {
			let arg_name = dispatch.input_arg_names[arg_idx].as_str();
			let arg_ty = builder
				.ty().ref_()
				.lifetime("'a")
				.ty().build(dispatch.input_arg_tys[arg_idx].clone());
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
			quote_stmt!(cx, let serialized_payload = ::bincode::serde::serialize(&payload, ::bincode::SizeLimit::Infinite).unwrap()));

		request_serialization_statements.push(
			quote_stmt!(cx, ::ipc::invoke(0, &Some(serialized_payload), &mut socket)));


		request_serialization_statements
	}
	else {
		vec![]
	};

	let wait_result_stmt = quote_stmt!(cx, while !socket.ready().load(::std::sync::atomic::Ordering::Relaxed) { });
	if let Some(ref return_ty) = dispatch.return_type_ty {
		let return_expr = quote_expr!(cx,
			::bincode::serde::deserialize_from::<_, $return_ty>(&mut socket, ::bincode::SizeLimit::Infinite).unwrap()
		);
		quote_expr!(cx, {
			$request
			$wait_result_stmt
			$return_expr
		})
	}
	else {
		quote_expr!(cx, {
			$request
			$wait_result_stmt
		})
	}
}

fn implement_proxy_method(
	cx: &ExtCtxt,
	builder: &aster::AstBuilder,
	dispatch: &Dispatch)
	-> ast::ImplItem
{
	let method_name = builder.id(dispatch.function_name.as_str());
	let body = implement_proxy_method_body(cx, builder, dispatch);

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
				tt.push(::syntax::ast::TokenTree::Token(_sp, ::syntax::parse::token::Ident(ext_cx.ident_of("pub"), ::syntax::parse::token::Plain)));
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

fn push_proxy_implementation(
	cx: &ExtCtxt,
	builder: &aster::AstBuilder,
	dispatches: &[Dispatch],
	push: &mut FnMut(Annotatable))
{
	let items = dispatches.iter()
		.map(|dispatch| P(implement_proxy_method(cx, builder, dispatch)))
		.collect::<Vec<P<ast::ImplItem>>>();

	let implement = quote_item!(cx,
		impl<S> ServiceProxy<S> where S: ::ipc::IpcSocket {
			pub fn new(socket: S) -> ServiceProxy<S> {
				ServiceProxy {
					socket: ::std::cell::RefCell::new(socket),
					phantom: ::std::marker::PhantomData,
				}
			}

			#[cfg(test)]
			pub fn socket(&self) -> &::std::cell::RefCell<S> {
				&self.socket
			}

			$items
		}).unwrap();

	push(Annotatable::Item(implement));
}

fn implement_interface(
	cx: &ExtCtxt,
	builder: &aster::AstBuilder,
	item: &Item,
	push: &mut FnMut(Annotatable),
) -> Result<(P<ast::Item>, Vec<Dispatch>), Error> {
	let (generics, impl_items) = match item.node {
		ast::ItemKind::Impl(_, _, ref generics, _, _, ref impl_items) => (generics, impl_items),
		_ => {
			cx.span_err(
				item.span,
				"`#[derive(Ipc)]` may only be applied to item implementations");
			return Err(Error);
		}
	};

	let impl_generics = builder.from_generics(generics.clone())
		.add_ty_param_bound(
			builder.path().global().ids(&["ethcore_ipc"]).build()
		)
		.build();

	let ty = builder.ty().path()
		.segment(item.ident).with_generics(impl_generics.clone()).build()
		.build();

	let where_clause = &impl_generics.where_clause;

	let mut dispatch_table = Vec::new();
	for impl_item in impl_items {
		if let ImplItemKind::Method(ref signature, _) = impl_item.node {
			dispatch_table.push(push_invoke_signature_aster(builder, &impl_item, signature, push));
		}
	}
	let mut index = -1;
	let dispatch_arms: Vec<_> = dispatch_table.iter()
		.map(|dispatch| { index = index + 1; implement_dispatch_arm(cx, builder, index as u32, dispatch) }).collect();

	Ok((quote_item!(cx,
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
				match method_num[0] as u16 + (method_num[1] as u16)*256 {
					$dispatch_arms
					_ => vec![]
				}
			}
		}
	).unwrap(), dispatch_table))
}
