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

import { action, computed, observable, transaction } from 'mobx';

import { ERRORS, validateAbi, validateAddress, validateName } from '~/util/validation';

import { ABI_TYPES } from './types';

export default class Store {
  @observable abi = '';
  @observable abiError = ERRORS.invalidAbi;
  @observable abiParsed = null;
  @observable abiTypes = ABI_TYPES;
  @observable abiTypeIndex = 0;
  @observable address = '';
  @observable addressError = ERRORS.invalidAddress;
  @observable description = '';
  @observable name = '';
  @observable nameError = ERRORS.invalidName;
  @observable step = 0;

  constructor (api, contracts) {
    this._api = api;
    this._contracts = contracts;

    this.setAbiTypeIndex(2);
  }

  @computed get abiType () {
    return this.abiTypes[this.abiTypeIndex];
  }

  @computed get hasError () {
    return !!(this.abiError || this.addressError || this.nameError);
  }

  @action nextStep = () => {
    this.step++;
  }

  @action prevStep = () => {
    this.step--;
  }

  @action setAbi = (_abi) => {
    const { abi, abiError, abiParsed } = validateAbi(_abi);

    transaction(() => {
      this.abi = abi;
      this.abiError = abiError;
      this.abiParsed = abiParsed;
    });
  }

  @action setAbiTypeIndex = (abiTypeIndex) => {
    transaction(() => {
      this.abiTypeIndex = abiTypeIndex;
      this.setAbi(this.abiTypes[abiTypeIndex].value);
    });
  }

  @action setAddress = (_address) => {
    let { address, addressError } = validateAddress(_address);

    if (!addressError) {
      const contract = this._contracts[address];

      if (contract) {
        addressError = ERRORS.duplicateAddress;
      }
    }

    transaction(() => {
      this.address = address;
      this.addressError = addressError;
    });
  }

  @action setDescription = (description) => {
    this.description = description;
  }

  @action setName = (_name) => {
    const { name, nameError } = validateName(_name);

    transaction(() => {
      this.name = name;
      this.nameError = nameError;
    });
  }

  addContract () {
    const meta = {
      contract: true,
      deleted: false,
      timestamp: Date.now(),
      abi: this.abiParsed,
      type: this.abiType.type,
      description: this.description
    };

    return Promise
      .all([
        this._api.parity.setAccountName(this.address, this.name),
        this._api.parity.setAccountMeta(this.address, meta)
      ])
      .catch((error) => {
        console.error('addContract', error);
        throw error;
      });
  }
}
