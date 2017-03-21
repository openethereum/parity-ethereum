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

import * as abis from './abi';

const REGISTRY_V1_HASHES = [
  '0x34f7c51bbb1b1902fbdabfdf04811100f5c9f998f26dd535d2f6f977492c748e', // ropsten
  '0x64c3ee34851517a9faecd995c102b339f03e564ad6772dc43a26f993238b20ec' // homestead
];

export default class Registry {
  _registryContract = null;

  constructor (api) {
    this._api = api;

    this._contracts = {};
    this._pendingContracts = {};

    this._instance = null;
    this._fetching = false;
    this._queue = [];

    this.getInstance();
  }

  getInstance () {
    if (this._instance) {
      return Promise.resolve(this._instance);
    }

    if (this._fetching) {
      return new Promise((resolve) => {
        this._queue.push({ resolve });
      });
    }

    this._fetching = true;

    return this.fetchContract()
      .then((contract) => {
        this._fetching = false;
        this._instance = contract.instance;

        this._queue.forEach((queued) => {
          queued.resolve(this._instance);
        });

        this._queue = [];

        return this._instance;
      });
  }

  getContract (_name) {
    const name = _name.toLowerCase();

    if (this._contracts[name]) {
      return Promise.resolve(this._contracts[name]);
    }

    if (this._pendingContracts[name]) {
      return this._pendingContracts[name];
    }

    const promise = this
      .lookupAddress(name)
      .then((address) => {
        this._contracts[name] = this._api.newContract(abis[name], address);
        delete this._pendingContracts[name];
        return this._contracts[name];
      });

    this._pendingContracts[name] = promise;

    return promise;
  }

  getContractInstance (_name) {
    return this
      .getContract(_name)
      .then((contract) => contract.instance);
  }

  fetchContract () {
    if (this._registryContract) {
      return Promise.resolve(this._registryContract);
    }

    return this._api.parity
      .registryAddress()
      .then((address) => Promise.all([ address, this._api.eth.getCode(address) ]))
      .then(([ address, code ]) => {
        const codeHash = this._api.util.sha3(code);
        const version = REGISTRY_V1_HASHES.includes(codeHash)
          ? 1
          : 2;
        const abi = version === 1
            ? abis.registry
            : abis.registry2;
        const contract = this._api.newContract(abi, address);

        // Add support for previous `set` and `get` methods
        if (!contract.instance.get && contract.instance.getData) {
          contract.instance.get = contract.instance.getData;
        }

        if (contract.instance.get && !contract.instance.getData) {
          contract.instance.getData = contract.instance.get;
        }

        if (!contract.instance.set && contract.instance.setData) {
          contract.instance.set = contract.instance.setData;
        }

        if (contract.instance.set && !contract.instance.setData) {
          contract.instance.setData = contract.instance.set;
        }

        console.log(`registry at ${address}, code ${codeHash}, version ${version}`);
        this._registryContract = contract;
        return this._registryContract;
      });
  }

  _createGetParams (_name, key) {
    const name = _name.toLowerCase();
    const sha3 = this._api.util.sha3.text(name);

    return [sha3, key];
  }

  lookupAddress (name) {
    return this
      .getInstance()
      .then((instance) => {
        return instance.getAddress.call({}, this._createGetParams(name, 'A'));
      })
      .then((address) => {
        console.log('[lookupAddress]', `${name}: ${address}`);
        return address;
      });
  }

  lookupMeta (name, key) {
    return this
      .getInstance()
      .then((instance) => {
        return instance.get.call({}, this._createGetParams(name, key));
      });
  }
}
