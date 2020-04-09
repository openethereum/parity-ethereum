//! Wrapper for user-provided WASM engine.

use engine::Engine;
use machine::Machine;
use wasmtime::*;

pub struct Wasm {
	machine: Machine,
	store: Store,
	instance: Instance,
}

impl Wasm {
	pub fn new(
		wabt_data: &str,
		machine: Machine,
	) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
		// A `Store` is a sort of "global object" in a sense, but for now it suffices
		// to say that it's generally passed to most constructors.
		let store = Store::default();

		// We start off by creating a `Module` which represents a compiled form
		// of our input wasm module. In this case it'll be JIT-compiled after
		// we parse the text format.
		let module = Module::new(&store, &wabt_data)?;

		// After we have a compiled `Module` we can then instantiate it, creating
		// an `Instance` which we can actually poke at functions on.
		let instance = Instance::new(&module, &[])?;

		Self {
			machine,
			store,
			instance,
		}
	}
}

impl Engine for Wasm {
	fn name(&self) -> &str {
		"WASM engine"
	}

	fn machine(&self) -> &Machine {
		&self.machine
	}

	fn seal_fields(&self, header: &Header) -> usize {
		self.instance
			.get_export("seal_fields")
			.unwrap()
			.func()
			.unwrap()
			.call(&[Val::I64(header.number())])
			.unwrap()[0]
			.unwrap_v128() as usize
	}
}
