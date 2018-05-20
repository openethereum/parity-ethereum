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

use std::fmt;
use instructions;

/// Stack trait with VM-friendly API
pub trait Stack<T> {
	/// Returns `Stack[len(Stack) - no_from_top]`
	fn peek(&self, no_from_top: usize) -> &T;
	/// Swaps Stack[len(Stack)] and Stack[len(Stack) - no_from_top]
	fn swap_with_top(&mut self, no_from_top: usize);
	/// Returns true if Stack has at least `no_of_elems` elements
	fn has(&self, no_of_elems: usize) -> bool;
	/// Get element from top and remove it from Stack. Panics if stack is empty.
	fn pop_back(&mut self) -> T;
	/// Get (up to `instructions::MAX_NO_OF_TOPICS`) elements from top and remove them from Stack. Panics if stack is empty.
	fn pop_n(&mut self, no_of_elems: usize) -> &[T];
	/// Add element on top of the Stack
	fn push(&mut self, elem: T);
	/// Get number of elements on Stack
	fn size(&self) -> usize;
	/// Returns all data on stack.
	fn peek_top(&self, no_of_elems: usize) -> &[T];
}

pub struct VecStack<S> {
	stack: Vec<S>,
	logs: [S; instructions::MAX_NO_OF_TOPICS]
}

impl<S : Copy> VecStack<S> {
	pub fn with_capacity(capacity: usize, zero: S) -> Self {
		VecStack {
			stack: Vec::with_capacity(capacity),
			logs: [zero; instructions::MAX_NO_OF_TOPICS]
		}
	}
}

impl<S : fmt::Display> Stack<S> for VecStack<S> {
	fn peek(&self, no_from_top: usize) -> &S {
		&self.stack[self.stack.len() - no_from_top - 1]
	}

	fn swap_with_top(&mut self, no_from_top: usize) {
		let len = self.stack.len();
		self.stack.swap(len - no_from_top - 1, len - 1);
	}

	fn has(&self, no_of_elems: usize) -> bool {
		self.stack.len() >= no_of_elems
	}

	fn pop_back(&mut self) -> S {
		let val = self.stack.pop();
		match val {
			Some(x) => x,
			None => panic!("Tried to pop from empty stack.")
		}
	}

	fn pop_n(&mut self, no_of_elems: usize) -> &[S] {
		assert!(no_of_elems <= instructions::MAX_NO_OF_TOPICS);

		for i in 0..no_of_elems {
			self.logs[i] = self.pop_back();
		}
		&self.logs[0..no_of_elems]
	}

	fn push(&mut self, elem: S) {
		self.stack.push(elem);
	}

	fn size(&self) -> usize {
		self.stack.len()
	}

	fn peek_top(&self, no_from_top: usize) -> &[S] {
		assert!(self.stack.len() >= no_from_top, "peek_top asked for more items than exist.");
		&self.stack[self.stack.len() - no_from_top .. self.stack.len()]
	}
}
