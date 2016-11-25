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

import { keccak_256 } from 'js-sha3'; // eslint-disable-line camelcase

import { Logging } from '../../subscriptions';
import JsonRpcBase from '../jsonRpcBase';
import TransportError from '../error';

/* global WebSocket */
export default class Ws extends JsonRpcBase {
  constructor (url, token) {
    super();

    this._url = url;
    this._token = token;
    this._messages = {};

    this._connecting = false;
    this._connected = false;
    this._lastError = null;
    this._autoConnect = true;
    this._retries = 0;
    this._reconnectTimeoutId = null;

    this._connect();
  }

  updateToken (token) {
    this._token = token;
    this._autoConnect = true;

    this._connect();
  }

  _connect () {
    if (this._connecting) {
      return;
    }

    if (this._reconnectTimeoutId) {
      window.clearTimeout(this._reconnectTimeoutId);
      this._reconnectTimeoutId = null;
    }

    const time = parseInt(new Date().getTime() / 1000, 10);
    const sha3 = keccak_256(`${this._token}:${time}`);
    const hash = `${sha3}_${time}`;

    if (this._ws) {
      this._ws.onerror = null;
      this._ws.onopen = null;
      this._ws.onclose = null;
      this._ws.onmessage = null;
      this._ws.close();
      this._ws = null;
    }

    this._connecting = true;
    this._connected = false;
    this._lastError = null;

    this._ws = new WebSocket(this._url, hash);
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
  }

  _onOpen = (event) => {
    console.log('ws:onOpen', event);
    this._connected = true;
    this._connecting = false;
    this._autoConnect = true;
    this._retries = 0;

    Object.keys(this._messages)
      .filter((id) => this._messages[id].queued)
      .forEach(this._send);
  }

  _onClose = (event) => {
    this._connected = false;
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
        this._connect();
      }, timeout);

      return;
    }

    console.log('ws:onClose', event);
  }

  _onError = (event) => {
    // Only print error if the WS is connected
    // ie. don't print if error == closed
    window.setTimeout(() => {
      if (this._connected) {
        console.error('ws:onError', event);

        event.timestamp = Date.now();
        this._lastError = event;
      }
    }, 50);
  }

  _onMessage = (event) => {
    // Event sent by Signer Broadcaster
    if (event.data === 'new_message') {
      return false;
    }

    try {
      const result = JSON.parse(event.data);
      const { method, params, json, resolve, reject } = this._messages[result.id];

      Logging.send(method, params, { json, result });

      if (result.error) {
        this.error(event.data);

        console.error(`${method}(${JSON.stringify(params)}): ${result.error.code}: ${result.error.message}`);

        const error = new TransportError(method, result.error.code, result.error.message);
        reject(error);

        delete this._messages[result.id];
        return;
      }

      resolve(result.result);
      delete this._messages[result.id];
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

  execute (method, ...params) {
    return new Promise((resolve, reject) => {
      const id = this.id;
      const json = this.encode(method, params);

      this._messages[id] = { id, method, params, json, resolve, reject };
      this._send(id);
    });
  }

  get token () {
    return this._token;
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
