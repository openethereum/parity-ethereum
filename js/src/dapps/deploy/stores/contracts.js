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
import { action, observable, transaction } from 'mobx';

import * as abis from '~/contracts/abi';

import { api } from '../parity';

const CONTRACTS = [
  {
    id: 'badgereg',
    abi: abis.badgereg
  },
  {
    id: 'dappreg',
    abi: abis.dappreg
  },
  {
    id: 'githubhint',
    abi: abis.githubhint
  },
  {
    id: 'tokenreg',
    abi: abis.tokenreg
  }
];

export default class ContractsStore {
  @observable error = null;
  @observable registryAddress = null;
  @observable registryInstance = null;
  @observable contracts = CONTRACTS;

  constructor () {
    api.subscribe('eth_blockNumber', this.onNewBlockNumber);
  }

  @action refreshContracts = () => {
    this.contracts = [].concat(this.contracts.peek());
  }

  @action setError = (error) => {
    this.error = error.message
      ? error.message
      : error;
  }

  @action setRegistryAddress = (address) => {
    console.log(`registry found at ${address}`);

    transaction(() => {
      this.registryAddress = address;
      this.registryInstance = api.newContract(abis.registry, address).instance;
    });
  }

  @action setContractAddress = (contract, address) => {
    console.log(`${contract.id} found at ${address}`);

    transaction(() => {
      contract.address = address;
      contract.instance = api.newContract(contract.abi, address).instance;
    });
  }

  isValidAddress (address) {
    return !(new BigNumber(address)).isZero();
  }

  findRegistry () {
    if (this.registryAddress) {
      return Promise.resolve(this.registryAddress);
    }

    return api.parity
      .registryAddress()
      .then((address) => {
        if (!this.isValidAddress(address)) {
          throw new Error('Unable to locate registry');
        }

        this.setRegistryAddress(address);
      });
  }

  findContracts () {
    return Promise
      .all(
        this.contracts.map((contract) => {
          return contract.instance
            ? Promise.resolve(0)
            : this.queryRegistryAddress(contract.id);
        })
      )
      .then((addresses) => {
        const errorred = [];

        addresses.forEach((address, index) => {
          const contract = this.contracts[index];

          if (this.isValidAddress(address)) {
            this.setContractAddress(contract, address);
          } else if (!contract.instance) {
            errorred.push(contract.id);
          }
        });

        this.refreshContracts();

        if (errorred.length) {
          throw new Error(`Unable to find contracts for: ${errorred.join(', ')}`);
        }
      });
  }

  queryRegistryAddress = (name) => {
    return this.registryInstance.getAddress.call({}, [api.util.sha3(name), 'A']);
  }

  onNewBlockNumber = (error, blockNumber) => {
    if (error) {
      return;
    }

    return this
      .findRegistry()
      .then(this.findContracts)
      .catch(this.setError);
  }
}
