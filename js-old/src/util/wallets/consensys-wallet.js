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

import Abi from '@parity/abi';
import Contract from '@parity/api/lib/contract';
import { toHex } from '@parity/api/lib/util/format';

import WalletAbi from '~/contracts/abi/consensys-multisig-wallet.json';

import {
  UPDATE_OWNERS,
  UPDATE_REQUIRE,
  UPDATE_TRANSACTIONS,
  UPDATE_CONFIRMATIONS
} from './updates';

const WALLET_CONTRACT = new Contract({}, WalletAbi);
const WALLET_ABI = new Abi(WalletAbi);

const walletEvents = WALLET_ABI.events.reduce((events, event) => {
  events[event.name] = event;
  return events;
}, {});

const WalletSignatures = {
  Confirmation: toHex(walletEvents.Confirmation.signature),
  Revocation: toHex(walletEvents.Revocation.signature),
  Deposit: toHex(walletEvents.Deposit.signature),
  Execution: toHex(walletEvents.Execution.signature),
  OwnerAddition: toHex(walletEvents.OwnerAddition.signature),
  OwnerRemoval: toHex(walletEvents.OwnerRemoval.signature),
  RequirementChange: toHex(walletEvents.RequirementChange.signature),
  Submission: toHex(walletEvents.Submission.signature)
};

export default class ConsensysWalletUtils {
  static fetchOwners (inWallet) {
    const wallet = new Contract(inWallet.api, WalletAbi).at(inWallet.address);

    return wallet.instance.getOwners.call()
      .then((owners) => {
        return owners.map((token) => token.value);
      });
  }

  static fetchPendingTransactions (inWallet) {
    const wallet = new Contract(inWallet.api, WalletAbi).at(inWallet.address);

    let transactions;
    let txIds;

    // Get pending and not exectued transactions
    return wallet.instance.getTransactionCount
      .call({}, [ true, false ])
      .then((txCount) => {
        // Get all the pending transactions
        const fromId = 0;
        const toId = txCount;

        return wallet.instance.getTransactionIds
          .call({}, [ fromId, toId, true, false ]);
      })
      .then((_txIds) => {
        txIds = _txIds.map((token) => token.value);

        const promises = txIds.map((txId) => {
          return wallet.instance.transactions
            .call({}, [ txId ]);
        });

        return Promise.all(promises);
      })
      .then((transactions) => {
        return transactions.map((transaction, index) => {
          const [ destination, value, data ] = transaction;
          const id = toHex(txIds[index]);

          return {
            to: destination,
            data,
            value,
            operation: id
          };
        });
      })
      .then((_transactions) => {
        transactions = _transactions;

        return wallet
          .getAllLogs({
            topics: [
              WalletSignatures.Submission,
              txIds.map((txId) => toHex(txId))
            ]
          });
      })
      .then((logs) => {
        transactions.forEach((tx) => {
          const log = logs
            .find((log) => {
              const id = toHex(log.params.transactionId.value);

              return id === tx.operation;
            });

          if (!log) {
            console.warn('could not find a Submission log for this operation', tx);
            return;
          }

          tx.transactionIndex = log.transactionIndex;
          tx.transactionHash = log.transactionHash;
          tx.blockNumber = log.blockNumber;
        });

        const confirmationsPromises = transactions.map((tx) => {
          return wallet.instance.getConfirmations
            .call({}, [ tx.operation ])
            .then((owners) => {
              return owners.map((token) => token.value);
            });
        });

        return Promise.all(confirmationsPromises);
      })
      .then((confirmations) => {
        transactions.forEach((tx, index) => {
          tx.confirmedBy = confirmations[index];
        });

        return transactions;
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
        txIds = _txIds.map((token) => token.value);

        const promises = txIds.map((txId) => {
          return wallet.instance.transactions
            .call({}, [ txId ]);
        });

        return Promise.all(promises);
      })
      .then((transactions) => {
        return transactions.map((transaction, index) => {
          const [ destination, value, data, executed ] = transaction;
          const id = toHex(txIds[index]);

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
            transactionIndex: log.transactionIndex,
            blockNumber: log.blockNumber
          };

          if (signature === WalletSignatures.Deposit) {
            transaction.from = log.params.sender.value;
            transaction.value = log.params.value.value;
            transaction.to = wallet.address;
          } else {
            const txId = toHex(log.params.transactionId.value);
            const tx = transactions.find((tx) => tx.id === txId);

            transaction.from = wallet.address;
            transaction.to = tx.destination;
            transaction.value = tx.value;
            transaction.data = tx.data;
            transaction.operation = toHex(tx.id);
          }

          return transaction;
        });
      });
  }

  static getChangeMethod (api, address, change) {
    const wallet = new Contract(api, WalletAbi).at(address);
    const walletInstance = wallet.instance;

    let data = '';

    if (change.type === 'require') {
      const func = walletInstance.changeRequirement;

      data = wallet.getCallData(func, {}, [ change.value ]);
    }

    if (change.type === 'add_owner') {
      const func = walletInstance.addOwner;

      data = wallet.getCallData(func, {}, [ change.value ]);
    }

    if (change.type === 'change_owner') {
      const func = walletInstance.replaceOwner;

      data = wallet.getCallData(func, {}, [ change.value.from, change.value.to ]);
    }

    if (change.type === 'remove_owner') {
      const func = walletInstance.removeOwner;

      data = wallet.getCallData(func, {}, [ change.value ]);
    }

    const method = walletInstance.submitTransaction;
    const values = [ address, 0, data ];

    return { method, values };
  }

  static getModifyOperationMethod (modification) {
    switch (modification) {
      case 'confirm':
        return 'confirmTransaction';

      case 'revoke':
        return 'revokeConfirmation';

      default:
        return '';
    }
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

  static logToUpdate (log) {
    const eventSignature = toHex(log.topics[0]);

    switch (eventSignature) {
      case WalletSignatures.OwnerAddition:
      case WalletSignatures.OwnerRemoval:
        return { [ UPDATE_OWNERS ]: true };

      case WalletSignatures.RequirementChange:
        return { [ UPDATE_REQUIRE ]: true };

      case WalletSignatures.Deposit:
      case WalletSignatures.Execution:
        return { [ UPDATE_TRANSACTIONS ]: true };

      case WalletSignatures.Submission:
      case WalletSignatures.Confirmation:
      case WalletSignatures.Revocation:
        const parsedLog = WALLET_CONTRACT.parseEventLogs([ log ])[0];
        const operation = toHex(parsedLog.params.transactionId.value);

        return { [ UPDATE_CONFIRMATIONS ]: operation };

      default:
        return {};
    }
  }

  /**
   * This type of wallet cannot create any contract...
   */
  static parseTransactionLogs (api, options, rawLogs) {
    return null;
  }
}
