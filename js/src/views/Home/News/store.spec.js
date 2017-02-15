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

import { VERSION_ID } from './news';
import { restoreGlobals, stubGlobals } from './news.test.js';
import Store from './store';

let store;

function create () {
  store = new Store();

  return store;
}

describe('views/Home/News/Store', () => {
  beforeEach(() => {
    stubGlobals();
    create();
  });

  afterEach(() => {
    restoreGlobals();
  });

  describe('@action', () => {
    describe('setNewsItems', () => {
      it('sets the items', () => {
        store.setNewsItems('testing');
        expect(store.newsItems).to.equal('testing');
      });
    });
  });

  describe('operations', () => {
    describe('retrieveNews', () => {
      it('retrieves the items', () => {
        return store.retrieveNews(VERSION_ID).then(() => {
          expect(store.newsItems).to.equal('testContent');
        });
      });
    });
  });
});
