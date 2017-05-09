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

export default class DappReg {
  constructor (api, registry) {
    this._api = api;
    this._registry = registry;

    this.getInstance();
  }

  getContract () {
    return this._registry.getContract('dappreg');
  }

  getInstance () {
    return this.getContract().then((contract) => contract.instance);
  }

  count () {
    return this.getInstance().then((instance) => {
      return instance.count.call();
    });
  }

  at (index) {
    return this.getInstance().then((instance) => {
      return instance.at.call({}, [index]);
    });
  }

  get (id) {
    return this.getInstance().then((instance) => {
      return instance.get.call({}, [id]);
    });
  }

  meta (id, key) {
    return this.getInstance().then((instance) => {
      return instance.meta.call({}, [id, key]);
    });
  }

  getImage (id) {
    return this.meta(id, 'IMG');
  }

  getContent (id) {
    return this.meta(id, 'CONTENT');
  }

  getManifest (id) {
    return this.meta(id, 'MANIFEST');
  }
}
