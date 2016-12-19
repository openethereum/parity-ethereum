// Copyright 2015, 2016 Ethcore (UK) Ltd.
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

import Contracts from '~/contracts';
import Abi from '~/abi';
import * as abis from '~/contracts/abi';

const CONTRACT_CREATE = '0x60606040';

let instance = null;

export default class MethodDecodingStore {

  api = null;

  _isContract = {};
  _methods = {};

  constructor (api, contracts = {}) {
    this.api = api;

    // Load the signatures from the local ABIs
    Object.keys(abis).forEach((abiKey) => {
      this.loadFromAbi(abis[abiKey]);
    });

    this.addContracts(contracts);
  }

  addContracts (contracts = {}) {
    // Load the User defined contracts
    Object.values(contracts).forEach((contract) => {
      if (!contract || !contract.meta || !contract.meta.abi) {
        return;
      }
      this.loadFromAbi(contract.meta.abi);
    });
  }

  loadFromAbi (_abi) {
    const abi = new Abi(_abi);
    abi
      .functions
      .map((f) => ({ sign: f.signature, abi: f.abi }))
      .forEach((mapping) => {
        const sign = (/^0x/.test(mapping.sign) ? '' : '0x') + mapping.sign;
        this._methods[sign] = mapping.abi;
      });
  }

  static get (api, contracts = {}) {
    if (!instance) {
      instance = new MethodDecodingStore(api, contracts);
    }

    // Set API if not set yet
    if (!instance.api) {
      instance.api = api;
    }

    return instance;
  }

  static loadContracts (contracts = {}) {
    if (!instance) {
      // Just create the instance with null API
      MethodDecodingStore.get(null, contracts);
    } else {
      instance.addContracts(contracts);
    }
  }

  /**
   * Looks up a transaction in the context of the given
   * address
   *
   * @param  {String} address      The address contract
   * @param  {Object} transaction  The transaction to lookup
   * @return {Promise}             The result of the lookup. Resolves with:
   *      {
   *        contract: Boolean,
   *        deploy: Boolean,
   *        inputs: Array,
   *        name: String,
   *        params: Array,
   *        received: Boolean,
   *        signature: String
   *      }
   */
  lookup (address, transaction) {
    const result = {};

    if (!transaction) {
      return Promise.resolve(result);
    }

    const isReceived = transaction.to === address;
    const contractAddress = isReceived ? transaction.from : transaction.to;
    const input = transaction.input || transaction.data;

    result.received = isReceived;

    // No input, should be a ETH transfer
    if (!input || input === '0x') {
      return Promise.resolve(result);
    }

    const { signature, paramdata } = this.api.util.decodeCallData(input);
    result.signature = signature;
    result.params = paramdata;

    // Contract deployment
    if (!signature || signature === CONTRACT_CREATE || transaction.creates) {
      return Promise.resolve({ ...result, deploy: true });
    }

    return this
      .isContract(contractAddress || transaction.creates)
      .then((isContract) => {
        result.contract = isContract;

        if (!isContract) {
          return result;
        }

        return this
          .fetchMethodAbi(signature)
          .then((abi) => {
            let methodName = null;
            let methodInputs = null;

            if (abi) {
              methodName = abi.name;
              methodInputs = this.api.util
                .decodeMethodInput(abi, paramdata)
                .map((value, index) => {
                  const type = abi.inputs[index].type;
                  return { type, value };
                });
            }

            return {
              ...result,
              name: methodName,
              inputs: methodInputs
            };
          });
      })
      .catch((error) => {
        console.warn('lookup', error);
      });
  }

  fetchMethodAbi (signature) {
    if (this._methods[signature] !== undefined) {
      return Promise.resolve(this._methods[signature]);
    }

    this._methods[signature] = Contracts.get()
      .signatureReg
      .lookup(signature)
      .then((method) => {
        let abi = null;

        if (method && method.length) {
          abi = this.api.util.methodToAbi(method);
        }

        this._methods[signature] = abi;
        return this._methods[signature];
      });

    return Promise.resolve(this._methods[signature]);
  }

  /**
   * Checks (and caches) if the given address is a
   * Contract or not, from its fetched bytecode
   */
  isContract (contractAddress) {
    // If zero address, it isn't a contract
    if (/^(0x)?0*$/.test(contractAddress)) {
      return Promise.resolve(false);
    }

    if (this._isContract[contractAddress]) {
      return Promise.resolve(this._isContract[contractAddress]);
    }

    this._isContract[contractAddress] = this.api.eth
      .getCode(contractAddress)
      .then((bytecode) => {
        // Is a contract if the address contains *valid* bytecode
        const _isContract = bytecode && /^(0x)?([0]*[1-9a-f]+[0]*)+$/.test(bytecode);

        this._isContract[contractAddress] = _isContract;
        return this._isContract[contractAddress];
      });

    return Promise.resolve(this._isContract[contractAddress]);
  }

}
