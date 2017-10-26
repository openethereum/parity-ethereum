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

import flatten from 'flat';
import { action, observable, transaction } from 'mobx';
import { addLocaleData } from 'react-intl';
import de from 'react-intl/locale-data/de';
import en from 'react-intl/locale-data/en';
import nl from 'react-intl/locale-data/nl';
import zh from 'react-intl/locale-data/zh';
import store from 'store';

import { DEFAULT_LOCALE, DEFAULT_LOCALES, LS_STORE_KEY } from './constants';
import languages from './languages';
import deMessages from './de';
import enMessages from './en';
import nlMessages from './nl';
import zhMessages from './zh';
import zhHantTWMessages from './zh-Hant-TW';

let instance = null;

const LANGUAGES = flatten({ languages });
const MESSAGES = {
  de: Object.assign(flatten(deMessages), LANGUAGES),
  en: Object.assign(flatten(enMessages), LANGUAGES),
  nl: Object.assign(flatten(nlMessages), LANGUAGES),
  zh: Object.assign(flatten(zhMessages), LANGUAGES),
  'zh-Hant-TW': Object.assign(flatten(zhHantTWMessages), LANGUAGES)
};

addLocaleData([...de, ...en, ...nl, ...zh]);

export default class Store {
  @observable locale = DEFAULT_LOCALE;
  @observable locales = DEFAULT_LOCALES;
  @observable messages = MESSAGES[DEFAULT_LOCALE];

  constructor () {
    const savedLocale = store.get(LS_STORE_KEY);

    this.locale = (savedLocale && DEFAULT_LOCALES.includes(savedLocale))
      ? savedLocale
      : DEFAULT_LOCALE;
    this.messages = MESSAGES[this.locale];
  }

  @action setLocale (locale) {
    transaction(() => {
      this.locale = locale;
      this.messages = MESSAGES[locale];

      store.set(LS_STORE_KEY, locale);
    });
  }

  static get () {
    if (!instance) {
      instance = new Store();
    }

    return instance;
  }
}

export {
  LANGUAGES,
  MESSAGES
};
