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
import { intersection, range, uniq } from 'lodash';
import store from 'store';

import Abi from '~/abi';
import Contract from '~/api/contract';
import { bytesToHex, toHex } from '~/api/util/format';
import { validateAddress } from '~/util/validation';
import WalletAbi from '~/contracts/abi/wallet.json';
import OldWalletAbi from '~/contracts/abi/old-wallet.json';

const LS_PENDING_CONTRACTS_KEY = '_parity::wallets::pendingContracts';

const _cachedWalletLookup = {};
let _cachedAccounts = {};

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

export default class WalletsUtils {
  static getWalletSignatures () {
    return WalletSignatures;
  }

  static getPendingContracts () {
    return store.get(LS_PENDING_CONTRACTS_KEY) || {};
  }

  static setPendingContracts (contracts = {}) {
    return store.set(LS_PENDING_CONTRACTS_KEY, contracts);
  }

  static removePendingContract (operationHash) {
    const nextContracts = WalletsUtils.getPendingContracts();

    delete nextContracts[operationHash];
    WalletsUtils.setPendingContracts(nextContracts);
  }

  static addPendingContract (address, operationHash, metadata) {
    const nextContracts = {
      ...WalletsUtils.getPendingContracts(),
      [ operationHash ]: {
        address,
        metadata,
        operationHash
      }
    };

    WalletsUtils.setPendingContracts(nextContracts);
  }

  static cacheAccounts (accounts) {
    _cachedAccounts = accounts;
  }

  static getCallArgs (api, options, values = []) {
    const walletContract = new Contract(api, WalletAbi);
    const walletAddress = options.from;

    return WalletsUtils
      .fetchOwners(walletContract.at(walletAddress))
      .then((owners) => {
        const addresses = Object.keys(_cachedAccounts);
        const ownerAddress = intersection(addresses, owners).pop();

        if (!ownerAddress) {
          return false;
        }

        const account = _cachedAccounts[ownerAddress];
        const _options = { ...options };
        const { to, value = new BigNumber(0), data } = _options;

        delete _options.data;

        const nextValues = [ to, value, data ];
        const nextOptions = {
          ..._options,
          from: ownerAddress,
          to: walletAddress,
          value: new BigNumber(0)
        };

        const execFunc = walletContract.instance.execute;
        const callArgs = { func: execFunc, options: nextOptions, values: nextValues };

        if (!account.wallet) {
          return callArgs;
        }

        const nextData = walletContract.getCallData(execFunc, nextOptions, nextValues);

        return WalletsUtils.getCallArgs(api, { ...nextOptions, data: nextData }, nextValues);
      });
  }

  static getDeployArgs (contract, options, values) {
    const { api } = contract;
    const func = contract.constructors[0];

    options.data = contract.getCallData(func, options, values);
    options.to = '0x';

    return WalletsUtils
      .getCallArgs(api, options, values)
      .then((callArgs) => {
        if (!callArgs) {
          console.error('no call args', callArgs);
          throw new Error('you do not own this wallet');
        }

        return callArgs;
      });
  }

  static parseLogs (api, logs = []) {
    const walletContract = new Contract(api, WalletAbi);

    return walletContract.parseEventLogs(logs);
  }

  /**
   * Check whether the given address could be
   * a Wallet. The result is cached in order not
   * to make unnecessary calls on non-wallet accounts
   */
  static isWallet (api, address) {
    if (!_cachedWalletLookup[address]) {
      const walletContract = new Contract(api, WalletAbi);

      _cachedWalletLookup[address] = walletContract
        .at(address)
        .instance
        .m_numOwners
        .call()
        .then((result) => {
          if (!result || result.equals(0)) {
            return false;
          }

          return true;
        })
        .then((bool) => {
          _cachedWalletLookup[address] = Promise.resolve(bool);
          return bool;
        });
    }

    return _cachedWalletLookup[address];
  }

  static fetchRequire (walletContract) {
    return walletContract.instance.m_required.call();
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
          .then((owners) => {
            const uniqOwners = uniq(owners);

            // If all owners are the zero account : must be Mist wallet contract
            if (uniqOwners.length === 1 && /^(0x)?0*$/.test(owners[0])) {
              return WalletsUtils.fetchMistOwners(walletContract, mNumOwners.toNumber());
            }

            return owners;
          });
      });
  }

  static fetchMistOwners (walletContract, mNumOwners) {
    const walletAddress = walletContract.address;

    return WalletsUtils
      .getMistOwnersOffset(walletContract)
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

  static getMistOwnersOffset (walletContract, offset = 3) {
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

        return WalletsUtils.getMistOwnersOffset(walletContract, offset + 1);
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
    const { api } = walletContract;
    const pendingContracts = WalletsUtils.getPendingContracts();

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
            const operation = bytesToHex(log.params.operation.value);

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
                  const transactionLogs = WalletsUtils.parseLogs(api, transactionReceipt.logs);
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
                      WalletsUtils.removePendingContract(operation);
                    });
                })
                .catch((error) => {
                  console.error('adding wallet contract', error);
                });
            }

            transaction.operation = operation;
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
