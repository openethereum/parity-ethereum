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

import rlp from 'rlp';

export function decodeExtraData (str) {
  try {
    // Try decoding as RLP
    const decoded = rlp.decode(str);
    const v = decoded[0];

    decoded[0] = decoded[1];
    decoded[1] = `${v[0]}.${v[1]}.${v[2]}`;
    return decoded.join('/');
  } catch (err) {
    // hex -> str
    return str.match(/.{1,2}/g).map(v => {
      return String.fromCharCode(parseInt(v, 16));
    }).join('');
  }
}
