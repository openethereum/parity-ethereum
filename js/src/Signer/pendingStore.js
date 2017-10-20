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

import { action, observable } from 'mobx';

let instance = null;

export default class PendingStore {
  @observable pending = [];

  constructor (api) {
    this._api = api;

    api.on('connected', this.subscribePending);

    if (api.isConnected) {
      this.subscribePending();
    }
  }

  @action confirmRequest = (id, payload) => {
  }

  @action rejectRequest = (id) => {
  }

  @action setPending = (pending = []) => {
    this.pending = pending;
  }

  subscribePending = () => {
    this._api.subscribe('signer_requestsToConfirm', (error, pending) => {
      if (!error) {
        this.setPending(pending);
      }
    });
  }

  static get (api) {
    if (!instance) {
      instance = new PendingStore(api);
    }

    return instance;
  }
}
