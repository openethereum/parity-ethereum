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

import { isError } from '../util/types';

import Eth from './eth';
import Logging from './logging';
import Personal from './personal';
import Signer from './signer';

const events = {
  'logging': { module: 'logging' },
  'eth_blockNumber': { module: 'eth' },
  'personal_accountsInfo': { module: 'personal' },
  'personal_listAccounts': { module: 'personal' },
  'personal_requestsToConfirm': { module: 'signer' }
};

let nextSubscriptionId = 0;

export default class Manager {
  constructor (api) {
    this._api = api;

    this.subscriptions = {};
    this.values = {};

    Object.keys(events).forEach((subscriptionName) => {
      this.subscriptions[subscriptionName] = {};
      this.values[subscriptionName] = {
        error: null,
        data: null
      };
    });

    this._logging = new Logging(this._updateSubscriptions);
    this._eth = new Eth(this._updateSubscriptions, api);
    this._personal = new Personal(this._updateSubscriptions, api, this);
    this._signer = new Signer(this._updateSubscriptions, api, this);
  }

  _validateType (subscriptionName) {
    const subscription = events[subscriptionName];

    if (!subscription) {
      return new Error(`${subscriptionName} is not a valid interface, subscribe using one of ${Object.keys(events).join(', ')}`);
    }

    return subscription;
  }

  subscribe (subscriptionName, callback) {
    return new Promise((resolve, reject) => {
      const subscription = this._validateType(subscriptionName);

      if (isError(subscription)) {
        reject(subscription);
        return;
      }

      const subscriptionId = nextSubscriptionId++;
      const { error, data } = this.values[subscriptionName];
      const engine = this[`_${subscription.module}`];

      this.subscriptions[subscriptionName][subscriptionId] = callback;

      if (!engine.isStarted) {
        engine.start();
      } else {
        this._sendData(subscriptionName, subscriptionId, callback, error, data);
      }

      resolve(subscriptionId);
    });
  }

  unsubscribe (subscriptionName, subscriptionId) {
    return new Promise((resolve, reject) => {
      const subscription = this._validateType(subscriptionName);

      if (isError(subscription)) {
        reject(subscription);
        return;
      }

      if (!this.subscriptions[subscriptionName][subscriptionId]) {
        reject(new Error(`Cannot find subscription ${subscriptionId} for type ${subscriptionName}`));
        return;
      }

      delete this.subscriptions[subscriptionName][subscriptionId];
      resolve();
    });
  }

  _sendData (subscriptionName, subscriptionId, callback, error, data) {
    try {
      callback(error, data);
    } catch (error) {
      console.error(`Unable to update callback for ${subscriptionName}, subscriptionId ${subscriptionId}`, error);
      this.unsubscribe(subscriptionName, subscriptionId);
    }
  }

  _updateSubscriptions = (subscriptionName, error, data) => {
    if (!this.subscriptions[subscriptionName]) {
      throw new Error(`Cannot find entry point for subscriptions of type ${subscriptionName}`);
    }

    this.values[subscriptionName] = { error, data };
    Object.keys(this.subscriptions[subscriptionName]).forEach((subscriptionId) => {
      const callback = this.subscriptions[subscriptionName][subscriptionId];

      this._sendData(subscriptionName, subscriptionId, callback, error, data);
    });
  }
}

export {
  events
};
