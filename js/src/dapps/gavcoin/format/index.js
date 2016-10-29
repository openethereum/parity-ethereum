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

import BigNumber from 'bignumber.js';

import { api } from '../parity';

const DIVISOR = 10 ** 6;
const ZERO = new BigNumber(0);

export function formatBlockNumber (blockNumber) {
  return ZERO.eq(blockNumber || 0)
    ? 'Pending'
    : `#${blockNumber.toFormat()}`;
}

export function formatCoins (amount, decimals = 6) {
  const adjusted = amount.div(DIVISOR);

  if (decimals === -1) {
    if (adjusted.gte(10000)) {
      decimals = 0;
    } else if (adjusted.gte(1000)) {
      decimals = 1;
    } else if (adjusted.gte(100)) {
      decimals = 2;
    } else if (adjusted.gte(10)) {
      decimals = 3;
    } else {
      decimals = 4;
    }
  }

  return adjusted.toFormat(decimals);
}

export function formatEth (eth, decimals = 3) {
  return api.util.fromWei(eth).toFormat(decimals);
}

export function formatHash (hash) {
  return `${hash.substr(0, 10)}...${hash.substr(-8)}`;
}
