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

//! Panic utilities

use std::thread;
use std::panic;
use std::sync::Mutex;
use std::any::Any;
use std::ops::DerefMut;

pub trait OnPanicListener<T>: Send + Sync + 'static {
	fn call(&mut self, arg: &T);
}

impl<F, T> OnPanicListener<T> for F
	where F: FnMut(&T) + Send + Sync + 'static {
	fn call(&mut self, arg: &T) {
		self(arg)
	}
}

pub trait ArgsConverter<T> {
	fn convert(&self, t: &Box<Any + Send>) -> Option<T>;
}

pub trait MayPanic<T> {
	fn on_panic<F>(&mut self, closure: F)
		where F: OnPanicListener<T>;
}

pub trait PanicHandler<T, C: ArgsConverter<T>> : MayPanic<T>{
	fn new(converter: C) -> Self;
	fn catch_panic<G, R>(&mut self, g: G) -> thread::Result<R>
		where G: FnOnce() -> R + panic::RecoverSafe;
}


pub struct StringConverter;
impl ArgsConverter<String> for StringConverter {
	fn convert(&self, t: &Box<Any + Send>) -> Option<String> {
		t.downcast_ref::<&'static str>().map(|t| t.clone().to_owned())
	}
}

pub struct BasePanicHandler<T, C>
	where C: ArgsConverter<T>, T: 'static {
	converter: C,
	listeners: Mutex<Vec<Box<OnPanicListener<T>>>>
}

impl<T, C> BasePanicHandler<T, C>
	where C: ArgsConverter<T>, T: 'static {
	fn notify_all(&mut self, res: Option<T>) {
		if let None = res {
			return;
		}
		let r = res.unwrap();
		let mut listeners = self.listeners.lock().unwrap();
		for listener in listeners.deref_mut() {
			listener.call(&r);
		}
	}
}

impl<T, C> PanicHandler<T, C> for BasePanicHandler<T, C>
	where C: ArgsConverter<T>, T: 'static {

	fn new(converter: C) -> Self {
		BasePanicHandler {
			converter: converter,
			listeners: Mutex::new(vec![])
		}
	}

	fn catch_panic<G, R>(&mut self, g: G) -> thread::Result<R>
		where G: FnOnce() -> R + panic::RecoverSafe {
			let result = panic::recover(g);

			println!("After calling function");
			if let Err(ref e) = result {
				let res = self.converter.convert(e);
				println!("Got error. Notifying");
				self.notify_all(res);
			}

			result
		}
}

impl<T, C> MayPanic<T> for BasePanicHandler<T, C>
	where C: ArgsConverter<T>, T: 'static {
	fn on_panic<F>(&mut self, closure: F)
		where F: OnPanicListener<T> {
		self.listeners.lock().unwrap().push(Box::new(closure));
	}
}

#[test]
fn should_notify_listeners_about_panic () {
	use std::sync::{Arc, RwLock};

	// given
	let invocations = Arc::new(RwLock::new(vec![]));
	let i = invocations.clone();
	let mut p = BasePanicHandler::new(StringConverter);
	p.on_panic(move |t: &String| i.write().unwrap().push(t.clone()));

	// when
	p.catch_panic(|| panic!("Panic!"));

	// then
	assert!(invocations.read().unwrap()[0] == "Panic!");
}

#[test]
fn should_notify_listeners_about_panic_in_other_thread () {
	use std::thread;
	use std::sync::{Arc, RwLock};

	// given
	let invocations = Arc::new(RwLock::new(vec![]));
	let i = invocations.clone();
	let mut p = BasePanicHandler::new(StringConverter);
	p.on_panic(move |t: &String| i.write().unwrap().push(t.clone()));

	// when
	let t = thread::spawn(move ||
		p.catch_panic(|| panic!("Panic!"))
	);
	t.join();

	// then
	assert!(invocations.read().unwrap()[0] == "Panic!");
}
