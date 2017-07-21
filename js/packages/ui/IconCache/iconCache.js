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

import { bytesToHex } from '@parity/api/util/format';

const ZERO = '0x0000000000000000000000000000000000000000000000000000000000000000';
const API_PATH = '/api/content/';

let instance = null;

export default class IconCache {
  @observable images = {};

  @action add (address, imageOrHash, isImage = false) {
    this.images = Object.assign({}, this.images, {
      [address]: isImage
        ? imageOrHash
        : IconCache.hashToImage(imageOrHash)
    });
  }

  static hashToImage (_hash) {
    const hash = _hash
      ? bytesToHex(_hash)
      : ZERO;

    return hash === ZERO
      ? null
      : `${API_PATH}${hash.substr(2)}`;
  }

  static get (force = false) {
    if (!instance || force) {
      instance = new IconCache();
    }

    return instance;
  }
}
