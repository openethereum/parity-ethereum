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

import flatten from 'flat';
import { action, observable, transaction } from 'mobx';
import { addLocaleData } from 'react-intl';
import de from 'react-intl/locale-data/de';
import en from 'react-intl/locale-data/en';
import store from 'store';

import languages from './languages';
import deMessages from './de';
import enMessages from './en';

const LS_STORE_KEY = '_parity::locale';

let instance = null;
const isProduction = process.env.NODE_ENV === 'production';

const DEFAULT = 'en';
const LANGUAGES = flatten({ languages });
const MESSAGES = {
  de: Object.assign(flatten(deMessages), LANGUAGES),
  en: Object.assign(flatten(enMessages), LANGUAGES)
};
const LOCALES = isProduction
  ? ['en']
  : ['en', 'de'];

addLocaleData([...de, ...en]);

export default class Store {
  @observable locale = DEFAULT;
  @observable locales = LOCALES;
  @observable messages = MESSAGES[DEFAULT];
  @observable isDevelopment = !isProduction;

  constructor () {
    const savedLocale = store.get(LS_STORE_KEY);

    this.locale = (savedLocale && LOCALES.includes(savedLocale))
      ? savedLocale
      : DEFAULT;
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
