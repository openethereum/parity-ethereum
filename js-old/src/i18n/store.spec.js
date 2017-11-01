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

import store from 'store';

import { DEFAULT_LOCALE, DEFAULT_LOCALES, LS_STORE_KEY } from './constants';
import { LocaleStore } from './';

let localeStore;

describe('i18n/Store', () => {
  before(() => {
    localeStore = LocaleStore.get();
    store.set(LS_STORE_KEY, 'testing');
  });

  it('creates a default instance', () => {
    expect(localeStore).to.be.ok;
  });

  it('sets the default locale to default (invalid localStorage)', () => {
    expect(localeStore.locale).to.equal(DEFAULT_LOCALE);
  });

  it('loads the locale from localStorage (valid localStorage)', () => {
    const testLocale = DEFAULT_LOCALES[DEFAULT_LOCALES.length - 1];

    store.set(LS_STORE_KEY, testLocale);

    const testStore = new LocaleStore();

    expect(testStore.locale).to.equal(testLocale);
  });

  it('lists the locales', () => {
    expect(localeStore.locales.length > 1).to.be.true;
  });

  it('lists locals including default', () => {
    expect(localeStore.locales.includes(DEFAULT_LOCALE)).to.be.true;
  });

  describe('@action', () => {
    describe('setLocale', () => {
      const testLocale = DEFAULT_LOCALES[DEFAULT_LOCALES.length - 1];

      beforeEach(() => {
        localeStore.setLocale(testLocale);
      });

      it('sets the locale as passed', () => {
        expect(localeStore.locale).to.equal(testLocale);
      });

      it('sets the locale in localStorage', () => {
        expect(store.get(LS_STORE_KEY)).to.equal(testLocale);
      });
    });
  });
});
