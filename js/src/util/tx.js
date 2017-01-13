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

import WalletsUtils from '~/util/wallets';

const isValidReceipt = (receipt) => {
  return receipt && receipt.blockNumber && receipt.blockNumber.gt(0);
};

function getTxArgs (func, options, values = []) {
  const { contract } = func;
  const { api } = contract;
  const address = options.from;

  if (!address) {
    return Promise.resolve({ func, options, values });
  }

  return WalletsUtils
    .isWallet(api, address)
    .then((isWallet) => {
      if (!isWallet) {
        return { func, options, values };
      }

      options.data = contract.getCallData(func, options, values);
      options.to = options.to || contract.address;

      if (!options.to) {
        return { func, options, values };
      }

      return WalletsUtils
        .getCallArgs(api, options, values)
        .then((callArgs) => {
          if (!callArgs) {
            return { func, options, values };
          }

          return callArgs;
        });
    });
}

export function estimateGas (_func, _options, _values = []) {
  return getTxArgs(_func, _options, _values)
    .then((callArgs) => {
      const { func, options, values } = callArgs;
      return func._estimateGas(options, values);
    })
    .then((gas) => {
      return WalletsUtils
        .isWallet(_func.contract.api, _options.from)
        .then((isWallet) => {
          if (isWallet) {
            return gas.mul(1.5);
          }

          return gas;
        });
    });
}

export function postTransaction (_func, _options, _values = []) {
  return getTxArgs(_func, _options, _values)
    .then((callArgs) => {
      const { func, options, values } = callArgs;
      return func._postTransaction(options, values);
    });
}

export function patchApi (api) {
  api.patch = {
    ...api.patch,
    contract: patchContract
  };
}

export function patchContract (contract) {
  contract._functions.forEach((func) => {
    if (!func.constant) {
      func._postTransaction = func.postTransaction;
      func._estimateGas = func.estimateGas;

      func.postTransaction = postTransaction.bind(contract, func);
      func.estimateGas = estimateGas.bind(contract, func);
    }
  });
}

export function checkIfTxFailed (api, tx, gasSent) {
  return api.pollMethod('eth_getTransactionReceipt', tx)
  .then((receipt) => {
    // TODO: Right now, there's no way to tell wether the EVM code crashed.
    // Because you usually send a bit more gas than estimated (to make sure
    // it gets mined quickly), we transaction probably failed if all the gas
    // has been used up.
    return receipt.gasUsed.eq(gasSent);
  });
}

export function waitForConfirmations (api, tx, confirmations) {
  return new Promise((resolve, reject) => {
    api.pollMethod('eth_getTransactionReceipt', tx, isValidReceipt)
    .then((receipt) => {
      let subscription;
      api.subscribe('eth_blockNumber', (err, block) => {
        if (err) {
          reject(err);
        } else if (block.minus(confirmations - 1).gte(receipt.blockNumber)) {
          if (subscription) {
            api.unsubscribe(subscription);
          }
          resolve();
        }
      })
      .then((_subscription) => {
        subscription = _subscription;
      })
      .catch(reject);
    });
  });
}
