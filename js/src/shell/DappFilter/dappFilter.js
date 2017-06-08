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

export default class DappFilter {
  constructor (provider, permissions) {
    this.permissions = permissions;
    this.provider = provider;

    window.addEventListener('message', this.receiveMessage, false);
  }

  receiveMessage = ({ data: { id, from, method, params, token }, origin, source }) => {
    if (from === 'shell' || from !== token) {
      return;
    }

    if (this.permissions.filtered.includes(method) && !this.permissions.tokens[token][method]) {
      source.postMessage({
        id,
        from: 'shell',
        error: new Error(`Method ${method} is not available to application`),
        result: null,
        token
      }, '*');
      return;
    }

    this.provider.send(method, params, (error, result) => {
      source.postMessage({
        error,
        id,
        from: 'shell',
        result,
        token
      }, '*');
    });
  }

  setPermissions (permissions) {
    this.permissions = permissions;
  }

  static instance = null;

  static create (provider, permissions) {
    DappFilter.instance = new DappFilter(provider, permissions);
  }

  static get () {
    return DappFilter.instance;
  }
}
