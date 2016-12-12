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

const CHECK_INTERVAL = 1 * 60 * 1000;

export default class UpgradeStore {
  @observable available = null;
  @observable consensusCapability = null;

  constructor (api) {
    this._api = api;

    this.checkUpgrade();
    setInterval(this.checkUpgrade, CHECK_INTERVAL);
  }

  @action setAvailableUpgrade (available, consensusCapability) {
    transaction(() => {
      this.available = available;
      this.consensusCapability = consensusCapability;
    });
  }

  checkUpgrade = () => {
    return Promise
      .all([
        this._api.parity.upgradeReady(),
        this._api.parity.consensusCapability()
      ])
      .then(([available, consensusCapability]) => {
        console.log('[checkUpgrade]', 'available:', available, 'consensusCapability:', consensusCapability);
        this.setAvailableUpgrade(available, consensusCapability);
      })
      .catch((error) => {
        console.warn('checkUpgrade', error);
      });
  }

  executeUpgrade = () => {
    return this._api.parity.executeUpgrade();
  }
}
