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

import { action, computed, observable, transaction, toJS } from 'mobx';
import store from 'store';

const LS_UPDATE = '_parity::update';

const A_MINUTE = 60 * 1000;
const A_DAY = 24 * 60 * A_MINUTE;

const STEP_INFO = 0;
const STEP_UPDATING = 1;
const STEP_COMPLETED = 2;
const STEP_ERROR = 2;

let instance = null;

export default class Store {
  @observable available = null;
  @observable consensusCapability = null;
  @observable closed = true;
  @observable error = null;
  @observable remindAt = 0;
  @observable step = 0;
  @observable upgrading = null;
  @observable version = null;

  constructor (api) {
    this._api = api;

    this.loadStorage();
    this.checkUpgrade();
  }

  @computed get isVisible () {
    return !this.closed && Date.now() >= this.remindAt;
  }

  @action closeModal = () => {
    transaction(() => {
      this.closed = true;
      this.setStep(0, null);
    });
  }

  @action loadStorage = () => {
    const values = store.get(LS_UPDATE) || {};

    this.remindAt = values.remindAt ? values.remindAt : 0;

    return values;
  }

  @action openModal = () => {
    this.closed = false;
  }

  @action setStep = (step, error = null) => {
    transaction(() => {
      this.error = error;
      this.step = step;
    });
  }

  @action setUpgrading () {
    transaction(() => {
      this.upgrading = this.available;
      this.setStep(STEP_UPDATING, null);
    });
  }

  @action setVersions (available, version, consensusCapability) {
    transaction(() => {
      this.available = available;
      this.consensusCapability = consensusCapability;
      this.version = version;
    });
  }

  @action snoozeTillTomorrow = () => {
    this.remindAt = Date.now() + A_DAY;
    store.set(LS_UPDATE, Object.assign(this.loadStorage(), { remindAt: this.remindAt }));
  }

  @action upgradeNow = () => {
    this.setUpgrading();

    return this._api.parity
      .executeUpgrade()
      .then((result) => {
        if (!result) {
          throw new Error('Unable to complete update');
        }

        this.setStep(STEP_COMPLETED, null);
      })
      .catch((error) => {
        console.error('upgradeNow', error);

        this.setStep(STEP_ERROR, error);
      });
  }

  checkUpgrade = () => {
    if (!this._api) {
      return Promise.resolve(false);
    }

    return Promise
      .all([
        this._api.parity.upgradeReady(),
        this._api.parity.consensusCapability(),
        this._api.parity.versionInfo()
      ])
      .then(([available, consensusCapability, version]) => {
        if (!this.version || version.hash !== this.version.hash) {
          console.log('[checkUpgrade]', 'available:', available, 'version:', toJS(version.version), 'consensusCapability:', consensusCapability);
        }

        this.setVersions(available, version, consensusCapability);

        return true;
      })
      .catch((error) => {
        console.warn('checkUpgrade', error);

        return false;
      });
  }

  static get (api) {
    if (!instance) {
      instance = new Store(api);
    }

    return instance;
  }
}

export {
  STEP_COMPLETED,
  STEP_ERROR,
  STEP_INFO,
  STEP_UPDATING
};
