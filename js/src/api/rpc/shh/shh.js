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

export default class Personal {
  constructor (transport) {
    this._transport = transport;
  }

  addToGroup (identity) {
    return this._transport
      .execute('shh_addToGroup', identity);
  }

  getFilterChanges (filterId) {
    return this._transport
      .execute('shh_getFilterChanges', filterId);
  }

  getMessages (filterId) {
    return this._transport
      .execute('shh_getMessages', filterId);
  }

  hasIdentity (identity) {
    return this._transport
      .execute('shh_hasIdentity', identity);
  }

  newFilter (options) {
    return this._transport
      .execute('shh_newFilter', options);
  }

  newGroup () {
    return this._transport
      .execute('shh_newGroup');
  }

  newIdentity () {
    return this._transport
      .execute('shh_newIdentity');
  }

  post (options) {
    return this._transport
      .execute('shh_post', options);
  }

  uninstallFilter (filterId) {
    return this._transport
      .execute('shh_uninstallFilter', filterId);
  }

  version () {
    return this._transport
      .execute('shh_version');
  }
}
