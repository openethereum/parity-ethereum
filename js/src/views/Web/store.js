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

import { action, observable, transaction } from 'mobx';
import { parse as parseQuery } from 'querystring';
import localStore from 'store';
import { parse as parseUrl, format as formatUrl } from 'url';

const DEFAULT_URL = 'https://mkr.market';
const LS_LAST_ADDRESS = '_parity::webLastAddress';
const LS_HISTORY = '_parity::webHistory';

const hasProtocol = /^https?:\/\//;

let instance = null;

export default class Store {
  @observable displayedUrl = null;
  @observable history = [];
  @observable parsedUrl = null;
  @observable token = null;
  @observable url = null;

  constructor (api) {
    this._api = api;
  }

  @action addHistoryUrl = (url) => {
    const timestamp = Date.now();
    this.urlhistory = [{ url, timestamp }].concat(this.urlhistory.filter((h) => h.url !== url));
  }

  @action setToken = (token) => {
    this.token = token;
  }

  @action setUrl = (url) => {
    url = url || this.retrieveStored();

    if (!hasProtocol.test(url)) {
      url = `https://${url}`;
    }

    transaction(() => {
      this.displayedUrl = url;
      this.parsedUrl = parseUrl(url);
      this.url = url;
    });
  }

  generateToken = () => {
    return this._api
      .signer
      .generateWebProxyAccessToken()
      .then((token) => {
        this.setToken(token);
      })
      .catch((error) => {
        console.warn('generateToken', error);
      });
  }

  gotoUrl = () => {
    this.addHistoryUrl();
  }

  retrieveStored = () => {
    return localStore.get(LS_LAST_ADDRESS) || DEFAULT_URL;
  }

  static get (api) {
    if (!instance) {
      instance = new Store(api);
    }

    return instance;
  }
}

export {
  DEFAULT_URL,
  LS_LAST_ADDRESS,
  LS_HISTORY
};
