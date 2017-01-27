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
import moment from 'moment';

import { api } from './parity';

const ZERO = new BigNumber(0);

export function formatBlockNumber (blockNumber) {
  return ZERO.eq(blockNumber || 0)
    ? 'Pending'
    : `${blockNumber.toFormat()}`;
}

export function formatSignature (signature) {
  return api.util.bytesToHex(signature);
}

export function formatBlockTimestamp (block) {
  if (!block || !block.timestamp) {
    return null;
  }

  return moment(block.timestamp).fromNow();
}
