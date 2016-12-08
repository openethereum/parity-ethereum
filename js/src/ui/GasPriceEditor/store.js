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
import { action, computed, observable, transaction } from 'mobx';

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

  constructor (api, gasLimit) {
    this._api = api;
    this.gasLimit = gasLimit;

    this.loadDefaults();
  }

  @action setEthValue = (ethValue) => {
    this.ethValue = ethValue;
  }

  @action setEstimated = (estimated) => {
    transaction(() => {
      this.estimated = estimated.toFixed(0);

      if (estimated.gte(MAX_GAS_ESTIMATION)) {
        this.errorEstimated = ERRORS.gasException;
      } else if (estimated.gte(this.gasLimit)) {
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

      this.gas = gas;

      if (numberError) {
        this.errorGas = numberError;
      } else if (new BigNumber(gas).gte(this.lgasLimit)) {
        this.errorGas = ERRORS.gasBlockLimit;
      } else {
        this.errorGas = null;
      }
    });
  }

  @computed get ethTotal () {
    try {
      return this.ethValue.add(new BigNumber(this.price).mul(this.gas));
    } catch (e) {
      return this.ethValue;
    }
  }

  @action loadDefaults () {
    Promise
      .all([
        this.api.parity.gasPriceHistogram(),
        this.api.eth.gasPrice()
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
