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
    });
}

export function postTransaction (_func, _options, _values = []) {
  return getTxArgs(_func, _options, _values)
    .then((callArgs) => {
      const { func, options, values } = callArgs;

      return func._postTransaction(options, values);
    });
}

export function deploy (contract, _options, values, metadata = {}, statecb = () => {}) {
  const options = { ..._options };
  const { api } = contract;
  const address = options.from;

  return WalletsUtils
    .isWallet(api, address)
    .then((isWallet) => {
      if (!isWallet) {
        return contract.deploy(options, values, statecb);
      }

      statecb(null, { state: 'estimateGas' });

      return deployEstimateGas(contract, options, values)
        .then(([gasEst, gas]) => {
          options.gas = gas.toFixed(0);

          statecb(null, { state: 'postTransaction', gas });

          return WalletsUtils.getDeployArgs(contract, options, values);
        })
        .then((callArgs) => {
          const { func, options, values } = callArgs;

          return func._postTransaction(options, values)
            .then((requestId) => {
              statecb(null, { state: 'checkRequest', requestId });
              return contract._pollCheckRequest(requestId);
            })
            .then((txhash) => {
              statecb(null, { state: 'getTransactionReceipt', txhash });
              return contract._pollTransactionReceipt(txhash, options.gas);
            })
            .then((receipt) => {
              if (receipt.gasUsed.eq(options.gas)) {
                throw new Error(`Contract not deployed, gasUsed == ${options.gas.toFixed(0)}`);
              }

              const logs = WalletsUtils.parseLogs(api, receipt.logs || []);

              const confirmationLog = logs.find((log) => log.event === 'ConfirmationNeeded');
              const transactionLog = logs.find((log) => log.event === 'SingleTransact');

              if (!confirmationLog && !transactionLog) {
                throw new Error('Something went wrong in the Wallet Contract (no logs have been emitted)...');
              }

              // Confirmations are needed from the other owners
              if (confirmationLog) {
                const operationHash = api.util.bytesToHex(confirmationLog.params.operation.value);

                // Add the contract to pending contracts
                WalletsUtils.addPendingContract(address, operationHash, metadata);
                statecb(null, { state: 'confirmationNeeded' });
                return;
              }

              // Set the contract address in the receip
              receipt.contractAddress = transactionLog.params.created.value;

              const contractAddress = receipt.contractAddress;

              statecb(null, { state: 'hasReceipt', receipt });
              contract._receipt = receipt;
              contract._address = contractAddress;

              statecb(null, { state: 'getCode' });

              return api.eth.getCode(contractAddress)
                .then((code) => {
                  if (code === '0x') {
                    throw new Error('Contract not deployed, getCode returned 0x');
                  }

                  statecb(null, { state: 'completed' });
                  return contractAddress;
                });
            });
        });
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
