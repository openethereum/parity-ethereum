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

import { isError } from '../util/types';

import Eth from './eth';
import Logging from './logging';
import Personal from './personal';
import Signer from './signer';

const events = {
  'logging': { module: 'logging' },
  'eth_blockNumber': { module: 'eth' },
  'parity_accountsInfo': { module: 'personal' },
  'parity_allAccountsInfo': { module: 'personal' },
  'parity_defaultAccount': { module: 'personal' },
  'eth_accounts': { module: 'personal' },
  'signer_requestsToConfirm': { module: 'signer' }
};

export default class Manager {
  constructor (api) {
    this._api = api;

    this.subscriptions = [];
    this.values = {};

    Object.keys(events).forEach((subscriptionName) => {
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

  subscribe (subscriptionName, callback, autoRemove = false) {
    return new Promise((resolve, reject) => {
      const subscription = this._validateType(subscriptionName);

      if (isError(subscription)) {
        reject(subscription);
        return;
      }

      const subscriptionId = this.subscriptions.length;
      const { error, data } = this.values[subscriptionName];
      const engine = this[`_${subscription.module}`];

      this.subscriptions[subscriptionId] = {
        name: subscriptionName,
        id: subscriptionId,
        autoRemove,
        callback
      };

      if (!engine.isStarted) {
        engine.start();
      } else {
        this._sendData(subscriptionId, error, data);
      }

      resolve(subscriptionId);
    });
  }

  unsubscribe (subscriptionId) {
    return new Promise((resolve, reject) => {
      if (!this.subscriptions[subscriptionId]) {
        reject(new Error(`Cannot find subscription ${subscriptionId}`));
        return;
      }

      delete this.subscriptions[subscriptionId];
      resolve();
    });
  }

  _sendData (subscriptionId, error, data) {
    const { autoRemove, callback } = this.subscriptions[subscriptionId];
    let result = true;

    try {
      result = callback(error, data);
    } catch (error) {
      console.error(`Unable to update callback for subscriptionId ${subscriptionId}`, error);
    }

    if (autoRemove && result && typeof result === 'boolean') {
      this.unsubscribe(subscriptionId);
    }
  }

  _updateSubscriptions = (subscriptionName, error, data) => {
    const subscriptions = this.subscriptions
      .filter(subscription => subscription.name === subscriptionName);

    this.values[subscriptionName] = { error, data };

    subscriptions
      .forEach((subscription) => {
        this._sendData(subscription.id, error, data);
      });
  }
}

export {
  events
};
