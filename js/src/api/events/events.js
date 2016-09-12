import BigNumber from 'bignumber.js';

const EVENTS = ['eth.blockNumber'];

export default class Events {
  constructor (api) {
    this._api = api;

    this._lastBlock = new BigNumber(0);
    this.subscriptions = {};
    this.values = {};

    EVENTS.forEach((eventName) => {
      this.subscriptions[eventName] = [];
      this.values[eventName] = 0;
    });

    this._pollBlockNumber();
  }

  _validateEvent (eventName) {
    if (!EVENTS.includes(eventName)) {
      throw new Error(`${eventName} is not a valid eventName, subscribe using one of ${EVENTS.join(', ')}`);
    }
  }

  subscribe (eventName, callback) {
    this._validateEvent(eventName);

    const subscriptionId = this.subscriptions[eventName].length;

    this.subscriptions[eventName].push(callback);
    this._sendData(callback, this.values[eventName]);

    return subscriptionId;
  }

  unsubscribe (eventName, subscriptionId) {
    this._validateEvent(eventName);

    this.subscriptions[eventName].filter((callback, idx) => idx !== subscriptionId);
  }

  _sendData (callback, data) {
    try {
      callback(data);
    } catch (error) {
    }
  }

  _sendEvents (eventName, data) {
    this.values[eventName] = data;
    this.subscriptions[eventName].forEach((callback) => {
      this._sendData(callback, data);
    });
  }

  _pollBlockNumber = () => {
    const nextTimeout = () => setTimeout(this._pollBlockNumber, 1000);

    // if (!this.subscriptions['eth.blockNumber'].length) {
    //   nextTimeout();
    //   return;
    // }

    this._api.eth
      .blockNumber()
      .then((blockNumber) => {
        if (blockNumber.gt(this._lastBlock)) {
          this._lastBlock = blockNumber;
          this._sendEvents('eth.blockNumber', blockNumber);
        }

        nextTimeout();
      })
      .catch(nextTimeout);
  }
}
