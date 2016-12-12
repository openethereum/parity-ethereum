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
import store from 'store';

const AN_HOUR = 60 * 60 * 1000;
const A_DAY = 24 * AN_HOUR;
const CHECK_INTERVAL = AN_HOUR;
const LS_UPDATE = '_parity::update';

export default class Store {
  @observable availableUpgrade = null;
  @observable remindAt = 0;
  @observable showUpgrade = false;

  constructor (api) {
    this._api = api;

    this.checkUpgrade();
    setInterval(this.pollUpgrade, CHECK_INTERVAL);
  }

  @action loadStorage () {
    const values = store.get(LS_UPDATE) || {};

    this.remindAt = values.remindAt ? values.remindAt : 0;

    return values;
  }

  @action setAvailableUpgrade (availableUpgrade, consensusCapability) {
    transaction(() => {
      this.setConsensusCapability(consensusCapability);
      this.availableUpgrade = availableUpgrade;

      if (availableUpgrade && Date.now() >= this.remindAt) {
        this.showUpgrade = true;
      }
    });
  }

  @action setConsensusCapability (consensusCapability) {
    this.consensusCapability = consensusCapability;
  }

  @action snoozeTillTomorrow () {
    store.set(LS_UPDATE, Object.assign(this.loadStorage(), {
      remindAt: Date.now() + A_DAY
    }));
  }

  checkUpgrade = () => {
    this.loadStorage();

    return Promise
      .all([
        this._api.parity.upgradeReady(),
        this._api.parity.consensusCapability()
      ])
      .then(([availableUpgrade, consensusCapability]) => {
        this.setAvailableUpgrade(availableUpgrade, consensusCapability);
      })
      .catch((error) => {
        console.warn('checkUpgrade', error);
      });
  }

  executeUpgrade = () => {
    return this._api.parity
      .executeUpgrade();
  }
}
