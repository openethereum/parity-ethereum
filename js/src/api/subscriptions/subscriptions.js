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

import Eth from './eth';
import Logging from './logging';
import Personal from './personal';

const EVENTS = [
  'logging',
  'eth_blockNumber',
  'personal_accountsInfo',
  'personal_listAccounts'
];
const ALIASSES = {};

export default class Subscriptions {
  constructor (api) {
    this._api = api;

    this.subscriptions = {};
    this.values = {};

    EVENTS.forEach((subscriptionName) => {
      this.subscriptions[subscriptionName] = [];
      this.values[subscriptionName] = {
        error: null,
        data: null
      };
    });

    this._logging = new Logging(this._updateSubscriptions);
    this._eth = new Eth(this._updateSubscriptions, api);
    this._personal = new Personal(this._updateSubscriptions, api, this);
  }

  _validateType (_subscriptionName) {
    const subscriptionName = ALIASSES[_subscriptionName] || _subscriptionName;

    if (!EVENTS.includes(subscriptionName)) {
      throw new Error(`${subscriptionName} is not a valid interface, subscribe using one of ${EVENTS.join(', ')}`);
    }

    return subscriptionName;
  }

  subscribe (_subscriptionName, callback) {
    const subscriptionName = this._validateType(_subscriptionName);
    const subscriptionId = this.subscriptions[subscriptionName].length;
    const { error, data } = this.values[subscriptionName];
    const [prefix] = subscriptionName.split('_');
    const engine = this[`_${prefix}`];

    this.subscriptions[subscriptionName].push(callback);

    if (!engine.isStarted) {
      engine.start();
    } else {
      this._sendData(callback, error, data);
    }

    return subscriptionId;
  }

  unsubscribe (_subscriptionName, subscriptionId) {
    const subscriptionName = this._validateType(_subscriptionName);

    if (subscriptionId >= this.subscriptions[subscriptionName].length) {
      throw new Error(`Cannot find subscriptions at index ${subscriptionId} for type ${subscriptionName}`);
    }

    this.subscriptions[subscriptionName][subscriptionId] = null;

    return true;
  }

  _sendData (callback, error, data) {
    if (!callback) {
      return;
    }

    try {
      callback(error, data);
    } catch (error) {
    }
  }

  _updateSubscriptions = (subscriptionName, error, data) => {
    if (!this.subscriptions[subscriptionName]) {
      throw new Error(`Cannot find entry point for subscriptions of type ${subscriptionName}`);
    }

    this.values[subscriptionName] = { error, data };
    this.subscriptions[subscriptionName].forEach((callback) => {
      this._sendData(callback, error, data);
    });
  }
}
