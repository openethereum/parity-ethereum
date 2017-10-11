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

import { range } from 'lodash';

export function bytesToHex (bytes) {
  return '0x' + Buffer.from(bytes).toString('hex');
}

export function cleanupValue (value, type) {
  // TODO: make work with arbitrary depth arrays
  if (value instanceof Array && type.match(/bytes[0-9]+/)) {
    // figure out if it's an ASCII string hiding in there:
    let ascii = '';

    for (let index = 0, ended = false; index < value.length && ascii !== null; ++index) {
      const val = value[index];

      if (val === 0) {
        ended = true;
      } else {
        ascii += String.fromCharCode(val);
      }

      if ((ended && val !== 0) || (!ended && (val < 32 || val >= 128))) {
        ascii = null;
      }
    }

    value = ascii === null
      ? bytesToHex(value)
      : ascii;
  }

  if (type.substr(0, 4) === 'uint' && +type.substr(4) <= 48) {
    value = +value;
  }

  return value;
}

export function hexToBytes (hex) {
  const raw = toHex(hex).slice(2);
  const bytes = [];

  for (let i = 0; i < raw.length; i += 2) {
    bytes.push(parseInt(raw.substr(i, 2), 16));
  }

  return bytes;
}

export function hexToAscii (hex) {
  const bytes = hexToBytes(hex);
  const str = bytes.map((byte) => String.fromCharCode(byte)).join('');

  return str;
}

export function bytesToAscii (bytes) {
  return bytes.map((b) => String.fromCharCode(b % 512)).join('');
}

export function asciiToHex (string) {
  let result = '0x';

  for (let i = 0; i < string.length; ++i) {
    result += ('0' + string.charCodeAt(i).toString(16)).substr(-2);
  }

  return result;
}

export function padRight (input, length) {
  const value = toHex(input).substr(2, length * 2);

  return '0x' + value + range(length * 2 - value.length).map(() => '0').join('');
}

export function padLeft (input, length) {
  const value = toHex(input).substr(2, length * 2);

  return '0x' + range(length * 2 - value.length).map(() => '0').join('') + value;
}

export function toHex (str) {
  if (str && str.toString) {
    str = str.toString(16);
  }

  if (str && str.substr(0, 2) === '0x') {
    return str.toLowerCase();
  }

  return `0x${(str || '').toLowerCase()}`;
}
