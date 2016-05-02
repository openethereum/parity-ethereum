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

//! benchmarking for EVM
//! should be started with:
//! ```bash
//! multirust run nightly cargo bench
//! ```

extern crate test;

use self::test::{Bencher, black_box};

use common::*;
use super::{Factory, VMType};
use super::tests::FakeExt;

#[bench]
fn mem_gas_calculation_same(b: &mut Bencher) {
	let vm = Factory::new(VMType::Interpreter).create();

	let address = Address::from_str("0f572e5295c57f15886f9b263e2f6d2d6c7b5ec6").unwrap();
	let mut ext = FakeExt::new();

	b.iter(|| {
		let n = black_box(0xff);

		let code = format!(
			"6110006001556001546000555b61{0:04x}805560016000540380600055600c57", n
		).from_hex().unwrap();

		let mut params = ActionParams::default();
		params.address = address.clone();
		params.gas = !U256::zero(); // to infinity and beyond!
		params.code = Some(code.clone());

		vm.exec(params, &mut ext).unwrap();
	});
}

#[bench]
fn mem_gas_calculation_increasing(b: &mut Bencher) {
	let vm = Factory::new(VMType::Interpreter).create();

	let address = Address::from_str("0f572e5295c57f15886f9b263e2f6d2d6c7b5ec6").unwrap();
	let mut ext = FakeExt::new();

	b.iter(|| {
		let n = black_box(0xff);

		let code = format!(
			"6110006001556001546000555b61{0:04x}60005401805560016000540380600055600c57", n
		).from_hex().unwrap();

		let mut params = ActionParams::default();
		params.address = address.clone();
		params.gas = !U256::zero(); // to infinity and beyond!
		params.code = Some(code.clone());

		vm.exec(params, &mut ext).unwrap();
	});
}

