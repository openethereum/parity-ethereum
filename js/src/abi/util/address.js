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

import { keccak_256 } from 'js-sha3'; // eslint-disable-line camelcase

export function isChecksumValid (_address) {
  const address = _address.replace('0x', '');
  const hash = keccak_256(address.toLowerCase());

  for (let n = 0; n < 40; n++) {
    const char = address[n];
    const isLower = char !== char.toUpperCase();
    const isUpper = char !== char.toLowerCase();
    const hashval = parseInt(hash[n], 16);

    if ((hashval > 7 && isLower) || (hashval <= 7 && isUpper)) {
      return false;
    }
  }

  return true;
}

export function isAddress (address) {
  if (address && address.length === 42) {
    if (!/^(0x)?[0-9a-f]{40}$/i.test(address)) {
      return false;
    } else if (/^(0x)?[0-9a-f]{40}$/.test(address) || /^(0x)?[0-9A-F]{40}$/.test(address)) {
      return true;
    }

    return isChecksumValid(address);
  }

  return false;
}

export function toChecksumAddress (_address) {
  const address = (_address || '').toLowerCase();

  if (!isAddress(address)) {
    return '';
  }

  const hash = keccak_256(address.slice(-40));
  let result = '0x';

  for (let n = 0; n < 40; n++) {
    result = `${result}${parseInt(hash[n], 16) > 7 ? address[n + 2].toUpperCase() : address[n + 2]}`;
  }

  return result;
}
