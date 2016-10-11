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

export default class Ws {
  constructor (path = window.location.host, reconnectDelay = 5000) {
    this._path = path;
    this._reconnectTimeout = reconnectDelay;
    this._isConnected = false;
    this._callbacks = {};
    this._queue = [];
    this._id = 1;
    this.onOpen = [];
    this.onMsg = [];
    this.onClose = [];
    this.onError = [];
  }

  init = token => {
    this.token = token; // store token for _onClose reconnect attemps
    clearTimeout(this._initTimeout);
    try {
      const hash = token ? this._hash(token) : null;
      this._ws = new global.WebSocket(`ws://${this._path}`, hash);
    } catch (err) {
      console.warn('[WS] error connecting to ws', err); // throws when port is blocked, not when hash is incorrect
    }

    this._ws.addEventListener('open', this._onOpen);
    this._ws.addEventListener('error', this._onError);
  }

  send (payload, callback) {
    if (!this._isConnected) {
      console.log('[WS] not connected. incoming msg added to queue');
      this._queue.push({ payload, callback });
      return;
    }
    const { _id } = this;
    payload.id = _id;
    payload = JSON.stringify(payload);
    this._callbacks[_id] = callback;
    this._ws.send(payload);
    this._id++;
  }

  _onOpen = () => {
    console.log('[WS] connected');
    this._ws.addEventListener('close', this._onClose);
    this._ws.addEventListener('message', this._onMsg);
    this._isConnected = true;
    this._executeQueue();
    this._triggerEvent(this.onOpen);
  }

  _onMsg = msg => {
    this._triggerEvent(this.onMsg, msg);
    try {
      msg = JSON.parse(msg.data);
    } catch (err) {
      return console.warn('[WS] unknown msg from server: ', msg, err);
    }
    const cb = this._callbacks[msg.id];
    delete this._callbacks[msg.id];

    if (!cb) {
      return;
    }

    cb(msg.error, msg.result);
  }

  _onClose = () => {
    console.warn('[WS] closed');
    this._executeCbsWithError();
    this._isConnected = false;
    this._triggerEvent(this.onClose);
    this.init(this.token);
  }

  _onError = err => {
    console.warn('[WS] error', err);
    this._triggerEvent(this.onError, err);
    this._initTimeout = this._initWithTimeout();
  }

  _triggerEvent (evts, arg) {
    evts.forEach(fn => fn(arg));
  }

  _initWithTimeout () {
    return setTimeout(() => this.init(this.token), this._reconnectTimeout);
  }

  _executeQueue () {
    console.log('[WS] executing queue: ', this._queue);
    this._queue.forEach(call => {
      this.send(call.payload, call.callback);
    });
    this._queue = [];
  }

  _executeCbsWithError () {
    console.log('[WS] executing callbacks with error: ', this._callbacks);
    for (const msgId in this._callbacks) {
      const cb = this._callbacks[msgId];
      cb('[WS] disconnected, cb cannot be called');
    }
    this._callbacks = {};
  }

  _hash (token) {
    const time = parseInt(new Date().getTime() / 1000, 10);
    return keccak_256(token + ':' + time) + '_' + time;
  }

  _noop () {}

}
