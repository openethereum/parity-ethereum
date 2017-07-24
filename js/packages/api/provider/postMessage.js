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

  constructor (token, destination) {
    this._token = token;
    this._destination = destination;

    window.addEventListener('message', this.receiveMessage, false);
  }

  addMiddleware () {
  }

  _send (data) {
    this._destination.postMessage(data, '*');
  }

  send = (method, params, callback) => {
    const id = ++this.id;

    this._messages[id] = { callback };
    this._send({
      id,
      from: this._token,
      method,
      params,
      token: this._token
    });
  }

  subscribe = (api, callback, params) => {
    console.log('paritySubscribe', JSON.stringify(params), api, callback);
    return new Promise((resolve, reject) => {
      const id = ++this.id;

      this._messages[id] = { callback, resolve, reject, subscription: true, initial: true };
      this._send({
        id,
        from: this._token,
        api,
        params,
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
        from: this._token,
        subId,
        token: this._token
      });
    });
  }

  unsubscribeAll () {
    return this.unsubscribe('*');
  }

  receiveMessage = ({ data: { id, error, from, token, result }, origin, source }) => {
    if (from !== 'shell' || token !== this._token) {
      return;
    }

    if (error) {
      console.error(from, error);
    }

    if (this._messages[id].subscription) {
      console.log('subscription', result, 'initial?', this._messages[id].initial);
      this._messages[id].initial ? this._messages[id].resolve(result) : this._messages[id].callback(error && new Error(error), result);
      this._messages[id].initial = false;
    } else {
      this._messages[id].callback(error && new Error(error), result);
      this._messages[id] = null;
    }
  }
}
