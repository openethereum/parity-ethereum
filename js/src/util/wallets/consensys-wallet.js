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

import Abi from '~/abi';
import Contract from '~/api/contract';
import { toHex } from '~/api/util/format';

import WalletAbi from '~/contracts/abi/consensys-multisig-wallet.json';

const walletAbi = new Abi(WalletAbi);

const walletEvents = walletAbi.events.reduce((events, event) => {
  events[event.name] = event;
  return events;
}, {});

const WalletSignatures = {
  Deposit: toHex(walletEvents.Deposit.signature),
  Execution: toHex(walletEvents.Execution.signature)
};

export default class ConsensysWalletUtils {
  static fetchOwners (inWallet) {
    const wallet = new Contract(inWallet.api, WalletAbi).at(inWallet.address);

    return wallet.instance.getOwners.call()
      .then((owners) => {
        return owners.map((token) => token.value);
      });
  }

  static fetchRequire (inWallet) {
    const wallet = new Contract(inWallet.api, WalletAbi).at(inWallet.address);

    return wallet.instance.required.call();
  }

  static fetchTransactions (inWallet) {
    const wallet = new Contract(inWallet.api, WalletAbi).at(inWallet.address);

    let transactions;
    let txIds;

    return wallet.instance.getTransactionCount
      .call({}, [ false, true ])
      .then((txCount) => {
        // Get the 20 last transactions
        const fromId = Math.max(0, txCount - 20);
        const toId = txCount;

        return wallet.instance.getTransactionIds
          .call({}, [ fromId, toId, false, true ]);
      })
      .then((_txIds) => {
        txIds = _txIds;

        const promises = txIds.map((txId) => {
          return wallet.instance.transactions
            .call({}, [ txId ]);
        });

        return Promise.all(promises);
      })
      .then((transactions) => {
        return transactions.map((transaction, index) => {
          const [ destination, value, data, executed ] = transaction;
          const id = txIds[index];

          return {
            destination, value, data, executed, id
          };
        });
      })
      .then((_transactions) => {
        transactions = _transactions;

        const depositLogs = wallet
          .getAllLogs({
            topics: [ WalletSignatures.Deposit ]
          });

        const executionLogs = wallet
          .getAllLogs({
            topics: [ WalletSignatures.Execution, txIds ]
          });

        return Promise.all([ depositLogs, executionLogs ]);
      })
      .then(([ depositLogs, executionLogs ]) => {
        const logs = [].concat(depositLogs, executionLogs);

        return logs.map((log) => {
          const signature = toHex(log.topics[0]);

          const transaction = {
            transactionHash: log.transactionHash,
            blockNumber: log.blockNumber
          };

          if (signature === WalletSignatures.Deposit) {
            transaction.from = log.params.sender.value;
            transaction.value = log.params.value.value;
            transaction.to = wallet.address;
          } else {
            const txId = log.params.transactionId.value;
            const transaction = transactions.find((tx) => tx.id === txId);

            transaction.from = wallet.address;
            transaction.to = transaction.destination;
            transaction.value = transaction.value;
            transaction.data = transaction.data;
            transaction.operation = transaction.id;
          }

          return transaction;
        });
      });
  }

  static getSubmitMethod () {
    return 'submitTransaction';
  }

  static getWalletContract (api) {
    return new Contract(api, WalletAbi);
  }

  static getWalletSignatures () {
    return WalletSignatures;
  }

  static fetchDailylimit () {
    return {
      last: new BigNumber(0),
      limit: new BigNumber(0),
      spent: new BigNumber(0)
    };
  }

  static isWallet (api, address) {
    const wallet = new Contract(api, WalletAbi).at(address);

    return ConsensysWalletUtils.fetchRequire(wallet)
      .then((result) => {
        if (!result || result.equals(0)) {
          return false;
        }

        return true;
      });
  }

  /**
   * This type of wallet cannot create any contract...
   */
  static parseTransactionLogs (api, options, rawLogs) {
    return null;
  }
}
