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

const A_MINUTE = 60 * 1000;
const A_DAY = 24 * 60 * A_MINUTE;
const TEN_MINUTES = 10 * A_MINUTE;
const NEXT_DISPLAY = '_parity::extensionWarning::nextDisplay';

// 'https://chrome.google.com/webstore/detail/fgodinogimdopkigkcoelpfkbnpngalc';
const EXTENSION_PAGE = 'https://chrome.google.com/webstore/detail/parity-ethereum-integrati/fgodinogimdopkigkcoelpfkbnpngalc';

export default class Store {
  @observable shouldInstall = false;
  @observable nextDisplay = 0;

  constructor () {
    this.nextDisplay = store.get(NEXT_DISPLAY) || 0;
    this.testInstall();
  }

  @computed get shouldShowWarning () {
    return this.shouldInstall && (Date.now() > this.nextDisplay);
  }

  @action hideWarning = (sleep = A_DAY) => {
    this.nextDisplay = Date.now() + sleep;
    store.set(NEXT_DISPLAY, this.nextDisplay);
  }

  @action testInstall = () => {
    this.shouldInstall = this.readStatus();
    console.log('testInstall', this.shouldInstall);
  }

  readStatus = () => {
    const hasExtension = Symbol.for('parity.extension') in window;
    const ua = browser.analyze(navigator.userAgent || '');

    console.log('readStatus', hasExtension, ua);

    if (hasExtension) {
      return false;
    }

    return (ua || {}).name.toLowerCase() === 'chrome';
  }

  installExtension = () => {
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
    })
    .catch((error) => {
      console.warn('Unable to perform direct install', error);
      window.open(EXTENSION_PAGE, '_blank');

      this.hideWarning(TEN_MINUTES + A_MINUTE);
      setTimeout(() => {
        this.testInstall();
      }, TEN_MINUTES);
    });
  }
}
