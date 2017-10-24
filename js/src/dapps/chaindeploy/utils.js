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

import { api } from './parity';

export function validateCode (source, retrieved) {
  const original = source.substr(12);
  const bytecode = retrieved.substr(12);

  const knownHash = api.util.sha3(original.slice(-1 * bytecode.length));
  const codeHash = api.util.sha3(bytecode);

  return knownHash === codeHash;
}

export function isValidNumber (number) {
  return number && !(new BigNumber(number)).isZero();
}

export function executeContract (logId, contract, funcName, options, values) {
  const func = contract.instance[funcName];

  return func
    .estimateGas(options, values)
    .then((gasEst) => {
      options.gas = gasEst.mul(1.2);

      return trackRequest(
        func.postTransaction(options, values),
        (error, data) => {
          if (error) {
            console.error(logId, error);
          } else {
            console.log(logId, data);
          }
        }
      );
    });
}

export function trackRequest (promise, callback) {
  return promise
    .then((requestId) => {
      callback(null, { state: 'checkRequest', requestId });

      return api.pollMethod('parity_checkRequest', requestId);
    })
    .then((txHash) => {
      callback(null, { state: 'getTransactionReceipt', txHash });

      return api.pollMethod('eth_getTransactionReceipt', txHash, (receipt) => {
        if (!receipt || !receipt.blockNumber || receipt.blockNumber.eq(0)) {
          return false;
        }

        return true;
      });
    })
    .then((receipt) => {
      callback(null, { state: 'hasReceipt', receipt });
    })
    .catch((error) => {
      callback(error);

      throw error;
    });
}
