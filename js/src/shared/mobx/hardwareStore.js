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

import Ledger from '~/3rdparty/ledger';

const HW_SCAN_INTERVAL = 5000;
let instance = null;

export default class HardwareStore {
  @observable isScanning = false;
  @observable wallets = {};

  constructor (api) {
    this._api = api;
    this._ledger = Ledger.create(api);
    this._pollId = null;

    this._pollScan();
  }

  isConnected (address) {
    return computed(() => !!this.wallets[address]).get();
  }

  @action setScanning = (isScanning) => {
    this.isScanning = isScanning;
  }

  @action setWallets = (wallets) => {
    this.wallets = wallets;
  }

  _pollScan = () => {
    this._pollId = setTimeout(() => {
      this.scan().then(this._pollScan);
    }, HW_SCAN_INTERVAL);
  }

  scanLedger () {
    if (!this._ledger.isSupported) {
      return Promise.resolve({});
    }

    return this._ledger
      .scan()
      .then((wallets) => {
        console.log('HardwareStore::scanLedger', wallets);

        return wallets.reduce((hwInfo, wallet) => {
          wallet.manufacturer = 'Ledger';
          wallet.name = 'Nano S';
          wallet.via = 'ledger';

          hwInfo[wallet.address] = wallet;

          return hwInfo;
        }, {});
      })
      .catch((error) => {
        console.warn('HardwareStore::scanLedger', error);

        return {};
      });
  }

  scanParity () {
    return this._api.parity
      .hardwareAccountsInfo()
      .then((hwInfo) => {
        Object
          .keys(hwInfo)
          .forEach((address) => {
            const info = hwInfo[address];

            info.address = address;
            info.via = 'parity';
          });

        return hwInfo;
      })
      .catch((error) => {
        console.warn('HardwareStore::scanParity', error);

        return {};
      });
  }

  scan () {
    this.setScanning(true);

    // NOTE: Depending on how the hardware is configured and how the local env setup
    // is done, different results will be retrieved via Parity vs. the browser APIs
    // (latter is Chrome-only, needs the browser app enabled on a Ledger, former is
    // not intended as a network call, i.e. hw wallet is with the user)
    return Promise
      .all([
        this.scanParity(),
        this.scanLedger()
      ])
      .then(([hwAccounts, ledgerAccounts]) => {
        transaction(() => {
          this.setWallets(Object.assign({}, hwAccounts, ledgerAccounts));
          this.setScanning(false);
        });
      });
  }

  createAccountInfo (entry, original = {}) {
    const { address, manufacturer, name } = entry;

    return Promise
      .all([
        original.name
          ? Promise.resolve(true)
          : this._api.parity.setAccountName(address, name),
        this._api.parity.setAccountMeta(address, Object.assign({
          description: `${manufacturer} ${name}`,
          hardware: {
            manufacturer
          },
          tags: ['hardware'],
          timestamp: Date.now()
        }, original.meta || {}))
      ])
      .catch((error) => {
        console.warn('HardwareStore::createEntry', error);
        throw error;
      });
  }

  signLedger (transaction) {
    return this._ledger.signTransaction(transaction);
  }

  static get (api) {
    if (!instance) {
      instance = new HardwareStore(api);
    }

    return instance;
  }
}

export {
  HW_SCAN_INTERVAL
};
