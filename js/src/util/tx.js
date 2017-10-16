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

import WalletsUtils from '~/util/wallets';

/**
 * The sender is by default (when the UI loads) the
 * default dapp address. It can then be modified when
 * sending transactions....
 */
let currentSender = '';
let hasCurrentSenderChanged = false;

export function getSender () {
  currentSender;
}

export function loadSender (api) {
  // If the current sender has been changed
  // then don't bother checking changes of the
  // default sender
  if (hasCurrentSenderChanged) {
    return Promise.resolve(currentSender);
  }

  return api.parity.getNewDappsDefaultAddress()
    .then((defaultAccount) => {
      currentSender = defaultAccount;
      return defaultAccount;
    });
}

export function setSender (sender) {
  currentSender = sender;
  hasCurrentSenderChanged = true;
}

export function trackRequest (api, options, statusCallback) {
  const { requestId, transactionHash } = options;
  const txHashPromise = transactionHash
    ? Promise.resolve(transactionHash)
    : api.pollMethod('parity_checkRequest', requestId);

  return txHashPromise
    .then((transactionHash) => {
      statusCallback(null, { transactionHash });
      return api.pollMethod('eth_getTransactionReceipt', transactionHash, isValidReceipt);
    })
    .then((transactionReceipt) => {
      statusCallback(null, { transactionReceipt });
    })
    .catch((error) => {
      statusCallback(error);
    });
}

const isValidReceipt = (receipt) => {
  return receipt && receipt.blockNumber && receipt.blockNumber.gt(0);
};

export function getTxOptions (api, func, _options, values = []) {
  const options = { ..._options };
  const address = options.from;

  if (func && func.contract) {
    options.to = options.to || func.contract.address;
  }

  const promise = (!address)
    ? Promise.resolve(false)
    : WalletsUtils.isWallet(api, address);

  return promise
    .then((isWallet) => {
      if (!isWallet) {
        return { func, options, values };
      }

      if (func && func.contract) {
        options.data = func.contract.getCallData(func, options, values);
      }

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
    })
    .then(({ func, options, values }) => {
      if (func) {
        options.data = func.contract.getCallData(func, options, values);
      }

      if (!options.value) {
        options.value = new BigNumber(0);
      }

      return options;
    });
}

export function estimateGas (_func, _options, _values = []) {
  const { api } = _func.contract;

  return getTxOptions(api, _func, _options, _values)
    .then((options) => {
      return api.eth.estimateGas(options);
    });
}

export function postTransaction (_func, _options, _values = []) {
  const { api } = _func.contract;

  return getTxOptions(api, _func, _options, _values)
    .then((options) => {
      return api.parity.postTransaction(options);
    });
}

export function deployEstimateGas (contract, _options, values) {
  const options = { ..._options };
  const { api } = contract;
  const address = options.from;

  return WalletsUtils
    .isWallet(api, address)
    .then((isWallet) => {
      if (!isWallet) {
        return contract.deployEstimateGas(options, values);
      }

      return WalletsUtils
        .getDeployArgs(contract, options, values)
        .then((callArgs) => {
          const { func, options, values } = callArgs;

          return func.estimateGas(options, values);
        })
        .then((gasEst) => {
          return [gasEst, gasEst.mul(1.05)];
        });
    });
}

export function deploy (contract, options, values, skipGasEstimate = false) {
  const { api } = contract;
  const address = options.from;

  const gasEstPromise = skipGasEstimate
    ? Promise.resolve(null)
    : deployEstimateGas(contract, options, values).then(([gasEst, gas]) => gas);

  return gasEstPromise
    .then((gas) => {
      if (gas) {
        options.gas = gas.toFixed(0);
      }

      return WalletsUtils.isWallet(api, address);
    })
    .then((isWallet) => {
      if (!isWallet) {
        const encodedOptions = contract._encodeOptions(contract.constructors[0], options, values);

        return api.parity.postTransaction(encodedOptions);
      }

      return WalletsUtils.getDeployArgs(contract, options, values)
        .then((callArgs) => {
          const { func, options, values } = callArgs;

          return func._postTransaction(options, values);
        });
    });
}

export function parseTransactionReceipt (api, options, receipt) {
  if (receipt.gasUsed.eq(options.gas)) {
    const error = new Error(`Contract not deployed, gasUsed == ${options.gas.toFixed(0)}`);

    return Promise.reject(error);
  }

  // If regular contract creation, only validate the contract
  if (receipt.contractAddress) {
    return validateContract(api, receipt.contractAddress);
  }

  // Otherwise, needs to check for a contract deployment
  // from a multisig wallet
  const walletResult = WalletsUtils.parseTransactionLogs(api, options, receipt.logs || []);

  if (!walletResult) {
    const error = new Error('Something went wrong in the contract deployment...');

    return Promise.reject(error);
  }

  if (walletResult.pending) {
    return Promise.resolve(null);
  }

  return validateContract(api, walletResult.contractAddress);
}

function validateContract (api, contractAddress) {
  return api.eth
    .getCode(contractAddress)
    .then((code) => {
      if (code === '0x') {
        throw new Error('Contract not deployed, getCode returned 0x');
      }

      return contractAddress;
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
