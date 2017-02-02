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

import browser from 'useragent.js/lib/browser';
import store from 'store';

const LAST_HIDDEN = '_parity::extensionWarning::lastHidden';

export const EXTENSION_PAGE = 'https://chrome.google.com/webstore/detail/fgodinogimdopkigkcoelpfkbnpngalc';

export const shouldShowWarning = () => {
  const hasExtension = Symbol.for('parity.extension') in window;

  if (hasExtension) {
    return false;
  }

  const ua = browser.analyze(navigator.userAgent || '');
  const browserName = (ua || {}).name.toLowerCase();

  if (browserName !== 'chrome') {
    return false;
  }

  const lastHidden = store.get(LAST_HIDDEN) || 0;

  return (Date.now() - lastHidden) >= 24 * 60 * 60 * 1000;
};

export const installExtension = () => {
  return new Promise((resolve, reject) => {
    const link = document.createElement('link');

    link.setAttribute('rel', 'chrome-webstore-item');
    link.setAttribute('href', EXTENSION_PAGE);
    document.querySelector('head').appendChild(link);

    if (chrome && chrome.webstore && chrome.webstore.install) { // eslint-disable-line
      chrome.webstore.install(EXTENSION_PAGE, resolve, reject); // eslint-disable-line
    } else {
      reject(new Error('Direct installation failed.'));
    }
  });
};

export const hideWarning = () => {
  store.set(LAST_HIDDEN, Date.now());
};
