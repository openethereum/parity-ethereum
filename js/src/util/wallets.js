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

import { range } from 'lodash';

import { bytesToHex, toHex } from '~/api/util/format';

export default class WalletsUtils {

  static fetchRequire (walletContract) {
    return walletContract.instance.m_required.call();
  }

  static fetchOwners (walletContract) {
    const walletInstance = walletContract.instance;
    return walletInstance
      .m_numOwners.call()
      .then((mNumOwners) => {
        return Promise.all(range(mNumOwners.toNumber()).map((idx) => walletInstance.getOwner.call({}, [ idx ])));
      });
  }

  static fetchDailylimit (walletContract) {
    const walletInstance = walletContract.instance;

    return Promise
      .all([
        walletInstance.m_dailyLimit.call(),
        walletInstance.m_spentToday.call(),
        walletInstance.m_lastDay.call()
      ])
      .then(([ limit, spent, last ]) => ({
        limit, spent, last
      }));
  }

  static fetchTransactions (walletContract) {
    const walletInstance = walletContract.instance;
    const signatures = {
      single: toHex(walletInstance.SingleTransact.signature),
      multi: toHex(walletInstance.MultiTransact.signature),
      deposit: toHex(walletInstance.Deposit.signature)
    };

    return walletContract
      .getAllLogs({
        topics: [ [ signatures.single, signatures.multi, signatures.deposit ] ]
      })
      .then((logs) => {
        return logs.sort((logA, logB) => {
          const comp = logB.blockNumber.comparedTo(logA.blockNumber);

          if (comp !== 0) {
            return comp;
          }

          return logB.transactionIndex.comparedTo(logA.transactionIndex);
        });
      })
      .then((logs) => {
        const transactions = logs.map((log) => {
          const signature = toHex(log.topics[0]);

          const value = log.params.value.value;
          const from = signature === signatures.deposit
            ? log.params['_from'].value
            : walletContract.address;

          const to = signature === signatures.deposit
            ? walletContract.address
            : log.params.to.value;

          const transaction = {
            transactionHash: log.transactionHash,
            blockNumber: log.blockNumber,
            from, to, value
          };

          if (log.params.operation) {
            transaction.operation = bytesToHex(log.params.operation.value);
          }

          if (log.params.data) {
            transaction.data = log.params.data.value;
          }

          return transaction;
        });

        return transactions;
      });
  }

}
