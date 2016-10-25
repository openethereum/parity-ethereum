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

let nextSubscriptionId = 0;

export default class Contract {
  constructor (api, abi) {
    if (!isInstanceOf(api, Api)) {
      throw new Error('API instance needs to be provided to Contract');
    } else if (!abi) {
      throw new Error('ABI needs to be provided to Contract instance');
    }

    this._api = api;
    this._abi = new Abi(abi);

    this._subscriptions = {};
    this._constructors = this._abi.constructors.map(this._bindFunction);
    this._functions = this._abi.functions.map(this._bindFunction);
    this._events = this._abi.events.map(this._bindEvent);

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
    this._instance.address = this._address;
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

  deploy (options, values, statecb) {
    let gas;

    const setState = (state) => {
      if (!statecb) {
        return;
      }

      return statecb(null, state);
    };

    setState({ state: 'estimateGas' });

    return this._api.eth
      .estimateGas(this._encodeOptions(this.constructors[0], options, values))
      .then((_gas) => {
        gas = _gas.mul(1.2);
        options.gas = gas.toFixed(0);

        setState({ state: 'postTransaction', gas });
        return this._api.eth.postTransaction(this._encodeOptions(this.constructors[0], options, values));
      })
      .then((requestId) => {
        setState({ state: 'checkRequest', requestId });
        return this._pollCheckRequest(requestId);
      })
      .then((txhash) => {
        setState({ state: 'getTransactionReceipt', txhash });
        return this._pollTransactionReceipt(txhash, gas);
      })
      .then((receipt) => {
        if (receipt.gasUsed.eq(gas)) {
          throw new Error(`Contract not deployed, gasUsed == ${gas.toFixed(0)}`);
        }

        setState({ state: 'hasReceipt', receipt });
        this._address = receipt.contractAddress;
        return this._address;
      })
      .then((address) => {
        setState({ state: 'getCode' });
        return this._api.eth.getCode(this._address);
      })
      .then((code) => {
        if (code === '0x') {
          throw new Error('Contract not deployed, getCode returned 0x');
        }

        setState({ state: 'completed' });
        return this._address;
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
      log.event = event.name;

      decoded.params.forEach((param) => {
        const { type, value } = param.token;

        log.params[param.name] = { type, value };
      });

      return log;
    });
  }

  parseTransactionEvents (receipt) {
    receipt.logs = this.parseEventLogs(receipt.logs);

    return receipt;
  }

  _pollCheckRequest = (requestId) => {
    return this._api.pollMethod('eth_checkRequest', requestId);
  }

  _pollTransactionReceipt = (txhash, gas) => {
    return this.api.pollMethod('eth_getTransactionReceipt', txhash, (receipt) => {
      if (!receipt || !receipt.blockNumber || receipt.blockNumber.eq(0)) {
        return false;
      }

      return true;
    });
  }

  _encodeOptions (func, options, values) {
    const tokens = func ? this._abi.encodeTokens(func.inputParamTypes(), values) : null;
    const call = tokens ? func.encodeCall(tokens) : null;

    if (options.data && options.data.substr(0, 2) === '0x') {
      options.data = options.data.substr(2);
    }
    options.data = `0x${options.data || ''}${call || ''}`;

    return options;
  }

  _addOptionsTo (options = {}) {
    return Object.assign({
      to: this._address
    }, options);
  }

  _bindFunction = (func) => {
    func.call = (options, values = []) => {
      return this._api.eth
        .call(this._encodeOptions(func, this._addOptionsTo(options), values))
        .then((encoded) => func.decodeOutput(encoded))
        .then((tokens) => tokens.map((token) => token.value))
        .then((returns) => returns.length === 1 ? returns[0] : returns);
    };

    if (!func.constant) {
      func.postTransaction = (options, values = []) => {
        return this._api.eth
          .postTransaction(this._encodeOptions(func, this._addOptionsTo(options), values));
      };

      func.estimateGas = (options, values = []) => {
        return this._api.eth
          .estimateGas(this._encodeOptions(func, this._addOptionsTo(options), values));
      };
    }

    return func;
  }

  _bindEvent = (event) => {
    event.subscribe = (options = {}, callback) => {
      return this._subscribe(event, options, callback);
    };

    event.unsubscribe = (subscriptionId) => {
      return this.unsubscribe(subscriptionId);
    };

    return event;
  }

  subscribe (eventName = null, options = {}, callback) {
    return new Promise((resolve, reject) => {
      let event = null;

      if (eventName) {
        event = this._events.find((evt) => evt.name === eventName);

        if (!event) {
          const events = this._events.map((evt) => evt.name).join(', ');
          reject(new Error(`${eventName} is not a valid eventName, subscribe using one of ${events} (or null to include all)`));
          return;
        }
      }

      return this._subscribe(event, options, callback).then(resolve).catch(reject);
    });
  }

  _subscribe (event = null, _options, callback) {
    const subscriptionId = nextSubscriptionId++;
    const options = Object.assign({}, _options, {
      address: this._address,
      topics: [event ? event.signature : null]
    });

    return this._api.eth
      .newFilter(options)
      .then((filterId) => {
        return this._api.eth
          .getFilterLogs(filterId)
          .then((logs) => {
            callback(null, this.parseEventLogs(logs));
            this._subscriptions[subscriptionId] = {
              options,
              callback,
              filterId
            };

            return subscriptionId;
          });
      });
  }

  unsubscribe (subscriptionId) {
    return this._api.eth
      .uninstallFilter(this._subscriptions[subscriptionId].filterId)
      .then(() => {
        delete this._subscriptions[subscriptionId];
      })
      .catch((error) => {
        console.error('unsubscribe', error);
      });
  }

  _sendSubscriptionChanges = () => {
    const subscriptions = Object.values(this._subscriptions);
    const timeout = () => setTimeout(this._sendSubscriptionChanges, 1000);

    Promise
      .all(
        subscriptions.map((subscription) => {
          return this._api.eth.getFilterChanges(subscription.filterId);
        })
      )
      .then((logsArray) => {
        logsArray.forEach((logs, idx) => {
          if (!logs || !logs.length) {
            return;
          }

          try {
            subscriptions[idx].callback(null, this.parseEventLogs(logs));
          } catch (error) {
            this.unsubscribe(idx);
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
