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

/* global WebSocket */
export default class Ws extends JsonRpcBase {
  constructor (url, token) {
    super();

    this._url = url;
    this._token = token;
    this._messages = {};
    this._connected = false;

    this._init();
  }

  _hash () {
    const time = parseInt(new Date().getTime() / 1000, 10);
    const sha3 = keccak_256(`${this._token}:${time}`);

    return `${sha3}_${time}`;
  }

  _init () {
    this._ws = new WebSocket(this._url, this._hash());

    this._ws.onerror = this._onError;
    this._ws.onopen = this._onOpen;
    this._ws.onclose = this._onClose;
    this._ws.onmessage = this._onMessage;
  }

  _onOpen = (event) => {
    console.log('ws:onOpen', event);
    this._connected = true;

    Object.keys(this._messages)
      .filter((id) => this._messages[id].queued)
      .forEach(this._send);
  }

  _onClose = (event) => {
    console.log('ws:onClose', event);
    this._connected = false;
    this._init();
  }

  _onError = (event) => {
    console.error('ws:onError', event);
  }

  _onMessage = (event) => {
    const result = JSON.parse(event.data);
    const { method, params, json, resolve, reject } = this._messages[result.id];

    Logging.send(method, params, { json, result });

    if (result.error) {
      this.error(event.data);

      reject(new Error(`${result.error.code}: ${result.error.message}`));
      delete this._messages[result.id];
      return;
    }

    resolve(result.result);
    delete this._messages[result.id];
  }

  _send = (id) => {
    const message = this._messages[id];

    message.queued = !this._connected;

    if (this._connected) {
      this._ws.send(message.json);
    }
  }

  execute (method, ...params) {
    return new Promise((resolve, reject) => {
      const id = this.id;
      const json = this.encode(method, params);

      this._messages[id] = { id, method, params, json, resolve, reject };
      this._send(id);
    });
  }
}
