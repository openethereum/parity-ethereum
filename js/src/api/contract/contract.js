import Abi from '../../abi';
import Api from '../api';
import { isInstanceOf } from '../util/types';

export default class Contract {
  constructor (api, abi) {
    if (!isInstanceOf(api, Api)) {
      throw new Error('Api instance needs to be provided to Contract');
    } else if (!abi) {
      throw new Error('ABI needs to be provided to Contract instance');
    }

    this._api = api;
    this._abi = new Abi(abi);

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

  _bindFunction (func) {
    const addAddress = (_options = {}) => {
      const options = {};

      Object.keys(_options).forEach((key) => {
        options[key] = _options[key];
      });
      options.to = options.to || this._address;

      return options;
    };

    func.call = (options, values) => {
      return this._api.eth
        .call(this._encodeOptions(func, addAddress(options), values))
        .then((encoded) => func.decodeOutput(encoded))
        .then((tokens) => tokens.map((token) => token.value))
        .then((returns) => returns.length === 1 ? returns[0] : returns);
    };

    if (!func.constant) {
      func.postTransaction = (options, values) => {
        return this._api.eth
          .postTransaction(this._encodeOptions(func, addAddress(options), values));
      };

      func.estimateGas = (options, values) => {
        return this._api.eth
          .estimateGas(this._encodeOptions(func, addAddress(options), values));
      };
    }

    return func;
  }

  _bindEvent (event) {
    const subscriptions = [];

    event.subscribe = (options, callback) => {
      const subscriptionId = subscriptions.length;

      options.address = this._address;
      options.topics = [event.signature];

      this._api.eth
        .newFilter(options)
        .then((filterId) => {
          return this._api.eth
            .getFilterLogs(filterId)
            .then((logs) => {
              callback(this.parseEventLogs(logs));

              const subscription = {
                options,
                callback,
                filterId
              };

              subscriptions.push(subscription);
            });
        });

      return subscriptionId;
    };

    event.unsubscribe = (subscriptionId) => {
      subscriptions.filter((callback, idx) => idx !== subscriptionId);
    };

    const sendChanges = (subscription) => {
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

    // this._api.events.subscribe('eth.blockNumber', onTriggerSend);
    setInterval(onTriggerSend, 1000);

    return event;
  }
}
