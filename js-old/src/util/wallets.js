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
import { intersection } from 'lodash';

import ConsensysWalletUtils from './wallets/consensys-wallet';
import FoundationWalletUtils from './wallets/foundation-wallet';

const CONSENSYS_WALLET = 'CONSENSYS_WALLET';
const FOUNDATION_WALLET = 'FOUNDATION_WALLET';

const _cachedWalletLookup = {};
const _cachedWalletTypes = {};
let _cachedAccounts = {};

export default class WalletsUtils {
  static cacheAccounts (accounts) {
    _cachedAccounts = accounts;
  }

  static delegateCall (api, address, method, args = []) {
    return WalletsUtils.getWalletType(api, address)
      .then((walletType) => {
        if (walletType === CONSENSYS_WALLET) {
          return ConsensysWalletUtils[method].apply(null, args);
        }

        return FoundationWalletUtils[method].apply(null, args);
      });
  }

  static fetchDailylimit (walletContract) {
    const { api } = walletContract;

    return WalletsUtils
      .delegateCall(api, walletContract.address, 'fetchDailylimit', [ walletContract ]);
  }

  static fetchOwners (walletContract) {
    const { api } = walletContract;

    return WalletsUtils
      .delegateCall(api, walletContract.address, 'fetchOwners', [ walletContract ]);
  }

  static fetchRequire (walletContract) {
    const { api } = walletContract;

    return WalletsUtils
      .delegateCall(api, walletContract.address, 'fetchRequire', [ walletContract ]);
  }

  static fetchPendingTransactions (walletContract, cache) {
    const { api } = walletContract;

    return WalletsUtils
      .delegateCall(api, walletContract.address, 'fetchPendingTransactions', [ walletContract, cache ]);
  }

  static fetchTransactions (walletContract) {
    const { api } = walletContract;

    return WalletsUtils
      .delegateCall(api, walletContract.address, 'fetchTransactions', [ walletContract ])
      .then((transactions) => {
        return transactions.sort((txA, txB) => {
          const comp = txB.blockNumber.comparedTo(txA.blockNumber);

          if (comp !== 0) {
            return comp;
          }

          return txB.transactionIndex.comparedTo(txA.transactionIndex);
        });
      });
  }

  static getCallArgs (api, options, values = []) {
    const walletAddress = options.from;
    let walletContract;
    let submitMethod;

    return Promise
      .all([
        WalletsUtils.getWalletContract(api, walletAddress),
        WalletsUtils.delegateCall(api, walletAddress, 'getSubmitMethod')
      ])
      .then(([ _walletContract, _submitMethod ]) => {
        walletContract = _walletContract;
        submitMethod = _submitMethod;

        return WalletsUtils.fetchOwners(walletContract);
      })
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
          from: options.sender || ownerAddress,
          to: walletAddress,
          value: new BigNumber(0)
        };

        const execFunc = walletContract.instance[submitMethod];
        const callArgs = { func: execFunc, options: nextOptions, values: nextValues };

        if (!account.wallet) {
          return callArgs;
        }

        const nextData = walletContract.getCallData(execFunc, nextOptions, nextValues);

        return WalletsUtils.getCallArgs(api, { ...nextOptions, data: nextData }, nextValues);
      });
  }

  static getChangeMethod (api, address, change) {
    return WalletsUtils
      .delegateCall(api, address, 'getChangeMethod', [ api, address, change ]);
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

  static getWalletContract (api, address) {
    return WalletsUtils
      .delegateCall(api, address, 'getWalletContract', [ api ])
      .then((walletContract) => {
        return walletContract.at(address);
      });
  }

  static getWalletType (api, address) {
    if (_cachedWalletTypes[address] === undefined) {
      _cachedWalletTypes[address] = Promise.resolve(null)
        .then((result) => {
          if (result) {
            return result;
          }

          return FoundationWalletUtils.isWallet(api, address)
            .then((isWallet) => isWallet && FOUNDATION_WALLET);
        })
        .then((result) => {
          if (result) {
            return result;
          }

          return ConsensysWalletUtils.isWallet(api, address)
            .then((isWallet) => isWallet && CONSENSYS_WALLET);
        })
        .then((result) => {
          _cachedWalletTypes[address] = result || null;

          return _cachedWalletTypes[address];
        });
    }

    return Promise.resolve(_cachedWalletTypes[address]);
  }

  /**
   * Check whether the given address could be
   * a Wallet. The result is cached in order not
   * to make unnecessary calls on non-wallet accounts
   */
  static isWallet (api, address) {
    if (!address) {
      return Promise.resolve(false);
    }

    if (!_cachedWalletLookup[address]) {
      _cachedWalletLookup[address] = WalletsUtils.getWalletType(api, address)
        .then((walletType) => walletType !== null)
        .then((bool) => {
          _cachedWalletLookup[address] = Promise.resolve(bool);
          return bool;
        });
    }

    return _cachedWalletLookup[address];
  }

  static logToUpdate (api, address, log) {
    return WalletsUtils
      .delegateCall(api, address, 'logToUpdate', [ log ]);
  }

  static parseTransactionLogs (api, options, rawLogs) {
    return WalletsUtils
      .delegateCall(api, options.from, 'parseTransactionLogs', [ api, options, rawLogs ]);
  }

  static postModifyOperation (api, walletAddress, modification, owner, operation) {
    const options = { from: owner };
    const values = [ operation ];

    return Promise
      .all([
        WalletsUtils
          .getWalletContract(api, walletAddress),
        WalletsUtils
          .delegateCall(api, walletAddress, 'getModifyOperationMethod', [ modification ])
      ])
      .then(([ wallet, method ]) => {
        return wallet.instance[method]
          .estimateGas(options, values)
          .then((gas) => {
            options.gas = gas.mul(1.5);
            return wallet.instance[method].postTransaction(options, values);
          });
      });
  }
}
