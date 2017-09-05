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

import { keccak_256 } from 'js-sha3'; // eslint-disable-line

import { hexToBytes } from './format';
import { isHex } from './types';

export function sha3 (value, options) {
  const forceHex = options && options.encoding === 'hex';

  if (forceHex || (!options && isHex(value))) {
    const bytes = hexToBytes(value);

    return sha3(bytes);
  }

  const hash = keccak_256(value);

  return `0x${hash}`;
}

sha3.text = (val) => sha3(val, { encoding: 'raw' });
