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

import { action, observable, transaction } from 'mobx';

import Ledger from '~/3rdparty/ledger';

const HW_SCAN_INTERVAL = 5000;
let instance = null;

export default class HardwareStore {
  @observable isScanning = false;
  @observable wallets = [];

  constructor (api) {
    this._api = api;
    this._ledger = Ledger.create();
    this._pollId = null;

    this._pollScan();
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
    return this._ledger
      .scan()
      .then((wallet) => {
        console.log('HardwareStore::scanLedger', wallet);

        return [];
        // return [
        //   wallet
        // ];
      })
      .catch((error) => {
        console.warn('HardwareStore::scanLedger', error);

        return [];
      });
  }

  scanParity () {
    return this._api.parity
      .hardwareAccountsInfo()
      .then((hwInfo) => {
        console.log('HardwareStore::scanParity', hwInfo);

        return [];
        // return Object
        //   .keys(hwInfo)
        //   .map((address) => {
        //     hwInfo[address] = address;
        //
        //     return hwInfo[address];
        //   });
      })
      .catch((error) => {
        console.warn('HardwareStore::scanParity', error);

        return [];
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
        this.scanLedger(),
        this.scanParity()
      ])
      .then(([ledgerAccounts, hwAccounts]) => {
        transaction(() => {
          this.setWallets(
            []
              .concat(ledgerAccounts)
              .concat(hwAccounts)
          );
          this.setScanning(false);
        });
      });
  }

  createAccountInfo (entry) {
    const { address, manufacturer, name } = entry;

    return Promise
      .all([
        this._api.parity.setAccountName(address, name),
        this._api.parity.setAccountMeta(address, {
          description: `${manufacturer} ${name}`,
          hardware: {
            manufacturer
          },
          tags: ['hardware'],
          timestamp: Date.now()
        })
      ])
      .catch((error) => {
        console.warn('HardwareStore::createEntry', error);
        throw error;
      });
  }

  signLedger (rawTransaction) {
    return this._ledger.signTransaction(rawTransaction);
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
