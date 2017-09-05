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

const PAGE_SIZE = 25;

import util from '../../api/util';
import { call } from './call';

function _call (method, params, test, netVersion) {
  return call('account', method, params, test, netVersion);
}

function balance (address, test, netVersion) {
  return _call('balance', {
    address: address,
    tag: 'latest'
  }, test, netVersion).then((balance) => {
    // same format as balancemulti below
    return {
      account: address,
      balance: balance
    };
  });
}

function balances (addresses, test, netVersion) {
  return _call('balancemulti', {
    address: addresses.join(','),
    tag: 'latest'
  }, test, netVersion);
}

function transactions (address, page, test, netVersion) {
  // page offset from 0
  return _call('txlist', {
    address: address,
    offset: PAGE_SIZE,
    page: (page || 0) + 1,
    sort: 'desc'
  }, test, netVersion).then((transactions) => {
    return transactions.map((tx) => {
      return {
        blockNumber: new BigNumber(tx.blockNumber || 0),
        from: util.toChecksumAddress(tx.from),
        hash: tx.hash,
        timeStamp: tx.timeStamp,
        to: util.toChecksumAddress(tx.to),
        value: tx.value
      };
    });
  });
}

const account = {
  balance,
  balances,
  transactions
};

export { account };
