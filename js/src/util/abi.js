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

const ARRAY_TYPE = 'ARRAY_TYPE';
const ADDRESS_TYPE = 'ADDRESS_TYPE';
const STRING_TYPE = 'STRING_TYPE';
const BOOL_TYPE = 'BOOL_TYPE';
const BYTES_TYPE = 'BYTES_TYPE';
const INT_TYPE = 'INT_TYPE';
const FIXED_TYPE = 'FIXED_TYPE';

export const ABI_TYPES = {
  ARRAY: ARRAY_TYPE, ADDRESS: ADDRESS_TYPE,
  STRING: STRING_TYPE, BOOL: BOOL_TYPE,
  BYTES: BYTES_TYPE, INT: INT_TYPE,
  FIXED: FIXED_TYPE
};

export function parseAbiType (type) {
  const arrayRegex = /^(.+)\[(\d*)]$/;

  if (arrayRegex.test(type)) {
    const matches = arrayRegex.exec(type);

    const subtype = parseAbiType(matches[1]);
    const M = parseInt(matches[2]) || null;
    const defaultValue = !M
      ? []
      : range(M).map(() => subtype.default);

    return {
      type: ARRAY_TYPE,
      subtype: subtype,
      length: M,
      default: defaultValue
    };
  }

  const lengthRegex = /^(u?int|bytes)(\d{1,3})$/;

  if (lengthRegex.test(type)) {
    const matches = lengthRegex.exec(type);

    const subtype = parseAbiType(matches[1]);
    const length = parseInt(matches[2]);

    return {
      ...subtype,
      length
    };
  }

  const fixedLengthRegex = /^(u?fixed)(\d{1,3})x(\d{1,3})$/;

  if (fixedLengthRegex.test(type)) {
    const matches = fixedLengthRegex.exec(type);

    const subtype = parseAbiType(matches[1]);
    const M = parseInt(matches[2]);
    const N = parseInt(matches[3]);

    return {
      ...subtype,
      M, N
    };
  }

  if (type === 'string') {
    return {
      type: STRING_TYPE,
      default: ''
    };
  }

  if (type === 'bool') {
    return {
      type: BOOL_TYPE,
      default: false
    };
  }

  if (type === 'address') {
    return {
      type: ADDRESS_TYPE,
      default: ''
    };
  }

  if (type === 'bytes') {
    return {
      type: BYTES_TYPE,
      default: '0x'
    };
  }

  if (type === 'uint') {
    return {
      type: INT_TYPE,
      default: 0,
      length: 256,
      signed: false
    };
  }

  if (type === 'int') {
    return {
      type: INT_TYPE,
      default: 0,
      length: 256,
      signed: true
    };
  }

  if (type === 'ufixed') {
    return {
      type: FIXED_TYPE,
      default: 0,
      length: 256,
      signed: false
    };
  }

  if (type === 'fixed') {
    return {
      type: FIXED_TYPE,
      default: 0,
      length: 256,
      signed: true
    };
  }

  // If no matches, return null
  return null;
}
