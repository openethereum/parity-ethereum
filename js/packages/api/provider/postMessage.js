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

export default class PostMessage {
  id = 0;
  _messages = {};

  constructor (appId, token, destination) {
    this._appId = appId;
    this._token = token;
    this._destination = destination || window.parent;

    window.addEventListener('message', this.receiveMessage, false);
  }

  addMiddleware () {
  }

  requestNewToken () {
    return new Promise((resolve, reject) => {
      this.send('shell_requestNewToken', [this._appId], (error, token) => {
        if (error) {
          reject(error);
        } else {
          this._token = token;
          resolve(token);
        }
      });
    });
  }

  _send (data) {
    this._destination.postMessage(data, '*');
  }

  send = (method, params, callback) => {
    const id = ++this.id;

    this._messages[id] = { callback };
    this._send({
      id,
      from: this._appId,
      method,
      params,
      to: 'shell',
      token: this._token
    });
  }

  subscribe = (api, callback, params) => {
    // console.log('paritySubscribe', JSON.stringify(params), api, callback);
    return new Promise((resolve, reject) => {
      const id = ++this.id;

      this._messages[id] = { callback, resolve, reject, subscription: true, initial: true };
      this._send({
        id,
        from: this._appId,
        api,
        params,
        to: 'shell',
        token: this._token
      });
    });
  }

  unsubscribe = (subId) => {
    return new Promise((resolve, reject) => {
      const id = ++this.id;

      this._messages[id] = { callback: (e, v) => e ? reject(e) : resolve(v) };
      this._send({
        id,
        from: this._appId,
        subId,
        to: 'shell',
        token: this._token
      });
    });
  }

  unsubscribeAll () {
    return this.unsubscribe('*');
  }

  receiveMessage = ({ data: { id, error, from, to, token, result }, origin, source }) => {
    const isTokenValid = token
      ? token === this._token
      : to === this._appId;

    if (from !== 'shell' || to !== this._appId || !isTokenValid) {
      return;
    }

    if (error) {
      console.error(from, error);
    }

    if (this._messages[id].subscription) {
      // console.log('subscription', result, 'initial?', this._messages[id].initial);
      this._messages[id].initial
        ? this._messages[id].resolve(result)
        : this._messages[id].callback(error && new Error(error), result);
      this._messages[id].initial = false;
    } else {
      this._messages[id].callback(error && new Error(error), result);
      this._messages[id] = null;
    }
  }
}
