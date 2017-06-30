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

export default class Middleware {
  constructor (transport) {
    this._transport = transport;
    this._handlers = {};
  }

  register (method, handler) {
    if (method in this._handlers) {
      throw new Error(`${method} is already defined in the middleware!`);
    }

    this._handlers[method] = handler;
  }

  handle (method, params) {
    const handler = this._handlers[method];

    if (handler != null) {
      return handler(params);
    }

    return null;
  }

  rpcRequest (method, params) {
    return this._transport._execute(method, params);
  }
}
