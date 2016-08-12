const PAGE_SIZE = 25;

import { call } from './call';

function _call (method, params) {
  return call('account', method, params);
}

function balance (address) {
  return _call('balance', {
    address: address,
    tag: 'latest'
  }).then((balance) => {
    // same format as balancemulti below
    return {
      account: address,
      balance: balance
    };
  });
}

function balances (addresses) {
  return _call('balancemulti', {
    address: addresses.join(','),
    tag: 'latest'
  });
}

function transactions (address, page) {
  // page offset from 0
  return _call('txlist', {
    address: address,
    page: (page || 0) + 1,
    offset: PAGE_SIZE,
    sort: 'desc'
  });
}

const account = {
  balance: balance,
  balances: balances,
  transactions: transactions
};

export { account };
