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
  @observable gethImported = [];
  @observable isBusy = false;
  @observable isWindowsPhrase = false;
  @observable name = '';
  @observable nameError = ERRORS.noName;
  @observable password = '';
  @observable passwordHint = '';
  @observable passwordRepeat = '';
  @observable phrase = '';
  @observable rawKey = '';
  @observable rawKeyError = ERRORS.nokey;
  @observable stage = STAGE_SELECT_TYPE;
  @observable vaultName = '';
  @observable walletFile = '';
  @observable walletFileError = ERRORS.noFile;
  @observable walletJson = '';

  constructor (api, accounts, loadGeth = true) {
    this._api = api;
    this.accounts = Object.assign({}, accounts);

    if (loadGeth) {
      this.loadAvailableGethAccounts();
    }
  }

  @computed get canCreate () {
    switch (this.createType) {
      case 'fromGeth':
        return this.gethAddresses.length !== 0;

      case 'fromJSON':
      case 'fromPresale':
        return !(this.nameError || this.walletFileError);

      case 'fromNew':
        return !(this.nameError || this.passwordRepeatError);

      case 'fromPhrase':
        return !(this.nameError || this.passwordRepeatError);

      case 'fromRaw':
        return !(this.nameError || this.passwordRepeatError || this.rawKeyError);

      default:
        return false;
    }
  }

  @computed get passwordRepeatError () {
    return this.password === this.passwordRepeat
      ? null
      : ERRORS.noMatchPassword;
  }

  @action clearErrors = () => {
    transaction(() => {
      this.password = '';
      this.passwordRepeat = '';
      this.phrase = '';
      this.name = '';
      this.nameError = null;
      this.rawKey = '';
      this.rawKeyError = null;
      this.vaultName = '';
      this.walletFile = '';
      this.walletFileError = null;
      this.walletJson = '';
    });
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

  @action setBusy = (isBusy) => {
    this.isBusy = isBusy;
  }

  @action setCreateType = (createType) => {
    this.clearErrors();
    this.createType = createType;
  }

  @action setDescription = (description) => {
    this.description = description;
  }

  @action setGethAccountsAvailable = (gethAccountsAvailable) => {
    this.gethAccountsAvailable = [].concat(gethAccountsAvailable);
  }

  @action setGethImported = (gethImported) => {
    this.gethImported = gethImported;
  }

  @action setVaultName = (vaultName) => {
    this.vaultName = vaultName;
  }

  @action setWindowsPhrase = (isWindowsPhrase = false) => {
    this.isWindowsPhrase = isWindowsPhrase;
  }

  @action setName = (name) => {
    let nameError = null;

    if (!name || !name.trim().length) {
      nameError = ERRORS.noName;
    }

    transaction(() => {
      this.name = name;
      this.nameError = nameError;
    });
  }

  @action setPassword = (password) => {
    this.password = password;
  }

  @action setPasswordHint = (passwordHint) => {
    this.passwordHint = passwordHint;
  }

  @action setPasswordRepeat = (passwordRepeat) => {
    this.passwordRepeat = passwordRepeat;
  }

  @action setPhrase = (phrase) => {
    const recoveryPhrase = phrase
      .toLowerCase() // wordlists are lowercase
      .trim() // remove whitespace at both ends
      .replace(/\s/g, ' ') // replace any whitespace with single space
      .replace(/ +/g, ' '); // replace multiple spaces with a single space

    const phraseParts = recoveryPhrase
      .split(' ')
      .map((part) => part.trim())
      .filter((part) => part.length);

    this.phrase = phraseParts.join(' ');
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
      this.walletJson = null;
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

  createAccount = (vaultStore) => {
    this.setBusy(true);

    return this
      ._createAccount()
      .then(() => {
        if (vaultStore && this.vaultName && this.vaultName.length) {
          return vaultStore.moveAccount(this.vaultName, this.address);
        }

        return true;
      })
      .then(() => {
        this.setBusy(false);
      })
      .catch((error) => {
        this.setBusy(false);
        throw error;
      });
  }

  _createAccount = () => {
    switch (this.createType) {
      case 'fromGeth':
        return this.createAccountFromGeth();

      case 'fromJSON':
      case 'fromPresale':
        return this.createAccountFromWallet();

      case 'fromNew':
      case 'fromPhrase':
        return this.createAccountFromPhrase();

      case 'fromRaw':
        return this.createAccountFromRaw();

      default:
        throw new Error(`Cannot create account for ${this.createType}`);
    }
  }

  createAccountFromGeth = (timestamp = Date.now()) => {
    return this._api.parity
      .importGethAccounts(this.gethAddresses.peek())
      .then((gethImported) => {
        console.log('createAccountFromGeth', gethImported);

        this.setGethImported(gethImported);

        return Promise
          .all(gethImported.map((address) => {
            return this._api.parity.setAccountName(address, 'Geth Import');
          }))
          .then(() => {
            return Promise.all(gethImported.map((address) => {
              return this._api.parity.setAccountMeta(address, { timestamp });
            }));
          });
      })
      .catch((error) => {
        console.error('createAccountFromGeth', error);
        throw error;
      });
  }

  createAccountFromPhrase = (timestamp = Date.now()) => {
    let formattedPhrase = this.phrase;

    if (this.isWindowsPhrase && this.createType === 'fromPhrase') {
      formattedPhrase = this.phrase
        .split(' ') // get the words
        .map((word) => word === 'misjudged' ? word : `${word}\r`) // add \r after each (except last in dict)
        .join(' '); // re-create string
    }

    return this._api.parity
      .newAccountFromPhrase(formattedPhrase, this.password)
      .then((address) => {
        this.setAddress(address);

        return this._api.parity
          .setAccountName(address, this.name)
          .then(() => this._api.parity.setAccountMeta(address, {
            passwordHint: this.passwordHint,
            timestamp
          }));
      })
      .catch((error) => {
        console.error('createAccount', error);
        throw error;
      });
  }

  createAccountFromRaw = (timestamp = Date.now()) => {
    return this._api.parity
      .newAccountFromSecret(this.rawKey, this.password)
      .then((address) => {
        this.setAddress(address);

        return this._api.parity
          .setAccountName(address, this.name)
          .then(() => this._api.parity.setAccountMeta(address, {
            passwordHint: this.passwordHint,
            timestamp
          }));
      })
      .catch((error) => {
        console.error('createAccount', error);
        throw error;
      });
  }

  createAccountFromWallet = (timestamp = Date.now()) => {
    return this._api.parity
      .newAccountFromWallet(this.walletJson, this.password)
      .then((address) => {
        this.setAddress(address);

        return this._api.parity
          .setAccountName(address, this.name)
          .then(() => this._api.parity.setAccountMeta(address, {
            passwordHint: this.passwordHint,
            timestamp
          }));
      })
      .catch((error) => {
        console.error('createAccount', error);
        throw error;
      });
  }

  createIdentities = () => {
    return Promise
      .all([
        this._api.parity.generateSecretPhrase(),
        this._api.parity.generateSecretPhrase(),
        this._api.parity.generateSecretPhrase(),
        this._api.parity.generateSecretPhrase(),
        this._api.parity.generateSecretPhrase(),
        this._api.parity.generateSecretPhrase(),
        this._api.parity.generateSecretPhrase()
      ])
      .then((phrases) => {
        return Promise
          .all(phrases.map((phrase) => this._api.parity.phraseToAddress(phrase)))
          .then((addresses) => {
            return phrases.reduce((accounts, phrase, index) => {
              const address = addresses[index];

              accounts[address] = {
                address,
                phrase
              };

              return accounts;
            }, {});
          });
      })
      .catch((error) => {
        console.error('createIdentities', error);
        throw error;
      });
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
