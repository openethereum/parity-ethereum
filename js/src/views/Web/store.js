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

import { action, computed, observable, transaction } from 'mobx';
import localStore from 'store';
import { parse as parseUrl } from 'url';

import { encodePath, encodeUrl } from '~/util/dapplink';

const DEFAULT_URL = 'https://oasisdex.com';
const LS_LAST_ADDRESS = '_parity::webLastAddress';

const hasProtocol = /^https?:\/\//;

let instance = null;

export default class Store {
  @observable counter = Date.now();
  @observable currentUrl = null;
  @observable history = [];
  @observable isLoading = false;
  @observable parsedUrl = null;
  @observable nextUrl = null;
  @observable token = null;

  constructor (api) {
    this._api = api;

    this.nextUrl = this.currentUrl = this.loadLastUrl();
  }

  @computed get encodedPath () {
    return `${this._api.dappsUrl}/web/${encodePath(this.token, this.currentUrl)}?t=${this.counter}`;
  }

  @computed get encodedUrl () {
    return `http://${encodeUrl(this.token, this.currentUrl)}:${this._api.dappsPort}?t=${this.counter}`;
  }

  @computed get frameId () {
    return `_web_iframe_${this.counter}`;
  }

  @computed get isPristine () {
    return this.currentUrl === this.nextUrl;
  }

  @action gotoUrl = (_url) => {
    let url = (_url || this.nextUrl).trim().replace(/\/+$/, '');

    if (!hasProtocol.test(url)) {
      url = `https://${url}`;
    }
    this.setNextUrl(url);
    return this.generateToken(url).then(() => {
      this.setCurrentUrl(this.nextUrl);
    });
  }

  @action reload = () => {
    transaction(() => {
      this.setLoading(true);
      this.counter = Date.now();
    });
  }

  @action restoreUrl = () => {
    this.setNextUrl(this.currentUrl);
  }

  @action setHistory = (urls) => {
    this.history = Object
      .keys(urls)
      .filter((url) => url && !url.startsWith(this._api.dappsUrl) && url.indexOf('127.0.0.1') === -1)
      .sort((urlA, urlB) => {
        const timeA = urls[urlA].getTime();
        const timeB = urls[urlB].getTime();

        if (timeA > timeB) {
          return -1;
        } else if (timeA < timeB) {
          return 1;
        }

        return 0;
      })
      .map((url) => {
        const hostname = url.replace(/^http[s]?:\/\//, '').split('/')[0];

        return {
          hostname,
          timestamp: urls[url],
          url
        };
      });
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

      this.saveLastUrl();

      this.reload();
    });
  }

  @action setNextUrl = (url) => {
    this.nextUrl = url;
  }

  generateToken = (_url) => {
    this.setToken(null);

    return this._api.signer
      .generateWebProxyAccessToken(_url)
      .then((token) => {
        this.setToken(token);
      })
      .catch((error) => {
        console.warn('generateToken', error);
      });
  }

  loadHistory = () => {
    return this._api.parity
      .listRecentDapps()
      .then((apps) => {
        this.setHistory(apps);
      })
      .catch((error) => {
        console.warn('loadHistory', error);
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
  LS_LAST_ADDRESS
};
