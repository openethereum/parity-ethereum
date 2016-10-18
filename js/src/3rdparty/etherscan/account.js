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

const PAGE_SIZE = 25;

import util from '../../api/util';
import { call } from './call';

function _call (method, params, test) {
  return call('account', method, params, test);
}

function balance (address, test = false) {
  return _call('balance', {
    address: address,
    tag: 'latest'
  }, test).then((balance) => {
    // same format as balancemulti below
    return {
      account: address,
      balance: balance
    };
  });
}

function balances (addresses, test = false) {
  return _call('balancemulti', {
    address: addresses.join(','),
    tag: 'latest'
  }, test);
}

function transactions (address, page, test = false) {
  // page offset from 0
  return _call('txlist', {
    address: address,
    page: (page || 0) + 1,
    offset: PAGE_SIZE,
    sort: 'desc'
  }, test).then((transactions) => {
    return transactions.map((tx) => {
      return {
        from: util.toChecksumAddress(tx.from),
        to: util.toChecksumAddress(tx.to),
        hash: tx.hash,
        blockNumber: tx.blockNumber,
        timeStamp: tx.timeStamp,
        value: tx.value
      };
    });
  });
}

const account = {
  balance: balance,
  balances: balances,
  transactions: transactions
};

export { account };
