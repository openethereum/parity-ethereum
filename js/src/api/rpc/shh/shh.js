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
  constructor (provider) {
    this._provider = provider;
  }

  addToGroup (identity) {
    return this._provider
      .send('shh_addToGroup', identity);
  }

  getFilterChanges (filterId) {
    return this._provider
      .send('shh_getFilterChanges', filterId);
  }

  getMessages (filterId) {
    return this._provider
      .send('shh_getMessages', filterId);
  }

  hasIdentity (identity) {
    return this._provider
      .send('shh_hasIdentity', identity);
  }

  newFilter (options) {
    return this._provider
      .send('shh_newFilter', options);
  }

  newGroup () {
    return this._provider
      .send('shh_newGroup');
  }

  newIdentity () {
    return this._provider
      .send('shh_newIdentity');
  }

  post (options) {
    return this._provider
      .send('shh_post', options);
  }

  uninstallFilter (filterId) {
    return this._provider
      .send('shh_uninstallFilter', filterId);
  }

  version () {
    return this._provider
      .send('shh_version');
  }
}
