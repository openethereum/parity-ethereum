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
  _callbacks = {};

  constructor (token, destination) {
    this._token = token;
    this._destination = destination;

    window.addEventListener('message', this.receiveMessage, false);
  }

  addMiddleware () {
  }

  send = (method, params, callback) => {
    const id = ++this.id;

    this._callbacks[id] = callback;
    this._destination.postMessage({
      id,
      from: this._token,
      method,
      params,
      token: this._token
    }, '*');
  }

  receiveMessage = ({ data: { id, error, from, token, result }, origin, source }) => {
    if (from !== 'shell' || token !== this._token) {
      return;
    }

    this._callbacks[id](error, result);
    this._callbacks[id] = null;
  }
}
