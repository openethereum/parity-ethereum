// Copyright 2015, 2016 Parity Technologies (UK) Ltd.
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

import Contract from '~/api/contract';
import { bytesToHex, toHex } from '~/api/util/format';
import { validateAddress } from '~/util/validation';
import WalletAbi from '~/contracts/abi/wallet.json';

const _cachedWalletLookup = {};

export default class WalletsUtils {
  static getCallArgs (api, options, values = []) {
    const walletContract = new Contract(api, WalletAbi);

    const promises = [
      api.parity.accountsInfo(),
      WalletsUtils.fetchOwners(walletContract.at(options.from))
    ];

    return Promise
      .all(promises)
      .then(([ accounts, owners ]) => {
        const addresses = Object.keys(accounts);
        const owner = intersection(addresses, owners).pop();

        if (!owner) {
          return false;
        }

        return owner;
      })
      .then((owner) => {
        if (!owner) {
          return false;
        }

        const _options = Object.assign({}, options);
        const { from, to, value = new BigNumber(0), data } = options;

        delete _options.data;

        const nextValues = [ to, value, data ];
        const nextOptions = {
          ..._options,
          from: owner,
          to: from,
          value: new BigNumber(0)
        };

        const execFunc = walletContract.instance.execute;

        return { func: execFunc, options: nextOptions, values: nextValues };
      });
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
