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

import { keccak_256 } from 'js-sha3'; // eslint-disable-line camelcase

import { Logging } from '../../subscriptions';
import JsonRpcBase from '../jsonRpcBase';
import TransportError from '../error';

/* global WebSocket */
export default class Ws extends JsonRpcBase {
  // token is optional (secure API)
  constructor (url, token = null, autoconnect = true) {
    super();

    this._url = url;
    this._token = token;
    this._messages = {};
    this._subscriptions = { 'eth_subscription': [], 'parity_subscription': [], 'shh_subscription': [] };
    this._sessionHash = null;

    this._connecting = false;
    this._connected = false;
    this._lastError = null;
    this._autoConnect = autoconnect;
    this._retries = 0;
    this._reconnectTimeoutId = null;

    this._connectPromise = null;
    this._connectPromiseFunctions = {};

    if (autoconnect) {
      this.connect();
    }
  }

  updateToken (token, connect = true) {
    this._token = token;
    // this._autoConnect = true;

    if (connect) {
      this.connect();
    }
  }

  connect () {
    if (this._connected) {
      return Promise.resolve();
    }

    if (this._connecting) {
      return this._connectPromise || Promise.resolve();
    }

    if (this._reconnectTimeoutId) {
      window.clearTimeout(this._reconnectTimeoutId);
      this._reconnectTimeoutId = null;
    }

    if (this._ws) {
      this._ws.onerror = null;
      this._ws.onopen = null;
      this._ws.onclose = null;
      this._ws.onmessage = null;
      this._ws.close();
      this._ws = null;
      this._sessionHash = null;
    }
    this._connecting = true;
    this._connected = false;
    this._lastError = null;

    // rpc secure API
    if (this._token) {
      const time = parseInt(new Date().getTime() / 1000, 10);
      const sha3 = keccak_256(`${this._token}:${time}`);
      const hash = `${sha3}_${time}`;

      this._sessionHash = sha3;
      this._ws = new WebSocket(this._url, hash);
    // non-secure API
    } else {
      this._ws = new WebSocket(this._url);
    }

    this._ws.onerror = this._onError;
    this._ws.onopen = this._onOpen;
    this._ws.onclose = this._onClose;
    this._ws.onmessage = this._onMessage;

    // Get counts in dev mode only
    if (process.env.NODE_ENV === 'development') {
      this._count = 0;
      this._lastCount = {
        timestamp: Date.now(),
        count: 0
      };

      window.setInterval(() => {
        const n = this._count - this._lastCount.count;
        const t = (Date.now() - this._lastCount.timestamp) / 1000;
        const s = Math.round(1000 * n / t) / 1000;

        if (this._debug) {
          console.log('::parityWS', `speed: ${s} req/s`, `count: ${this._count}`, `(+${n})`);
        }

        this._lastCount = {
          timestamp: Date.now(),
          count: this._count
        };
      }, 5000);

      window._parityWS = this;
    }

    this._connectPromise = new Promise((resolve, reject) => {
      this._connectPromiseFunctions = { resolve, reject };
    });

    return this._connectPromise;
  }

  _onOpen = (event) => {
    this._setConnected();
    this._connecting = false;
    this._retries = 0;

    Object.keys(this._messages)
      .filter((id) => this._messages[id].queued)
      .forEach(this._send);

    this._connectPromiseFunctions.resolve();

    this._connectPromise = null;
    this._connectPromiseFunctions = {};
  }

  _onClose = (event) => {
    this._setDisconnected();
    this._connecting = false;

    event.timestamp = Date.now();
    this._lastError = event;

    if (this._autoConnect) {
      const timeout = this.retryTimeout;

      const time = timeout < 1000
        ? Math.round(timeout) + 'ms'
        : (Math.round(timeout / 10) / 100) + 's';

      console.log('ws:onClose', `trying again in ${time}...`);

      this._reconnectTimeoutId = setTimeout(() => {
        this.connect();
      }, timeout);

      return;
    }

    if (this._connectPromise) {
      this._connectPromiseFunctions.reject(event);

      this._connectPromise = null;
      this._connectPromiseFunctions = {};
    }

    console.log('ws:onClose');
  }

  _onError = (event) => {
    // Only print error if the WS is connected
    // ie. don't print if error == closed
    window.setTimeout(() => {
      if (this._connected) {
        console.error('ws:onError');

        event.timestamp = Date.now();
        this._lastError = event;

        if (this._connectPromise) {
          this._connectPromiseFunctions.reject(event);

          this._connectPromise = null;
          this._connectPromiseFunctions = {};
        }
      }
    }, 50);
  }

  _extract = (result) => {
    const { result: res, id, method, params } = result;
    const msg = this._messages[id];

    // initial pubsub ACK
    if (id && msg.subscription) {
      // save subscription to map subId -> messageId
      this._subscriptions[msg.subscription][res] = id;
      // resolve promise with messageId because subId's can collide (eth/parity)
      msg.resolve(id);
      // save subId for unsubscribing later
      msg.subId = res;
      return msg;
    }

    // normal message
    if (id) {
      return msg;
    }

    // pubsub format
    if (method.includes('subscription')) {
      const messageId = this._messages[this._subscriptions[method][params.subscription]];

      if (messageId) {
        return messageId;
      } else {
        throw Error(`Received Subscription which is already unsubscribed ${JSON.stringify(result)}`);
      }
    }

    throw Error(`Unknown message format: No ID or subscription ${JSON.stringify(result)}`);
  }

  _onMessage = (event) => {
    try {
      const result = JSON.parse(event.data);
      const { method, params, json, resolve, reject, callback, subscription } = this._extract(result);

      Logging.send(method, params, { json, result });

      result.error = (result.params && result.params.error) || result.error;
      if (result.error) {
        this.error(event.data);

        // Don't print error if request rejected or not is not yet up...
        if (!/(rejected|not yet up)/.test(result.error.message)) {
          console.error(`${method}(${JSON.stringify(params)}): ${result.error.code}: ${result.error.message}`);
        }

        const error = new TransportError(method, result.error.code, result.error.message);

        if (result.id) {
          reject(error);
        } else {
          callback(error);
        }

        delete this._messages[result.id];
        return;
      }

      // if not initial subscription message resolve & delete
      if (result.id && !subscription) {
        resolve(result.result);
        delete this._messages[result.id];
      } else if (result.params) {
        callback(null, result.params.result);
      }
    } catch (e) {
      console.error('ws::_onMessage', event.data, e);
    }
  }

  _send = (id) => {
    const message = this._messages[id];

    if (this._connected) {
      if (process.env.NODE_ENV === 'development') {
        this._count++;
      }

      return this._ws.send(message.json);
    }

    message.queued = !this._connected;
    message.timestamp = Date.now();
  }

  _execute (method, params) {
    return new Promise((resolve, reject) => {
      const id = this.id;
      const json = this.encode(method, params);

      this._messages[id] = { id, method, params, json, resolve, reject };
      this._send(id);
    });
  }

  _methodsFromApi (api) {
    const method = `${api}_subscribe`;
    const uMethod = `${api}_unsubscribe`;
    const subscription = `${api}_subscription`;

    return { method, uMethod, subscription };
  }

  subscribe (api, callback, ...params) {
    return new Promise((resolve, reject) => {
      const id = this.id;
      const { method, uMethod, subscription } = this._methodsFromApi(api);
      const json = this.encode(method, params);

      this._messages[id] = { id, method, uMethod, params, json, resolve, reject, callback, subscription };

      this._send(id);
    });
  }

  unsubscribe (messageId) {
    return new Promise((resolve, reject) => {
      const id = this.id;
      const { subId, uMethod, subscription } = this._messages[messageId];
      const params = [subId];
      const json = this.encode(uMethod, params);
      const uResolve = (v) => {
        delete this._messages[messageId];
        delete this._subscriptions[subscription][subId];
        resolve(v);
      };

      this._messages[id] = { id, method: uMethod, params, json, resolve: uResolve, reject };
      this._send(id);
    });
  }

  set url (url) {
    this._url = url;
  }

  get token () {
    return this._token;
  }

  get sessionHash () {
    return this._sessionHash;
  }

  get isAutoConnect () {
    return this._autoConnect;
  }

  get isConnecting () {
    return this._connecting;
  }

  get lastError () {
    return this._lastError;
  }

  /**
   * Exponential Timeout for Retries
   *
   * @see http://dthain.blogspot.de/2009/02/exponential-backoff-in-distributed.html
   */
  get retryTimeout () {
    // R between 1 and 2
    const R = Math.random() + 1;
    // Initial timeout (100ms)
    const T = 100;
    // Exponential Factor
    const F = 2;
    // Max timeout (4s)
    const M = 4000;
    // Current number of retries
    const N = this._retries;

    // Increase retries number
    this._retries++;

    return Math.min(R * T * Math.pow(F, N), M);
  }
}
