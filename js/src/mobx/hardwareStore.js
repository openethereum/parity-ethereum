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

import { action, observable } from 'mobx';

import Ledger from '~/3rdparty/ledger';

const HW_SCAN_INTERVAL = 5000;
let instance = null;

export default class HardwareStore {
  @observable isScanning = false;
  @observable wallet = null;

  constructor (api) {
    this._api = api;
    this._ledger = Ledger.create();
    this._pollId = null;

    this.pollScan();
  }

  @action setScanning = (isScanning) => {
    this.isScanning = isScanning;
  }

  @action setWallet = (wallet) => {
    this.wallet = wallet;
  }

  scanLedger () {
    return this._ledger
      .scan()
      .then((wallet) => {
        console.log('HardwareStore::scanLedger', wallet);

        this.setWallet(wallet);
      })
      .catch((error) => {
        console.warn('HardwareStore::scanLedger', error);
      });
  }

  scan () {
    this.setScanning(true);

    return this
      .scanLedger()
      .then(() => {
        this.setScanning(false);
      });
  }

  createEntry (address, name, description, type) {
    return Promise
      .all([
        this._api.setAccountName(address, name),
        this._api.setAccountMeta(address, {
          deleted: false,
          description,
          hardware: { type },
          name,
          tags: ['hardware'],
          timestamp: Date.now(),
          wallet: true
        })
      ])
      .catch((error) => {
        console.warn('HardwareStore::createEntry', error);
        throw error;
      });
  }

  pollScan = () => {
    this._pollId = setTimeout(() => {
      this.scan().then(this.pollScan);
    }, HW_SCAN_INTERVAL);
  }

  static get () {
    if (!instance) {
      instance = new HardwareStore();
    }

    return instance;
  }
}

export {
  HW_SCAN_INTERVAL
};
