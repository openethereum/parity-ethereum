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

import { toChecksumAddress } from './address';

export function asU32 (slice) {
  // TODO: validation

  return new BigNumber(slice, 16);
}

export function asI32 (slice) {
  if (new BigNumber(slice.substr(0, 1), 16).toString(2)[0] === '1') {
    return new BigNumber(slice, 16)
      .minus(new BigNumber('ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff', 16))
      .minus(1);
  }

  return new BigNumber(slice, 16);
}

export function asAddress (slice) {
  // TODO: address validation?

  return toChecksumAddress(`0x${slice.slice(-40)}`);
}

export function asBool (slice) {
  // TODO: everything else should be 0

  return new BigNumber(slice[63]).eq(1);
}
