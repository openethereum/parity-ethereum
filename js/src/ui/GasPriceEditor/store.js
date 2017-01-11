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

import BigNumber from 'bignumber.js';
import { action, computed, observable, transaction } from 'mobx';

import { ERRORS, validatePositiveNumber } from '~/util/validation';
import { DEFAULT_GAS, DEFAULT_GASPRICE, MAX_GAS_ESTIMATION } from '~/util/constants';

export default class GasPriceEditor {
  @observable errorEstimated = null;
  @observable errorGas = null;
  @observable errorPrice = null;
  @observable errorTotal = null;
  @observable estimated = DEFAULT_GAS;
  @observable gas;
  @observable gasLimit;
  @observable histogram = null;
  @observable isEditing = false;
  @observable price;
  @observable priceDefault;
  @observable weiValue = '0';

  constructor (api, { gas, gasLimit, gasPrice }) {
    this._api = api;

    this.gas = gas;
    this.gasLimit = gasLimit;
    this.price = gasPrice;

    if (api) {
      this.loadDefaults();
    }
  }

  @computed get totalValue () {
    try {
      return new BigNumber(this.gas).mul(this.price).add(this.weiValue);
    } catch (error) {
      return new BigNumber(0);
    }
  }

  @action setEditing = (isEditing) => {
    this.isEditing = isEditing;
  }

  @action setErrorTotal = (errorTotal) => {
    this.errorTotal = errorTotal;
  }

  @action setEstimatedError = (errorEstimated = ERRORS.gasException) => {
    this.errorEstimated = errorEstimated;
  }

  @action setEstimated = (estimated) => {
    transaction(() => {
      const bn = new BigNumber(estimated);

      this.estimated = estimated;

      if (bn.gte(MAX_GAS_ESTIMATION)) {
        this.setEstimatedError(ERRORS.gasException);
      } else if (bn.gte(this.gasLimit)) {
        this.setEstimatedError(ERRORS.gasBlockLimit);
      } else {
        this.setEstimatedError(null);
      }
    });
  }

  @action setEthValue = (weiValue) => {
    this.weiValue = weiValue;
  }

  @action setGas = (gas) => {
    transaction(() => {
      const { numberError } = validatePositiveNumber(gas);

      this.gas = gas;

      if (numberError) {
        this.errorGas = numberError;
      } else {
        const bn = new BigNumber(gas);

        if (bn.gte(this.gasLimit)) {
          this.errorGas = ERRORS.gasBlockLimit;
        } else {
          this.errorGas = null;
        }
      }
    });
  }

  @action setGasLimit = (gasLimit) => {
    this.gasLimit = gasLimit;
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

  @action loadDefaults () {
    Promise
      .all([
        this._api.parity.gasPriceHistogram(),
        this._api.eth.gasPrice()
      ])
      .then(([histogram, _price]) => {
        transaction(() => {
          const price = _price.toFixed(0);

          if (!this.price) {
            this.setPrice(price);
          }
          this.setHistogram(histogram);

          this.priceDefault = price;
        });
      })
      .catch((error) => {
        console.warn('getDefaults', error);
      });
  }

  overrideTransaction = (transaction) => {
    if (this.errorGas || this.errorPrice) {
      return transaction;
    }

    return Object.assign({}, transaction, {
      gas: new BigNumber(this.gas || DEFAULT_GAS),
      gasPrice: new BigNumber(this.price || DEFAULT_GASPRICE)
    });
  }
}
