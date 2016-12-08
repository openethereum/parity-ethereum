// Copyright 2015, 2016 Ethcore (UK) Ltd.
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

import BigNumber from 'bignumber.js';
import { action, observable, transaction } from 'mobx';

import { ERRORS, validatePositiveNumber } from '~/util/validation';
import { DEFAULT_GAS, DEFAULT_GASPRICE, MAX_GAS_ESTIMATION } from '~/util/constants';

export default class GasPriceEditor {
  @observable errorEstimated = null;
  @observable errorGas = null;
  @observable errorPrice = null;
  @observable estimated = DEFAULT_GAS;
  @observable histogram = null;
  @observable price = DEFAULT_GASPRICE;
  @observable priceDefault = DEFAULT_GASPRICE;
  @observable gas = DEFAULT_GAS;
  @observable gasLimit = 0;

  constructor (api, gasLimit, loadDefaults = true) {
    this._api = api;
    this.gasLimit = gasLimit;

    if (loadDefaults) {
      this.loadDefaults();
    }
  }

  @action setEstimated = (estimated) => {
    transaction(() => {
      const bn = new BigNumber(estimated);

      this.estimated = estimated;

      if (bn.gte(MAX_GAS_ESTIMATION)) {
        this.errorEstimated = ERRORS.gasException;
      } else if (bn.gte(this.gasLimit)) {
        this.errorEstimated = ERRORS.gasBlockLimit;
      } else {
        this.errorEstimated = null;
      }
    });
  }

  @action setHistogram = (gasHistogram) => {
    this.histogram = gasHistogram;
  }

  @action setPrice = (price) => {
    transaction(() => {
      this.errorPrice = validatePositiveNumber(price).numberError;
      this.price = price;
    });
  }

  @action setGas = (gas) => {
    transaction(() => {
      const { numberError } = validatePositiveNumber(gas);
      const bn = new BigNumber(gas);

      this.gas = gas;

      if (numberError) {
        this.errorGas = numberError;
      } else if (bn.gte(this.gasLimit)) {
        this.errorGas = ERRORS.gasBlockLimit;
      } else {
        this.errorGas = null;
      }
    });
  }

  @action loadDefaults () {
    Promise
      .all([
        this._api.parity.gasPriceHistogram(),
        this._api.eth.gasPrice()
      ])
      .then(([gasPriceHistogram, gasPrice]) => {
        transaction(() => {
          this.setPrice(gasPrice.toFixed(0));
          this.setHistogram(gasPriceHistogram);

          this.priceDefault = gasPrice.toFixed();
        });
      })
      .catch((error) => {
        console.warn('getDefaults', error);
      });
  }
}
