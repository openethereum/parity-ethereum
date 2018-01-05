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

import BigNumber from 'bignumber.js';
import { isFunction, isInstanceOf } from '@parity/api/lib/util/types';
import { isAddress } from '@parity/abi/lib/util/address';

export { isAddress, isFunction, isInstanceOf };

const ZEROS = '000000000000000000000000000000000000000000000000000000000000';

export function isBigNumber (test) {
  return isInstanceOf(test, BigNumber);
}

export function isBoolean (test) {
  return Object.prototype.toString.call(test) === '[object Boolean]';
}

export function isHexNumber (_test) {
  if (_test.substr(0, 2) !== '0x') {
    return false;
  }

  const test = _test.substr(2);
  const hex = `${ZEROS}${(new BigNumber(_test, 16)).toString(16)}`.slice(-1 * test.length);

  return hex === test;
}
