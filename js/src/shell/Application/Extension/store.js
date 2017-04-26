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

/* global chrome */

import { action, computed, observable } from 'mobx';

import store from 'store';
import browser from 'useragent.js/lib/browser';

import { DOMAIN } from '~/util/constants';

const A_DAY = 24 * 60 * 60 * 1000;
const NEXT_DISPLAY = '_parity::extensionWarning::nextDisplay';

// 'https://chrome.google.com/webstore/detail/parity-ethereum-integrati/himekenlppkgeaoeddcliojfddemadig';
const EXTENSION_PAGE = 'https://chrome.google.com/webstore/detail/himekenlppkgeaoeddcliojfddemadig';

let instance;

export default class Store {
  @observable hasExtension = false;
  @observable isInstalling = false;
  @observable nextDisplay = 0;
  @observable shouldInstall = false;

  constructor () {
    this.nextDisplay = store.get(NEXT_DISPLAY) || 0;
    this.testInstall();
  }

  @computed get showWarning () {
    return !this.isInstalling && this.shouldInstall && (Date.now() > this.nextDisplay);
  }

  @action setExtensionActive = () => {
    this.hasExtension = true;
  }

  @action setInstalling = (isInstalling) => {
    this.isInstalling = isInstalling;
  }

  @action snoozeWarning = (sleep = A_DAY) => {
    this.nextDisplay = Date.now() + sleep;
    store.set(NEXT_DISPLAY, this.nextDisplay);
  }

  @action testInstall = () => {
    this.shouldInstall = this.readStatus();
  }

  readStatus = () => {
    const hasExtension = Symbol.for('parity.extension') in window;
    const ua = browser.analyze(navigator.userAgent || '');

    if (hasExtension) {
      this.setExtensionActive();
      return false;
    }

    return (ua || {}).name.toLowerCase() === 'chrome';
  }

  installExtension = () => {
    this.setInstalling(true);

    if (window.location.hostname.toString().endsWith(DOMAIN)) {
      return this.inlineInstall()
        .catch((error) => {
          console.warn('Unable to perform direct install', error);
          window.open(EXTENSION_PAGE, '_blank');
        });
    }

    window.open(EXTENSION_PAGE, '_blank');
    return Promise.resolve(true);
  }

  inlineInstall = () => {
    return new Promise((resolve, reject) => {
      const link = document.createElement('link');

      link.setAttribute('rel', 'chrome-webstore-item');
      link.setAttribute('href', EXTENSION_PAGE);
      document.querySelector('head').appendChild(link);

      if (chrome && chrome.webstore && chrome.webstore.install) {
        chrome.webstore.install(EXTENSION_PAGE, resolve, reject);
      } else {
        reject(new Error('Direct installation failed.'));
      }
    });
  }

  static get () {
    if (!instance) {
      instance = new Store();
    }

    return instance;
  }
}
