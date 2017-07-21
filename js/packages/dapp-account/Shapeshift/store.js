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

import initShapeshift from '@parity/shapeshift/index';

const STAGE_OPTIONS = 0;
const STAGE_WAIT_DEPOSIT = 1;
const STAGE_WAIT_EXCHANGE = 2;
const STAGE_COMPLETED = 3;

const WARNING_NONE = 0;
const WARNING_NO_PRICE = -1;

export default class Store {
  @observable address = null;
  @observable coinPair = 'btc_eth';
  @observable coinSymbol = 'BTC';
  @observable coins = [];
  @observable depositAddress = '';
  @observable depositInfo = null;
  @observable exchangeInfo = null;
  @observable error = null;
  @observable hasAcceptedTerms = false;
  @observable price = null;
  @observable refundAddress = '';
  @observable stage = STAGE_OPTIONS;
  @observable warning = 0;

  constructor (address) {
    this._shapeshiftApi = initShapeshift();
    this.address = address;
  }

  @action setCoins = (coins) => {
    this.coins = coins;
  }

  @action setCoinSymbol = (coinSymbol) => {
    transaction(() => {
      this.coinSymbol = coinSymbol;
      this.coinPair = `${coinSymbol.toLowerCase()}_eth`;
      this.price = null;
    });

    return this.getCoinPrice();
  }

  @action setDepositAddress = (depositAddress) => {
    this.depositAddress = depositAddress;
  }

  @action setDepositInfo = (depositInfo) => {
    transaction(() => {
      this.depositInfo = depositInfo;
      this.setStage(STAGE_WAIT_EXCHANGE);
    });
  }

  @action setError = (error) => {
    this.error = error;
  }

  @action setExchangeInfo = (exchangeInfo) => {
    transaction(() => {
      this.exchangeInfo = exchangeInfo;
      this.setStage(STAGE_COMPLETED);
    });
  }

  @action setPrice = (price) => {
    transaction(() => {
      this.price = price;
      this.setWarning();
    });
  }

  @action setRefundAddress = (refundAddress) => {
    this.refundAddress = refundAddress;
  }

  @action setStage = (stage) => {
    this.stage = stage;
  }

  @action setWarning = (warning = WARNING_NONE) => {
    this.warning = warning;
  }

  @action toggleAcceptTerms = () => {
    this.hasAcceptedTerms = !this.hasAcceptedTerms;
  }

  getCoinPrice () {
    return this._shapeshiftApi
      .getMarketInfo(this.coinPair)
      .then((price) => {
        this.setPrice(price);
      })
      .catch((error) => {
        console.warn('getCoinPrice', error);

        this.setWarning(WARNING_NO_PRICE);
      });
  }

  retrieveCoins () {
    return this._shapeshiftApi
      .getCoins()
      .then((coins) => {
        this.setCoins(Object.values(coins).filter((coin) => coin.status === 'available'));

        return this.getCoinPrice();
      })
      .catch((error) => {
        console.error('retrieveCoins', error);
        const message = `Failed to retrieve available coins from ShapeShift.io: ${error.message}`;

        this.setError(message);
      });
  }

  shift () {
    this.setStage(STAGE_WAIT_DEPOSIT);

    return this._shapeshiftApi
      .shift(this.address, this.refundAddress, this.coinPair)
      .then((result) => {
        console.log('onShift', result);

        this.setDepositAddress(result.deposit);
        return this.subscribe();
      })
      .catch((error) => {
        console.error('onShift', error);
        const message = `Failed to start exchange: ${error.message}`;

        this.setError(new Error(message));
      });
  }

  onExchangeInfo = (error, result) => {
    if (error) {
      console.error('onExchangeInfo', error);

      if (error.fatal) {
        this.setError(error);
      }
      return;
    }

    console.log('onExchangeInfo', result.status, result);

    switch (result.status) {
      case 'received':
        if (this.stage !== STAGE_WAIT_EXCHANGE) {
          this.setDepositInfo(result);
        }
        return;

      case 'complete':
        if (this.stage !== STAGE_COMPLETED) {
          this.setExchangeInfo(result);
        }
        return;
    }
  }

  subscribe () {
    return this._shapeshiftApi.subscribe(this.depositAddress, this.onExchangeInfo);
  }

  unsubscribe () {
    return this._shapeshiftApi.unsubscribe(this.depositAddress);
  }
}

export {
  STAGE_COMPLETED,
  STAGE_OPTIONS,
  STAGE_WAIT_DEPOSIT,
  STAGE_WAIT_EXCHANGE,
  WARNING_NONE,
  WARNING_NO_PRICE
};
