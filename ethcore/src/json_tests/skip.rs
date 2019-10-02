// Copyright 2015-2019 Parity Technologies (UK) Ltd.
// This file is part of Parity Ethereum.

// Parity Ethereum is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity Ethereum is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity Ethereum.  If not, see <http://www.gnu.org/licenses/>.

//! State or blockchain tests to skip.
//!
//! Looks in the `ethereum/tests/test-issues/currents.json` file. This file contains two
//! collections, `block` and `state`, each with a different format to specify single tests to skip.
//!
//! To skip a blockchain test, add a JSON object to the `block` array, where `failing` names the
//! leaf folder with the tests to skip. The `subtests` array contains the names of the tests to skip.
//! Note that this does not handle duplicate folder names, e.g. `ValidBlocks/funTests/` and
//! `Something/funTests` would both be matched when `failing` is set to `funTests`.
//!
//! To skip a state test, add a JSON object to the `state` array. The `failing` works like for block
//! tests, but the `subtests` key is an object on the form:
//! "testName": {"subnumbers": [INDEX_OF_SKIPPED_SUBTESTS | "*"], "chain": "Blockchain name (informational)"}`
//!
//! Use the `reference` key to point to the github issue tracking to solution to the problem.
//!
//! Note: the `declare_test!` macro can also be use to skip tests, but skips entire files rather
//! than single tests.

use ethjson::test_helpers::skip::SkipTests;

lazy_static! {
	pub static ref SKIP_TESTS: SkipTests = {
		let skip_data = include_bytes!("../../res/ethereum/tests-issues/currents.json");
		SkipTests::load(&skip_data[..]).expect("JSON from disk is valid")
	};
}
