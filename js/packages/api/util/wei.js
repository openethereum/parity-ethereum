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

const UNITS = ['wei', 'ada', 'babbage', 'shannon', 'szabo', 'finney', 'ether', 'kether', 'mether', 'gether', 'tether'];

export function _getUnitMultiplier (unit) {
  const position = UNITS.indexOf(unit.toLowerCase());

  if (position === -1) {
    throw new Error(`Unknown unit ${unit} passed to wei formatter`);
  }

  return 10 ** (position * 3);
}

export function fromWei (value, unit = 'ether') {
  return new BigNumber(value).div(_getUnitMultiplier(unit));
}

export function toWei (value, unit = 'ether') {
  return new BigNumber(value).mul(_getUnitMultiplier(unit));
}
