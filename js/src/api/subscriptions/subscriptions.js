import Eth from './eth';
import Logging from './logging';
import Personal from './personal';

const EVENTS = ['logging', 'eth_blockNumber'];
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
    const [prefix] = subscriptionName.split('.');
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
    this.values[subscriptionName] = { error, data };
    this.subscriptions[subscriptionName].forEach((callback) => {
      this._sendData(callback, error, data);
    });
  }
}
