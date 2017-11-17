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

import Contracts from '~/contracts';
import Abi from '@parity/abi';
import * as abis from '~/contracts/abi';

import { decodeMethodInput } from '@parity/api/lib/util/decode';

const CONTRACT_CREATE = '0x60606040';

let instance = null;

export default class MethodDecodingStore {
  api = null;

  _bytecodes = {};
  _contractsAbi = {};
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
      this.loadFromAbi(contract.meta.abi, contract.address);
    });
  }

  loadFromAbi (_abi, contractAddress) {
    let abi;

    try {
      abi = new Abi(_abi);
    } catch (error) {
      console.warn('loadFromAbi', error, _abi);
    }

    if (!abi) {
      return;
    }

    if (contractAddress) {
      this._contractsAbi[contractAddress] = abi;
    }

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
  lookup (currentAddress, transaction) {
    const result = {};

    if (!transaction) {
      return Promise.resolve(result);
    }

    const isReceived = transaction.to === currentAddress;
    const contractAddress = isReceived ? transaction.from : transaction.to;
    const input = transaction.input || transaction.data;

    result.input = input;
    result.received = isReceived;

    // No input, should be a ETH transfer
    if (!input || input === '0x') {
      return Promise.resolve(result);
    }

    if (!transaction.to) {
      return this.decodeContractCreation(result);
    }

    let signature;

    try {
      const decodeCallDataResult = this.api.util.decodeCallData(input);

      signature = decodeCallDataResult.signature;
    } catch (e) {}

    // Contract deployment
    if (!signature || signature === CONTRACT_CREATE || transaction.creates) {
      const address = contractAddress || transaction.creates;

      return this.isContractCreation(input, address)
        .then((isContractCreation) => {
          if (!isContractCreation) {
            result.contract = false;
            result.deploy = false;

            return result;
          }

          return this.decodeContractCreation(result, address);
        });
    }

    return this
      .isContract(contractAddress)
      .then((isContract) => {
        result.contract = isContract;

        if (!isContract) {
          return result;
        }

        const { signature, paramdata } = this.api.util.decodeCallData(input);

        result.signature = signature;
        result.params = paramdata;

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
                  const { name, type } = abi.inputs[index];

                  return { name, type, value };
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

  decodeContractCreation (data, contractAddress = '') {
    const result = {
      ...data,
      contract: true,
      deploy: true
    };

    const { input } = data;
    const abi = this._contractsAbi[contractAddress];

    if (!abi || !abi.constructors || abi.constructors.length === 0) {
      return Promise.resolve(result);
    }

    const constructorAbi = abi.constructors[0];

    const rawInput = /^(?:0x)?(.*)$/.exec(input)[1];

    return this
      .getCode(contractAddress)
      .then((code) => {
        if (!code || /^(0x)0*?$/.test(code)) {
          return result;
        }

        const rawCode = /^(?:0x)?(.*)$/.exec(code)[1];
        const codeOffset = rawInput.indexOf(rawCode);

        if (codeOffset === -1) {
          return result;
        }

        // Params are the last bytes of the transaction Input
        // (minus the bytecode). It seems that they are repeated
        // twice
        const params = rawInput.slice(codeOffset + rawCode.length);
        const paramsBis = params.slice(params.length / 2);

        let decodedInputs;

        try {
          decodedInputs = decodeMethodInput(constructorAbi, params);
        } catch (e) {}

        try {
          if (!decodedInputs) {
            decodedInputs = decodeMethodInput(constructorAbi, paramsBis);
          }
        } catch (e) {}

        if (decodedInputs && decodedInputs.length > 0) {
          result.inputs = decodedInputs
            .map((value, index) => {
              const type = constructorAbi.inputs[index].kind.type;

              return { type, value };
            });
        }

        return result;
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
    if (!contractAddress || /^(0x)?0*$/.test(contractAddress)) {
      return Promise.resolve(false);
    }

    if (this._isContract[contractAddress]) {
      return Promise.resolve(this._isContract[contractAddress]);
    }

    this._isContract[contractAddress] = this
      .getCode(contractAddress)
      .then((bytecode) => {
        // Is a contract if the address contains *valid* bytecode
        const _isContract = bytecode && /^(0x)?([0]*[1-9a-f]+[0]*)+$/.test(bytecode);

        this._isContract[contractAddress] = _isContract;
        return this._isContract[contractAddress];
      });

    return Promise.resolve(this._isContract[contractAddress]);
  }

  /**
   * Check if the input resulted in a contract creation
   * by checking that the contract address code contains
   * a part of the input, or vice-versa
   */
  isContractCreation (input, contractAddress) {
    return this.api.eth
      .getCode(contractAddress)
      .then((code) => {
        if (/^(0x)?0*$/.test(code)) {
          return false;
        }

        const strippedCode = code.replace(/^0x/, '');
        const strippedInput = input.replace(/^0x/, '');

        return strippedInput.indexOf(strippedInput) >= 0 || strippedCode.indexOf(strippedInput) >= 0;
      })
      .catch((error) => {
        console.error(error);
        return false;
      });
  }

  getCode (contractAddress) {
    // If zero address, resolve to '0x'
    if (!contractAddress || /^(0x)?0*$/.test(contractAddress)) {
      return Promise.resolve('0x');
    }

    if (this._bytecodes[contractAddress]) {
      return Promise.resolve(this._bytecodes[contractAddress]);
    }

    this._bytecodes[contractAddress] = this.api.eth
      .getCode(contractAddress)
      .then((bytecode) => {
        this._bytecodes[contractAddress] = bytecode;
        return this._bytecodes[contractAddress];
      });

    return Promise.resolve(this._bytecodes[contractAddress]);
  }
}
