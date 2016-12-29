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

import { action, observable, transaction } from 'mobx';

import apiutil from '~/api/util';

import ERRORS from './errors';

const FAKEPATH = 'C:\\fakepath\\';
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
  @observable nameError = ERRORS.noName;
  @observable passwordHint = '';
  @observable phrase = '';
  @observable rawKey = '';
  @observable rawKeyError = ERRORS.nokey;
  @observable stage = STAGE_SELECT_TYPE;
  @observable walletFile = '';
  @observable walletFileError = ERRORS.noFile;
  @observable walletJson = '';

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
    let nameError = null;

    if (!name || !name.trim().length) {
      nameError = ERRORS.noName;
    }

    this.name = name;
  }

  @action setPasswordHint = (passwordHint) => {
    this.passwordHint = passwordHint;
  }

  @action setPhrase = (phrase) => {
    this.phrase = phrase;
  }

  @action setRawKey = (rawKey) => {
    let rawKeyError = null;

    if (!rawKey || !rawKey.trim().length) {
      rawKeyError = ERRORS.noKey;
    } else if (rawKey.substr(0, 2) !== '0x' || rawKey.substr(2).length !== 64 || !apiutil.isHex(rawKey)) {
      rawKeyError = ERRORS.invalidKey;
    }

    transaction(() => {
      this.rawKey = rawKey;
      this.rawKeyError = rawKeyError;
    });
  }

  @action setStage = (stage) => {
    this.stage = stage;
  }

  @action setWalletFile = (walletFile) => {
    transaction(() => {
      this.walletFile = walletFile.replace(FAKEPATH, '');
      this.walletFileError = ERRORS.noFile;
    });
  }

  @action setWalletJson = (walletJson) => {
    transaction(() => {
      this.walletFileError = null;
      this.walletJson = walletJson;
    });
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
