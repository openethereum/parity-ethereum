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

import { action, observable } from 'mobx';

import apiutil from '~/api/util';

const STAGE_SELECT_TYPE = 0;
const STAGE_CREATE = 1;
const STAGE_INFO = 2;

export default class Store {
  @observable accounts = null;
  @observable address = null;
  @observable createType = 'fromNew';
  @observable description = '';
  @observable gethAccountsAvailable = [];
  @observable gethAddresses = [];
  @observable isWindowsPhrase = false;
  @observable name = '';
  @observable nameError = null;
  @observable passwordHint = '';
  @observable phrase = '';
  @observable stage = STAGE_SELECT_TYPE;

  constructor (api, accounts) {
    this._api = api;
    this.accounts = accounts;

    this.loadAvailableGethAccounts();
  }

  @action selectGethAccount = (address) => {
    if (this.gethAddresses.includes(address)) {
      this.gethAddresses = this.gethAddresses.filter((_address) => _address !== address);
    } else {
      this.gethAddresses = [address].concat(this.gethAddresses.peek());
    }
  }

  @action setAddress = (address) => {
    this.address = address;
  }

  @action setCreateType = (createType) => {
    this.createType = createType;
  }

  @action setDescription = (description) => {
    this.description = description;
  }

  @action setGethAccountsAvailable = (gethAccountsAvailable) => {
    this.gethAccountsAvailable = [].concat(gethAccountsAvailable);
  }

  @action setIsWindowsPhrase = (isWindowsPhrase = false) => {
    this.isWindowsPhrase = isWindowsPhrase;
  }

  @action setName = (name) => {
    this.name = name;
  }

  @action setPasswordHint = (passwordHint) => {
    this.passwordHint = passwordHint;
  }

  @action setPhrase = (phrase) => {
    this.phrase = phrase;
  }

  @action setStage = (stage) => {
    this.stage = stage;
  }

  @action nextStage = () => {
    this.stage++;
  }

  @action prevStage = () => {
    this.stage--;
  }

  loadAvailableGethAccounts () {
    return this._api.parity
      .listGethAccounts()
      .then((_addresses) => {
        const addresses = (_addresses || []).filter((address) => !this.accounts[address]);

        return Promise
          .all(addresses.map((address) => this._api.eth.getBalance(address)))
          .then((balances) => {
            this.setGethAccountsAvailable(addresses.map((address, index) => {
              return {
                address,
                balance: apiutil.fromWei(balances[index]).toFormat(5)
              };
            }));
          });
      })
      .catch((error) => {
        console.warn('loadAvailableGethAccounts', error);
      });
  }
}

export {
  STAGE_CREATE,
  STAGE_INFO,
  STAGE_SELECT_TYPE
};
