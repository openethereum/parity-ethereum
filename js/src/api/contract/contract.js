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

import Abi from '~/abi';

let nextSubscriptionId = 0;

export default class Contract {
  constructor (api, abi) {
    if (!api) {
      throw new Error('API instance needs to be provided to Contract');
    }

    if (!abi) {
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
      this._instance[evt.signature] = evt;
    });

    this._functions.forEach((fn) => {
      this._instance[fn.name] = fn;
      this._instance[fn.signature] = fn;
    });

    this._subscribedToPendings = false;
    this._pendingsSubscriptionId = null;

    this._subscribedToBlock = false;
    this._blockSubscriptionId = null;

    if (api && api.patch && api.patch.contract) {
      api.patch.contract(this);
    }
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

  get receipt () {
    return this._receipt;
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

  deployEstimateGas (options, values) {
    const _options = this._encodeOptions(this.constructors[0], options, values);

    return this._api.eth
      .estimateGas(_options)
      .then((gasEst) => {
        return [gasEst, gasEst.mul(1.2)];
      });
  }

  deploy (options, values, statecb = () => {}) {
    statecb(null, { state: 'estimateGas' });

    return this
      .deployEstimateGas(options, values)
      .then(([gasEst, gas]) => {
        options.gas = gas.toFixed(0);

        statecb(null, { state: 'postTransaction', gas });

        const encodedOptions = this._encodeOptions(this.constructors[0], options, values);

        return this._api.parity
          .postTransaction(encodedOptions)
          .then((requestId) => {
            statecb(null, { state: 'checkRequest', requestId });
            return this._pollCheckRequest(requestId);
          })
          .then((txhash) => {
            statecb(null, { state: 'getTransactionReceipt', txhash });
            return this._pollTransactionReceipt(txhash, gas);
          })
          .then((receipt) => {
            if (receipt.gasUsed.eq(gas)) {
              throw new Error(`Contract not deployed, gasUsed == ${gas.toFixed(0)}`);
            }

            statecb(null, { state: 'hasReceipt', receipt });
            this._receipt = receipt;
            this._address = receipt.contractAddress;
            return this._address;
          })
          .then((address) => {
            statecb(null, { state: 'getCode' });
            return this._api.eth.getCode(this._address);
          })
          .then((code) => {
            if (code === '0x') {
              throw new Error('Contract not deployed, getCode returned 0x');
            }

            statecb(null, { state: 'completed' });
            return this._address;
          });
      });
  }

  parseEventLogs (logs) {
    return logs
      .map((log) => {
        const signature = log.topics[0].substr(2);
        const event = this.events.find((evt) => evt.signature === signature);

        if (!event) {
          console.warn(`Unable to find event matching signature ${signature}`);
          return null;
        }

        const decoded = event.decodeLog(log.topics, log.data);

        log.params = {};
        log.event = event.name;

        decoded.params.forEach((param) => {
          const { type, value } = param.token;

          log.params[param.name] = { type, value };
        });

        return log;
      })
      .filter((log) => log);
  }

  parseTransactionEvents (receipt) {
    receipt.logs = this.parseEventLogs(receipt.logs);

    return receipt;
  }

  _pollCheckRequest = (requestId) => {
    return this._api.pollMethod('parity_checkRequest', requestId);
  }

  _pollTransactionReceipt = (txhash, gas) => {
    return this.api.pollMethod('eth_getTransactionReceipt', txhash, (receipt) => {
      if (!receipt || !receipt.blockNumber || receipt.blockNumber.eq(0)) {
        return false;
      }

      return true;
    });
  }

  getCallData = (func, options, values) => {
    let data = options.data;

    const tokens = func ? Abi.encodeTokens(func.inputParamTypes(), values) : null;
    const call = tokens ? func.encodeCall(tokens) : null;

    if (data && data.substr(0, 2) === '0x') {
      data = data.substr(2);
    }

    return `0x${data || ''}${call || ''}`;
  }

  _encodeOptions (func, options, values) {
    const data = this.getCallData(func, options, values);

    return {
      ...options,
      data
    };
  }

  _addOptionsTo (options = {}) {
    return {
      to: this._address,
      ...options
    };
  }

  _bindFunction = (func) => {
    func.contract = this;

    func.call = (_options = {}, values = []) => {
      const rawTokens = !!_options.rawTokens;
      const options = {
        ..._options
      };

      delete options.rawTokens;

      let callParams;

      try {
        callParams = this._encodeOptions(func, this._addOptionsTo(options), values);
      } catch (error) {
        return Promise.reject(error);
      }

      return this._api.eth
        .call(callParams)
        .then((encoded) => func.decodeOutput(encoded))
        .then((tokens) => {
          if (rawTokens) {
            return tokens;
          }

          return tokens.map((token) => token.value);
        })
        .then((returns) => returns.length === 1 ? returns[0] : returns)
        .catch((error) => {
          console.warn(`${func.name}.call`, values, error);
          throw error;
        });
    };

    if (!func.constant) {
      func.postTransaction = (options, values = []) => {
        let _options;

        try {
          _options = this._encodeOptions(func, this._addOptionsTo(options), values);
        } catch (error) {
          return Promise.reject(error);
        }

        return this._api.parity
          .postTransaction(_options)
          .catch((error) => {
            console.warn(`${func.name}.postTransaction`, values, error);
            throw error;
          });
      };

      func.estimateGas = (options, values = []) => {
        const _options = this._encodeOptions(func, this._addOptionsTo(options), values);

        return this._api.eth
          .estimateGas(_options)
          .catch((error) => {
            console.warn(`${func.name}.estimateGas`, values, error);
            throw error;
          });
      };
    }

    return func;
  }

  _bindEvent = (event) => {
    event.subscribe = (options = {}, callback, autoRemove) => {
      return this._subscribe(event, options, callback, autoRemove);
    };

    event.unsubscribe = (subscriptionId) => {
      return this.unsubscribe(subscriptionId);
    };

    event.getAllLogs = (options = {}) => {
      return this.getAllLogs(event);
    };

    return event;
  }

  getAllLogs (event, _options) {
    // Options as first parameter
    if (!_options && event && event.topics) {
      return this.getAllLogs(null, event);
    }

    const options = this._getFilterOptions(event, _options);

    options.fromBlock = 0;
    options.toBlock = 'latest';

    return this._api.eth
      .getLogs(options)
      .then((logs) => this.parseEventLogs(logs));
  }

  _findEvent (eventName = null) {
    const event = eventName
      ? this._events.find((evt) => evt.name === eventName)
      : null;

    if (eventName && !event) {
      const events = this._events.map((evt) => evt.name).join(', ');

      throw new Error(`${eventName} is not a valid eventName, subscribe using one of ${events} (or null to include all)`);
    }

    return event;
  }

  _getFilterOptions (event = null, _options = {}) {
    const optionTopics = _options.topics || [];
    const signature = event && event.signature || null;

    // If event provided, remove the potential event signature
    // as the first element of the topics
    const topics = signature
      ? [ signature ].concat(optionTopics.filter((t, idx) => idx > 0 || t !== signature))
      : optionTopics;

    const options = Object.assign({}, _options, {
      address: this._address,
      topics
    });

    return options;
  }

  _createEthFilter (event = null, _options) {
    const options = this._getFilterOptions(event, _options);

    return this._api.eth.newFilter(options);
  }

  subscribe (eventName = null, options = {}, callback, autoRemove) {
    try {
      const event = this._findEvent(eventName);

      return this._subscribe(event, options, callback, autoRemove);
    } catch (e) {
      return Promise.reject(e);
    }
  }

  _sendData (subscriptionId, error, logs) {
    const { autoRemove, callback } = this._subscriptions[subscriptionId];
    let result = true;

    try {
      result = callback(error, logs);
    } catch (error) {
      console.warn('_sendData', subscriptionId, error);
    }

    if (autoRemove && result && typeof result === 'boolean') {
      this.unsubscribe(subscriptionId);
    }
  }

  _subscribe (event = null, _options, callback, autoRemove = false) {
    const subscriptionId = nextSubscriptionId++;
    const { skipInitFetch } = _options;

    delete _options['skipInitFetch'];

    return this
      ._createEthFilter(event, _options)
      .then((filterId) => {
        this._subscriptions[subscriptionId] = {
          options: _options,
          autoRemove,
          callback,
          filterId,
          id: subscriptionId
        };

        if (skipInitFetch) {
          this._subscribeToChanges();
          return subscriptionId;
        }

        return this._api.eth
          .getFilterLogs(filterId)
          .then((logs) => {
            this._sendData(subscriptionId, null, this.parseEventLogs(logs));
            this._subscribeToChanges();
            return subscriptionId;
          });
      })
      .catch((error) => {
        console.warn('subscribe', event, _options, error);
        throw error;
      });
  }

  unsubscribe (subscriptionId) {
    return this._api.eth
      .uninstallFilter(this._subscriptions[subscriptionId].filterId)
      .catch((error) => {
        console.error('unsubscribe', error);
      })
      .then(() => {
        delete this._subscriptions[subscriptionId];
        this._unsubscribeFromChanges();
      });
  }

  _subscribeToChanges = () => {
    const subscriptions = Object.values(this._subscriptions);

    const pendingSubscriptions = subscriptions
      .filter((s) => s.options.toBlock && s.options.toBlock === 'pending');

    const otherSubscriptions = subscriptions
      .filter((s) => !(s.options.toBlock && s.options.toBlock === 'pending'));

    if (pendingSubscriptions.length > 0 && !this._subscribedToPendings) {
      this._subscribedToPendings = true;
      this._subscribeToPendings();
    }

    if (otherSubscriptions.length > 0 && !this._subscribedToBlock) {
      this._subscribedToBlock = true;
      this._subscribeToBlock();
    }
  }

  _unsubscribeFromChanges = () => {
    const subscriptions = Object.values(this._subscriptions);

    const pendingSubscriptions = subscriptions
      .filter((s) => s.options.toBlock && s.options.toBlock === 'pending');

    const otherSubscriptions = subscriptions
      .filter((s) => !(s.options.toBlock && s.options.toBlock === 'pending'));

    if (pendingSubscriptions.length === 0 && this._subscribedToPendings) {
      this._subscribedToPendings = false;
      clearTimeout(this._pendingsSubscriptionId);
    }

    if (otherSubscriptions.length === 0 && this._subscribedToBlock) {
      this._subscribedToBlock = false;
      this._api.unsubscribe(this._blockSubscriptionId);
    }
  }

  _subscribeToBlock = () => {
    this._api
      .subscribe('eth_blockNumber', (error) => {
        if (error) {
          console.error('::_subscribeToBlock', error, error && error.stack);
        }

        const subscriptions = Object.values(this._subscriptions)
          .filter((s) => !(s.options.toBlock && s.options.toBlock === 'pending'));

        this._sendSubscriptionChanges(subscriptions);
      })
      .then((blockSubId) => {
        this._blockSubscriptionId = blockSubId;
      })
      .catch((e) => {
        console.error('::_subscribeToBlock', e, e && e.stack);
      });
  }

  _subscribeToPendings = () => {
    const subscriptions = Object.values(this._subscriptions)
      .filter((s) => s.options.toBlock && s.options.toBlock === 'pending');

    const timeout = () => setTimeout(() => this._subscribeToPendings(), 1000);

    this._sendSubscriptionChanges(subscriptions)
      .then(() => {
        this._pendingsSubscriptionId = timeout();
      });
  }

  _sendSubscriptionChanges = (subscriptions) => {
    return Promise
      .all(
        subscriptions.map((subscription) => {
          return this._api.eth.getFilterChanges(subscription.filterId);
        })
      )
      .then((logsArray) => {
        logsArray.forEach((logs, index) => {
          if (!logs || !logs.length) {
            return;
          }

          try {
            this._sendData(subscriptions[index].id, null, this.parseEventLogs(logs));
          } catch (error) {
            console.error('_sendSubscriptionChanges', error);
          }
        });
      })
      .catch((error) => {
        console.error('_sendSubscriptionChanges', error);
      });
  }
}
