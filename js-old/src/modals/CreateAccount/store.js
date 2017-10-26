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
const STAGE_CONFIRM_BACKUP = 3;

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
  @observable backupPhraseAddress = null;
  @observable phraseBackedUp = '';
  @observable qrAddress = null;
  @observable rawKey = '';
  @observable rawKeyError = ERRORS.nokey;
  @observable stage = STAGE_SELECT_TYPE;
  @observable vaultName = '';
  @observable walletFile = '';
  @observable walletFileError = ERRORS.noFile;
  @observable walletJson = '';

  constructor (api, accounts, isTest, loadGeth = true) {
    this._api = api;
    this.accounts = Object.assign({}, accounts);
    this.isTest = isTest;

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
        return !(this.nameError || this.passwordRepeatError) && this.hasAddress && this.hasPhrase;

      case 'fromPhrase':
        return !(this.nameError || this.passwordRepeatError || this.passPhraseError);

      case 'fromQr':
        return this.qrAddressValid && !this.nameError;

      case 'fromRaw':
        return !(this.nameError || this.passwordRepeatError || this.rawKeyError);

      default:
        return false;
    }
  }

  @computed get passPhraseError () {
    return !this.isTest && this.phrase.length === 0;
  }

  @computed get hasAddress () {
    return !!(this.address);
  }

  @computed get hasPhrase () {
    return this.phrase.length !== 0;
  }

  @computed get passwordRepeatError () {
    return this.password === this.passwordRepeat
      ? null
      : ERRORS.noMatchPassword;
  }

  @computed get backupPhraseError () {
    return !this.backupPhraseAddress || this.address === this.backupPhraseAddress
      ? null
      : ERRORS.noMatchBackupPhrase;
  }

  @computed get phraseBackedUpError () {
    return this.phraseBackedUp === 'I have written down the phrase'
      ? null
      : ERRORS.noMatchPhraseBackedUp;
  }

  @computed get qrAddressValid () {
    return this._api.util.isAddressValid(this.qrAddress);
  }

  @action clearPhrase = () => {
    transaction(() => {
      this.phrase = '';
      this.phraseBackedUp = '';
    });
  }

  @action clearErrors = () => {
    transaction(() => {
      this.address = '';
      this.description = '';
      this.password = '';
      this.passwordRepeat = '';
      this.phrase = '';
      this.backupPhraseAddress = null;
      this.phraseBackedUp = '';
      this.name = '';
      this.nameError = ERRORS.noName;
      this.qrAddress = null;
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

  @action setQrAddress = (qrAddress) => {
    if (qrAddress && qrAddress.substr(0, 2) !== '0x') {
      qrAddress = `0x${qrAddress}`;
    }

    // FIXME: Current native signer encoding is not 100% for EIP-55, lowercase for now
    this.qrAddress = qrAddress && this._api.util
        ? this._api.util.toChecksumAddress(qrAddress.toLowerCase())
        : qrAddress;
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

  @action setBackupPhraseAddress = (address) => {
    this.backupPhraseAddress = address;
  }

  @action computeBackupPhraseAddress = () => {
    return this._api.parity.phraseToAddress(this.phrase)
      .then(address => {
        this.setBackupPhraseAddress(address);
        return address !== this.address;
      })
      .catch((error) => {
        console.error('createAccount', error);
        throw error;
      });
  }

  @action setPhraseBackedUp = (backedUp) => {
    this.phraseBackedUp = backedUp;
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
    this.backupPhraseAddress = null;
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
    if (this.stage === 0) {
      this.clearErrors();
    }

    this.stage++;
  }

  @action prevStage = () => {
    this.stage--;
  }

  createAccount = (vaultStore) => {
    if (!this.canCreate) {
      return false;
    }

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

      case 'fromQr':
        return this.createAccountFromQr();

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

        this.setName('Geth Import');
        this.setDescription('Imported from Geth keystore');
        this.setGethImported(gethImported);

        return Promise.all(gethImported.map((address) => {
          return this.setupMeta(address, timestamp);
        }));
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

        return this.setupMeta(address, timestamp);
      })
      .catch((error) => {
        console.error('createAccount', error);
        throw error;
      });
  }

  createAccountFromQr = (timestamp = Date.now()) => {
    this.setAddress(this.qrAddress);

    return this.setupMeta(this.qrAddress, timestamp, { external: true });
  }

  createAccountFromRaw = (timestamp = Date.now()) => {
    return this._api.parity
      .newAccountFromSecret(this.rawKey, this.password)
      .then((address) => {
        this.setAddress(address);

        return this.setupMeta(address, timestamp);
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

        return this.setupMeta(address, timestamp);
      })
      .catch((error) => {
        console.error('createAccount', error);
        throw error;
      });
  }

  setupMeta = (address, timestamp = Date.now(), extra = {}) => {
    const meta = Object.assign({}, extra, {
      description: this.description,
      passwordHint: this.passwordHint,
      timestamp
    });

    return this._api.parity
      .setAccountName(address, this.name)
      .then(() => this._api.parity.setAccountMeta(address, meta));
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
  STAGE_INFO,
  STAGE_CONFIRM_BACKUP,
  STAGE_CREATE,
  STAGE_SELECT_TYPE
};
