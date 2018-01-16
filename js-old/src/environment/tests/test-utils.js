// Copyright 2015-2017 Parity Technologies (UK) Ltd.
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

import { Component } from 'react';

const isProd = process.env.NODE_ENV === 'production';

// Component utils for integration tests hooks.
const TEST_HOOK = 'data-test';

Component.prototype._test = isProd ? noop : testHook;
Component.prototype._testInherit = isProd ? noop : testHookInherit;

function noop (name) {}

function testHookInherit (name) {
  let hook = this.props[TEST_HOOK];

  if (name) {
    hook += `-${name}`;
  }
  return {
    [TEST_HOOK]: hook
  };
}

function testHook (name) {
  let hook = this.constructor.name;

  if (name) {
    hook += `-${name}`;
  }
  return {
    [TEST_HOOK]: hook
  };
}
