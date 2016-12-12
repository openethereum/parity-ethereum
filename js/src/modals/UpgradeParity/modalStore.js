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

import { action, computed, observable, transaction } from 'mobx';
import store from 'store';

const LS_UPDATE = '_parity::update';

const A_DAY = 24 * 60 * 60 * 1000;

const STEP_INFO = 1;
const STEP_UPDATING = 2;
const STEP_COMPLETED = 3;
const STEP_ERROR = 4;

export default class ModalStore {
  @observable closed = false;
  @observable error = null;
  @observable step = 0;
  @observable upgrade = null;

  constructor (upgradeStore) {
    this.upgrade = upgradeStore;

    this.loadStorage();
  }

  @computed get showUpgrade () {
    return !closed && Date.now() >= this.remindAt;
  }

  @action closeModal = () => {
    transaction(() => {
      this.closed = true;
      this.setStep(STEP_INFO);
    });
  }

  @action loadStorage = () => {
    const values = store.get(LS_UPDATE) || {};

    this.remindAt = values.remindAt ? values.remindAt : 0;

    return values;
  }

  @action setStep = (step, error = null) => {
    transaction(() => {
      this.error = error;
      this.setp = step;
    });
  }

  @action snoozeTillTomorrow = () => {
    this.remindAt = Date.now() + A_DAY;
    store.set(LS_UPDATE, Object.assign(this.loadStorage(), { remindAt: this.remindAt }));
  }

  @action upgradeNow = () => {
    this.setStep(STEP_UPDATING);

    this.upgrade
      .executeUpgrade()
      .then((result) => {
        if (!result) {
          throw new Error('Unable to complete update');
        }

        this.setStep(STEP_COMPLETED);
      })
      .catch((error) => {
        console.error('upgradeNow', error);

        this.setStep(STEP_ERROR, error);
      });
  }
}

export {
  STEP_COMPLETED,
  STEP_ERROR,
  STEP_INFO,
  STEP_UPDATING
};
