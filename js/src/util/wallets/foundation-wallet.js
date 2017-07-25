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

import { range, uniq } from 'lodash';

import Abi from '~/abi';
import Contract from '~/api/contract';
import { bytesToHex, toHex } from '~/api/util/format';
import { validateAddress } from '~/util/validation';

import WalletAbi from '~/contracts/abi/wallet.json';
import OldWalletAbi from '~/contracts/abi/old-wallet.json';

import PendingContracts from './pending-contracts';

const walletAbi = new Abi(WalletAbi);
const oldWalletAbi = new Abi(OldWalletAbi);

const walletEvents = walletAbi.events.reduce((events, event) => {
  events[event.name] = event;
  return events;
}, {});

const oldWalletEvents = oldWalletAbi.events.reduce((events, event) => {
  events[event.name] = event;
  return events;
}, {});

const WalletSignatures = {
  OwnerChanged: toHex(walletEvents.OwnerChanged.signature),
  OwnerAdded: toHex(walletEvents.OwnerAdded.signature),
  OwnerRemoved: toHex(walletEvents.OwnerRemoved.signature),
  RequirementChanged: toHex(walletEvents.RequirementChanged.signature),
  Confirmation: toHex(walletEvents.Confirmation.signature),
  Revoke: toHex(walletEvents.Revoke.signature),
  Deposit: toHex(walletEvents.Deposit.signature),
  SingleTransact: toHex(walletEvents.SingleTransact.signature),
  MultiTransact: toHex(walletEvents.MultiTransact.signature),
  ConfirmationNeeded: toHex(walletEvents.ConfirmationNeeded.signature),

  Old: {
    SingleTransact: toHex(oldWalletEvents.SingleTransact.signature),
    MultiTransact: toHex(oldWalletEvents.MultiTransact.signature)
  }
};

export default class FoundationWalletUtils {
  static fetchConfirmations (walletContract, operation, _owners = null) {
    const ownersPromise = _owners
      ? Promise.resolve(_owners)
      : FoundationWalletUtils.fetchOwners(walletContract);

    return ownersPromise
      .then((owners) => {
        const promises = owners.map((owner) => {
          return walletContract.instance.hasConfirmed.call({}, [ operation, owner ]);
        });

        return Promise
          .all(promises)
          .then((data) => {
            return owners.filter((_, index) => data[index]);
          });
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

  static fetchOwners (walletContract) {
    const walletInstance = walletContract.instance;

    return walletInstance
      .m_numOwners.call()
      .then((mNumOwners) => {
        const promises = range(mNumOwners.toNumber())
          .map((idx) => walletInstance.getOwner.call({}, [ idx ]));

        return Promise
          .all(promises)
          .then((_owners) => {
            const owners = validateOwners(_owners);

            // If all owners are the zero account : must be Mist wallet contract
            if (!owners) {
              return fetchMistOwners(walletContract, mNumOwners.toNumber());
            }

            return owners;
          });
      });
  }

  static fetchPendingTransactions (walletContract, cache = {}) {
    const { owners, transactions } = cache;

    return walletContract
      .instance
      .ConfirmationNeeded
      .getAllLogs()
      .then((logs) => {
        return logs.map((log) => ({
          initiator: log.params.initiator.value,
          to: log.params.to.value,
          data: log.params.data.value,
          value: log.params.value.value,
          operation: bytesToHex(log.params.operation.value),
          transactionIndex: log.transactionIndex,
          transactionHash: log.transactionHash,
          blockNumber: log.blockNumber,
          confirmedBy: []
        }));
      })
      .then((logs) => {
        return logs.sort((logA, logB) => {
          const comp = logA.blockNumber.comparedTo(logB.blockNumber);

          if (comp !== 0) {
            return comp;
          }

          return logA.transactionIndex.comparedTo(logB.transactionIndex);
        });
      })
      .then((pendingTxs) => {
        if (pendingTxs.length === 0) {
          return pendingTxs;
        }

        // Only fetch confirmations for operations not
        // yet confirmed (ie. not yet a transaction)
        if (transactions) {
          const operations = transactions
            .filter((t) => t.operation)
            .map((t) => t.operation);

          return pendingTxs.filter((pendingTx) => {
            return !operations.includes(pendingTx.operation);
          });
        }

        return pendingTxs;
      })
      .then((pendingTxs) => {
        const promises = pendingTxs.map((tx) => {
          return FoundationWalletUtils
            .fetchConfirmations(walletContract, tx.operation, owners)
            .then((confirmedBy) => {
              tx.confirmedBy = confirmedBy;

              return tx;
            });
        });

        return Promise.all(promises);
      });
  }

  static fetchRequire (wallet) {
    return wallet.instance.m_required.call();
  }

  static fetchTransactions (walletContract) {
    const { api } = walletContract;

    return walletContract
      .getAllLogs({
        topics: [ [
          WalletSignatures.SingleTransact,
          WalletSignatures.MultiTransact,
          WalletSignatures.Deposit,
          WalletSignatures.Old.SingleTransact,
          WalletSignatures.Old.MultiTransact
        ] ]
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
          const from = signature === WalletSignatures.Deposit
            ? log.params['_from'].value
            : walletContract.address;

          const to = signature === WalletSignatures.Deposit
            ? walletContract.address
            : log.params.to.value;

          const transaction = {
            transactionHash: log.transactionHash,
            blockNumber: log.blockNumber,
            from, to, value
          };

          if (log.params.created && log.params.created.value && !/^(0x)?0*$/.test(log.params.created.value)) {
            transaction.creates = log.params.created.value;
            delete transaction.to;
          }

          if (log.params.operation) {
            transaction.operation = bytesToHex(log.params.operation.value);
            checkPendingOperation(api, log, transaction.operation);
          }

          if (log.params.data) {
            transaction.data = log.params.data.value;
          }

          return transaction;
        });

        return transactions;
      });
  }

  static getSubmitMethod () {
    return 'execute';
  }

  static getWalletContract (api) {
    return new Contract(api, WalletAbi);
  }

  static getWalletSignatures () {
    return WalletSignatures;
  }

  static isWallet (api, address) {
    const walletContract = new Contract(api, WalletAbi);

    return walletContract
      .at(address)
      .instance
      .m_numOwners
      .call()
      .then((result) => {
        if (!result || result.equals(0)) {
          return false;
        }

        return true;
      });
  }

  static parseLogs (api, logs = []) {
    const walletContract = new Contract(api, WalletAbi);

    return walletContract.parseEventLogs(logs);
  }

  static parseTransactionLogs (api, options, rawLogs) {
    const { metadata } = options;
    const address = options.from;
    const logs = FoundationWalletUtils.parseLogs(api, rawLogs);

    const confirmationLog = logs.find((log) => log.event === 'ConfirmationNeeded');
    const transactionLog = logs.find((log) => log.event === 'SingleTransact');

    if (!confirmationLog && !transactionLog) {
      return null;
    }

    // Confirmations are needed from the other owners
    if (confirmationLog) {
      const operationHash = bytesToHex(confirmationLog.params.operation.value);

      // Add the contract to pending contracts
      PendingContracts.addPendingContract(address, operationHash, metadata);

      return { pending: true };
    }

    return { contractAddress: transactionLog.params.created.value };
  }
}

function checkPendingOperation (api, log, operation) {
  const pendingContracts = PendingContracts.getPendingContracts();

  // Add the pending contract to the contracts
  if (pendingContracts[operation]) {
    const { metadata } = pendingContracts[operation];
    const contractName = metadata.name;

    metadata.blockNumber = log.blockNumber;

    // The contract creation might not be in the same log,
    // but must be in the same transaction (eg. Contract creation
    // from Wallet within a Wallet)
    api.eth
      .getTransactionReceipt(log.transactionHash)
      .then((transactionReceipt) => {
        const transactionLogs = FoundationWalletUtils.parseLogs(api, transactionReceipt.logs);
        const creationLog = transactionLogs.find((log) => {
          return log.params.created && !/^(0x)?0*$/.test(log.params.created.value);
        });

        if (!creationLog) {
          return false;
        }

        const contractAddress = creationLog.params.created.value;

        return Promise
          .all([
            api.parity.setAccountName(contractAddress, contractName),
            api.parity.setAccountMeta(contractAddress, metadata)
          ])
          .then(() => {
            PendingContracts.removePendingContract(operation);
          });
      })
      .catch((error) => {
        console.error('adding wallet contract', error);
      });
  }
}

function fetchMistOwners (walletContract, mNumOwners) {
  const walletAddress = walletContract.address;

  return getMistOwnersOffset(walletContract)
    .then((result) => {
      if (!result || result.offset === -1) {
        return [];
      }

      const owners = [ result.address ];

      if (mNumOwners === 1) {
        return owners;
      }

      const initOffset = result.offset + 1;
      let promise = Promise.resolve();

      range(initOffset, initOffset + mNumOwners - 1).forEach((offset) => {
        promise = promise
          .then(() => {
            return walletContract.api.eth.getStorageAt(walletAddress, offset);
          })
          .then((result) => {
            const resultAddress = '0x' + (result || '').slice(-40);
            const { address } = validateAddress(resultAddress);

            owners.push(address);
          });
      });

      return promise.then(() => owners);
    });
}

function getMistOwnersOffset (walletContract, offset = 3) {
  return walletContract.api.eth
    .getStorageAt(walletContract.address, offset)
    .then((result) => {
      if (result && !/^(0x)?0*$/.test(result)) {
        const resultAddress = '0x' + result.slice(-40);
        const { address, addressError } = validateAddress(resultAddress);

        if (!addressError) {
          return { offset, address };
        }
      }

      if (offset >= 100) {
        return { offset: -1 };
      }

      return getMistOwnersOffset(walletContract, offset + 1);
    });
}

function validateOwners (owners) {
  const uniqOwners = uniq(owners);

  // If all owners are the zero account : must be Mist wallet contract
  if (uniqOwners.length === 1 && /^(0x)?0*$/.test(owners[0])) {
    return null;
  }

  return uniqOwners;
}
