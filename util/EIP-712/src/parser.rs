// Copyright 2015-2018 Parity Technologies (UK) Ltd.
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

//! Solidity type-name parsing
use lunarity_lexer::{Lexer, Token};
use crate::error::*;
use toolshed::Arena;
use std::{fmt, result};

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
	Address,
	Uint,
	Int,
	String,
	Bool,
	Bytes,
	Byte(u8),
	Custom(String),
	Array {
		length: Option<u64>,
		inner: Box<Type>
	}
}

impl From<Type> for String {
	fn from(field_type: Type) -> String {
		match field_type {
			Type::Address => "address".into(),
			Type::Uint => "uint".into(),
			Type::Int => "int".into(),
			Type::String => "string".into(),
			Type::Bool => "bool".into(),
			Type::Bytes => "bytes".into(),
			Type::Byte(len) => format!("bytes{}", len),
			Type::Custom(custom) => custom,
			Type::Array {
				inner,
				length
			} => {
				let inner: String = (*inner).into();
				match length {
					None => format!("{}[]", inner),
					Some(length) => format!("{}[{}]", inner, length)
				}
			}
		}
	}
}

impl fmt::Display for Type {
	fn fmt(&self, f: &mut fmt::Formatter) -> result::Result<(), fmt::Error> {
		let item: String = self.clone().into();
		write!(f, "{}", item)
	}
}

pub struct Parser {
	arena: Arena,
}

impl Parser {
	pub fn new() -> Self {
		Parser {
			arena: Arena::new()
		}
	}

	/// the type string is being validated before it's parsed.
	pub fn parse_type(&self, field_type: &str) -> Result<Type> {
		#[derive(PartialEq)]
		enum State { Open, Close }

		let mut lexer = Lexer::new(&self.arena, field_type);
		let mut token = None;
		let mut state = State::Close;
		let mut array_depth = 0;
		let mut current_array_length: Option<u64> = None;

		while lexer.token != Token::EndOfProgram  {
			let type_ = match lexer.token {
				Token::Identifier => Type::Custom(lexer.token_as_str().to_owned()),
				Token::TypeByte => Type::Byte(lexer.type_size.0),
				Token::TypeBytes => Type::Bytes,
				Token::TypeBool => Type::Bool,
				Token::TypeUint => Type::Uint,
				Token::TypeInt => Type::Int,
				Token::TypeString => Type::String,
				Token::TypeAddress => Type::Address,
				Token::LiteralInteger => {
					let length = lexer.token_as_str();
					current_array_length = Some(length
						.parse()
						.map_err(|_|
							ErrorKind::InvalidArraySize(length.into())
						)?
					);
					lexer.consume();
					continue;
				},
				Token::BracketOpen if token.is_some() && state == State::Close => {
					state = State::Open;
					lexer.consume();
					continue
				}
				Token::BracketClose if array_depth < 10 => {
					if state == State::Open && token.is_some() {
						let length = current_array_length.take();
						state = State::Close;
						token = Some(Type::Array {
							inner: Box::new(token.expect("if statement checks for some; qed")),
							length
						});
						lexer.consume();
						array_depth += 1;
						continue
					} else {
						return Err(ErrorKind::UnexpectedToken(lexer.token_as_str().to_owned(), field_type.to_owned()))?
					}
				}
				Token::BracketClose if array_depth == 10 => {
					return Err(ErrorKind::UnsupportedArrayDepth)?
				}
				_  => return Err(ErrorKind::UnexpectedToken(lexer.token_as_str().to_owned(), field_type.to_owned()))?
			};

			token = Some(type_);
			lexer.consume();
		}

		Ok(token.ok_or_else(|| ErrorKind::NonExistentType)?)
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_parser() {
		let parser = Parser::new();
		let source = "byte[][][7][][][][][][][]";
		parser.parse_type(source).unwrap();
	}

	#[test]
	fn test_nested_array() {
		let parser = Parser::new();
		let source = "byte[][][7][][][][][][][][]";
		assert_eq!(parser.parse_type(source).is_err(), true);
	}

	#[test]
	fn test_malformed_array_type() {
		let parser = Parser::new();
		let source = "byte[7[]uint][]";
		assert_eq!(parser.parse_type(source).is_err(), true)
	}
}
