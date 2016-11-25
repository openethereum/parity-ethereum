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

import { range } from 'lodash';
import { inHex } from '../format/input';

export function bytesToHex (bytes) {
  return '0x' + bytes.map((b) => ('0' + b.toString(16)).slice(-2)).join('');
}

export function hex2Ascii (_hex) {
  const hex = /^(?:0x)?(.*)$/.exec(_hex.toString())[1];

  let str = '';

  for (let i = 0; i < hex.length; i += 2) {
    str += String.fromCharCode(parseInt(hex.substr(i, 2), 16));
  }

  return str;
}

export function asciiToHex (string) {
  return '0x' + string.split('').map((s) => s.charCodeAt(0).toString(16)).join('');
}

export function padRight (input, length) {
  const value = inHex(input).substr(2, length * 2);
  return '0x' + value + range(length * 2 - value.length).map(() => '0').join('');
}

export function padLeft (input, length) {
  const value = inHex(input).substr(2, length * 2);
  return '0x' + range(length * 2 - value.length).map(() => '0').join('') + value;
}
