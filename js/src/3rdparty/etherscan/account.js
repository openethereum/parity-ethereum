const PAGE_SIZE = 25;

import format from '../../api/format';
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
        from: format.toChecksumAddress(tx.from),
        to: format.toChecksumAddress(tx.to),
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
