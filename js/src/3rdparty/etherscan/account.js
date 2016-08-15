const PAGE_SIZE = 25;

import Api from '../../api';
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
  }).then((transactions) => {
    return transactions.map((tx) => {
      return {
        from: Api.format.toChecksumAddress(tx.from),
        to: Api.format.toChecksumAddress(tx.to),
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
