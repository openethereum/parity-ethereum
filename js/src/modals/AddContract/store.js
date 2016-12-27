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

import { computed, observable } from 'mobx';

import { ERRORS, validateAbi, validateAddress, validateName } from '~/util/validation';

import { ABI_TYPES } from './types';

export default class Store {
  @observable abi = '';
  @observable abiError = ERRORS.invalidAbi;
  @observable abiTypes = ABI_TYPES;
  @observable abiTypeIndex = 2;
  @observable address = '';
  @observable addressError = ERRORS.invalidAddress;
  @observable description = null;
  @observable name = '';
  @observable nameError = ERRORS.invalidName;

  constructor (api, abiTypes) {
    this._api = api;
  }

  @computed get abiType () {
    return this.abiTypes[this.abiTypeIndex];
  }

  addContract () {
    const { abiParsed, abiType } = this.state;

    const meta = {
      contract: true,
      deleted: false,
      timestamp: Date.now(),
      abi: abiParsed,
      type: abiType.type,
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
