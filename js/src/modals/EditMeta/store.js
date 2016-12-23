// Copyright 2015, 2016 Parity Technologies (UK) Ltd.
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

import { observable } from 'mobx';

export default class Store {
  @observable address = null;
  @observable isAccount = false;
  @observable description = null;
  @observable descriptionError = null;
  @observable meta = {};
  @observable name = null;
  @observable nameError = null;
  @observable tags = [];

  constructor (api, account) {
    const { address, name, meta, uuid } = account;

    this._api = api;

    this.isAccount = !!uuid;
    this.address = address;
    this.meta = meta || {};
    this.name = name || '';

    this.description = this.meta.description || '';
    this.tags = this.meta.tags || [];
  }
}
