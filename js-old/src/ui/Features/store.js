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
import store from 'store';

import { LS_STORE_KEY } from './constants';
import defaults, { FEATURES, MODES } from './defaults';

const isProductionMode = process.env.NODE_ENV === 'production';

let instance = null;

export default class Store {
  @observable active = {};

  constructor () {
    this.loadActiveFeatures();
  }

  @action setActiveFeatures = (features = {}, isProduction) => {
    this.active = Object.assign({}, this.getDefaultActive(isProduction), features);
  }

  @action toggleActive = (featureKey) => {
    this.active = Object.assign({}, this.active, { [featureKey]: !this.active[featureKey] });
    this.saveActiveFeatures();
  }

  getDefaultActive (isProduction = isProductionMode) {
    const modesTest = [MODES.PRODUCTION];

    if (!isProduction) {
      modesTest.push(MODES.TESTING);
    }

    return Object
      .keys(FEATURES)
      .reduce((visibility, feature) => {
        visibility[feature] = modesTest.includes(defaults[feature].mode);
        return visibility;
      }, {});
  }

  loadActiveFeatures () {
    this.setActiveFeatures(store.get(LS_STORE_KEY));
  }

  saveActiveFeatures () {
    store.set(LS_STORE_KEY, this.active);
  }

  static get () {
    if (!instance) {
      instance = new Store();
    }

    return instance;
  }
}
