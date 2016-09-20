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

import { Logging } from '../../subscriptions';
import JsonRpcBase from '../jsonRpcBase';

/* global WebSocket */
export default class Ws extends JsonRpcBase {
  constructor (url, protocols) {
    super();

    this._messages = {};

    this._ws = new WebSocket(url, protocols);
    this._ws.onerror = this._onError;
    this._ws.onopen = this._onOpen;
    this._ws.onclose = this._onClose;
    this._ws.onmessage = this._onMessage;
  }

  _onOpen = (event) => {
  }

  _onClose = (event) => {
  }

  _onError = (event) => {
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

    this.log(event.data);

    resolve(result.result);
    delete this._messages[result.id];
  }

  execute (method, ...params) {
    return new Promise((resolve, reject) => {
      const json = this.encode(method, params);
      this._messages[this.id] = { method, params, json, resolve: resolve, reject: reject };

      this.log(json);

      this._ws.send(json);
    });
  }
}
