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

import Abi from '../../abi';
import Api from '../api';
import { isInstanceOf } from '../util/types';

export default class Contract {
  constructor (api, abi) {
    if (!isInstanceOf(api, Api)) {
      throw new Error('API instance needs to be provided to Contract');
    } else if (!abi) {
      throw new Error('ABI needs to be provided to Contract instance');
    }

    this._api = api;
    this._abi = new Abi(abi);

    this._subscriptions = [];
    this._constructors = this._abi.constructors.map((cons) => this._bindFunction(cons));
    this._functions = this._abi.functions.map((func) => this._bindFunction(func));
    this._events = this._abi.events.map((event) => this._bindEvent(event));

    this._instance = {};

    this._events.forEach((evt) => {
      this._instance[evt.name] = evt;
    });
    this._functions.forEach((fn) => {
      this._instance[fn.name] = fn;
    });

    this._sendSubscriptionChanges();
  }

  get address () {
    return this._address;
  }

  get constructors () {
    return this._constructors;
  }

  get events () {
    return this._events;
  }

  get functions () {
    return this._functions;
  }

  get instance () {
    return this._instance;
  }

  get api () {
    return this._api;
  }

  get abi () {
    return this._abi;
  }

  at (address) {
    this._address = address;
    return this;
  }

  deploy (code, values) {
    const options = {
      data: code,
      gas: 900000
    };

    return this._api.eth
      .postTransaction(this._encodeOptions(this.constructors[0], options, values))
      .then((txhash) => this.pollTransactionReceipt(txhash))
      .then((receipt) => {
        this._address = receipt.contractAddress;
        return this._api.eth.getCode(this._address);
      })
      .then((code) => {
        if (code === '0x') {
          throw new Error('Contract not deployed, getCode returned 0x');
        }

        return this.address;
      });
  }

  parseEventLogs (logs) {
    return logs.map((log) => {
      const signature = log.topics[0].substr(2);
      const event = this.events.find((evt) => evt.signature === signature);

      if (!event) {
        throw new Error(`Unable to find event matching signature ${signature}`);
      }

      const decoded = event.decodeLog(log.topics, log.data);

      log.params = {};
      log.address = decoded.address;
      log.event = event.name;

      decoded.params.forEach((param) => {
        log.params[param.name] = param.token.value;
      });

      return log;
    });
  }

  parseTransactionEvents (receipt) {
    receipt.logs = this.parseEventLogs(receipt.logs);

    return receipt;
  }

  pollTransactionReceipt (txhash) {
    return new Promise((resolve, reject) => {
      const timeout = () => {
        this._api.eth
          .getTransactionReceipt(txhash)
          .then((receipt) => {
            if (receipt) {
              resolve(receipt);
            } else {
              setTimeout(timeout, 500);
            }
          })
          .catch((error) => {
            console.error('pollTransactionReceipt', error);
            reject(error);
          });
      };

      timeout();
    });
  }

  _encodeOptions (func, options, values) {
    const tokens = this._abi.encodeTokens(func.inputParamTypes(), values);

    if (options.data && options.data.substr(0, 2) === '0x') {
      options.data = options.data.substr(2);
    }
    options.data = `0x${options.data || ''}${func.encodeCall(tokens)}`;

    return options;
  }

  _addOptionsTo (options = {}) {
    return Object.assign({
      to: this._address
    }, options);
  }

  _bindFunction (func) {
    func.call = (options, values) => {
      return this._api.eth
        .call(this._encodeOptions(func, this._addOptionsTo(options), values))
        .then((encoded) => func.decodeOutput(encoded))
        .then((tokens) => tokens.map((token) => token.value))
        .then((returns) => returns.length === 1 ? returns[0] : returns);
    };

    if (!func.constant) {
      func.postTransaction = (options, values) => {
        return this._api.eth
          .postTransaction(this._encodeOptions(func, this._addOptionsTo(options), values));
      };

      func.estimateGas = (options, values) => {
        return this._api.eth
          .estimateGas(this._encodeOptions(func, this._addOptionsTo(options), values));
      };
    }

    return func;
  }

  _bindEvent (event) {
    const subscriptions = [];

    event.subscribe = (_options, callback) => {
      const subscriptionId = subscriptions.length;
      const options = Object.assign({}, _options, {
        address: this._address,
        topics: [event.signature]
      });

      this._api.eth
        .newFilter(options)
        .then((filterId) => {
          return this._api.eth
            .getFilterLogs(filterId)
            .then((logs) => {
              callback(this.parseEventLogs(logs));

              subscriptions.push({
                options,
                callback,
                filterId
              });
            });
        });

      return subscriptionId;
    };

    event.unsubscribe = (subscriptionId) => {
      const subscription = subscriptions[subscriptionId];

      this._api.eth
        .uninstallFilter(subscription.filterId);

      subscriptions[subscriptionId] = null;
    };

    const sendChanges = (subscription) => {
      if (!subscription) {
        return;
      }

      this._api.eth
        .getFilterChanges(subscription.filterId)
        .then((logs) => {
          try {
            subscription.callback(this.parseEventLogs(logs));
          } catch (error) {
            console.error('pollChanges', error);
          }
        });
    };

    const onTriggerSend = (blockNumber) => {
      subscriptions.forEach(sendChanges);
    };

    setInterval(onTriggerSend, 1000);

    return event;
  }

  subscribe (eventName, _options = {}, callback) {
    const subscriptionId = this._subscriptions.length;
    let event = null;

    if (eventName) {
      event = this._events.find((evt) => evt.name === eventName);

      if (!event) {
        const events = this._events.map((evt) => evt.name).join(', ');
        throw new Error(`${eventName} is not a valid eventName, subscribe using one of ${events} (or null to include all)`);
      }
    }

    const options = Object.assign({}, _options, {
      address: this._address,
      topics: [event ? event.signature : null]
    });

    this._api.eth
      .newFilter(options)
      .then((filterId) => {
        return this._api.eth
          .getFilterLogs(filterId)
          .then((logs) => {
            callback(null, this.parseEventLogs(logs));

            this._subscriptions.push({
              options,
              callback,
              filterId
            });
          });
      })
      .catch((error) => {
        console.log('subscribe', error);
        callback(error);
      });

    return subscriptionId;
  }

  unsubscribe (subscriptionId) {
    const subscription = this._subscriptions[subscriptionId];

    this._api.eth.uninstallFilter(subscription.filterId);
    this._subscriptions[subscriptionId] = null;
  }

  _sendSubscriptionChanges = () => {
    const subscriptions = this._subscriptions.filter((subscription) => subscription);
    const timeout = () => setTimeout(this._sendSubscriptionChanges, 1000);

    Promise
      .all(
        subscriptions.map((subscription) => {
          return this._api.eth.getFilterChanges(subscription.filterId);
        })
      )
      .then((logsArray) => {
        logsArray.forEach((logs, idx) => {
          try {
            subscriptions[idx].callback(null, this.parseEventLogs(logs));
          } catch (error) {
            subscriptions[idx].callback(error);
            console.error('_sendSubscriptionChanges', error);
          }
        });

        timeout();
      })
      .catch((error) => {
        console.error('_sendSubscriptionChanges', error);
        timeout();
      });
  }
}
