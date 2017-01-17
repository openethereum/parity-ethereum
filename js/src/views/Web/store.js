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

import { action, computed, observable, transaction } from 'mobx';
import localStore from 'store';
import { parse as parseUrl } from 'url';

const DEFAULT_URL = 'https://mkr.market';
const LS_LAST_ADDRESS = '_parity::webLastAddress';
const LS_HISTORY = '_parity::webHistory';

const hasProtocol = /^https?:\/\//;

let instance = null;

export default class Store {
  @observable counter = 1;
  @observable currentUrl = null;
  @observable displayedUrl = null;
  @observable history = [];
  @observable isLoading = false;
  @observable parsedUrl = null;
  @observable nextUrl = null;
  @observable token = null;

  constructor (api) {
    this._api = api;
    this.nextUrl = this.currentUrl = this.loadLastUrl();
  }

  @computed get frameId () {
    return `_web_iframe_${this.counter}`;
  }

  @computed get isPristine () {
    return this.currentUrl === this.nextUrl;
  }

  @action addHistoryUrl = (_url) => {
    const url = _url || this.currentUrl;

    this.history = [{
      timestamp: Date.now(),
      url
    }].concat(this.history.filter((h) => h.url !== url));
  }

  @action gotoUrl = (url) => {
    transaction(() => {
      this.setNextUrl(url || this.nextUrl);
      this.setCurrentUrl(this.nextUrl);
    });
  }

  @action reload = () => {
    transaction(() => {
      this.setLoading(true);
      this.counter++;
    });
  }

  @action restoreUrl = () => {
    this.setNextUrl(this.currentUrl);
  }

  @action setLoading = (isLoading) => {
    this.isLoading = isLoading;
  }

  @action setToken = (token) => {
    this.token = token;
  }

  @action setCurrentUrl = (_url) => {
    const url = _url || this.currentUrl;

    transaction(() => {
      this.currentUrl = url;
      this.parsedUrl = parseUrl(url);

      this.addHistoryUrl();
      this.saveLastUrl();

      this.reload();
    });
  }

  @action setNextUrl = (_url) => {
    let url = (_url || this.currentUrl).trim();

    if (!hasProtocol.test(url)) {
      url = `https://${url}`;
    }

    this.nextUrl = url;
  }

  generateToken = () => {
    return this._api.signer
      .generateWebProxyAccessToken()
      .then((token) => {
        this.setToken(token);
      })
      .catch((error) => {
        console.warn('generateToken', error);
      });
  }

  loadLastUrl = () => {
    return localStore.get(LS_LAST_ADDRESS) || DEFAULT_URL;
  }

  saveLastUrl = () => {
    return localStore.set(LS_LAST_ADDRESS, this.currentUrl);
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
