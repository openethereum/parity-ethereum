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

import Store from './historyStore';

const TEST_ENTRY_1 = 'testing 1';
const TEST_ENTRY_2 = 'testing 2';
const TEST_ENTRY_3 = 'testing 3';

let store;

function create () {
  store = Store.get('test');
  store.clear();

  return store;
}

describe('mobx/HistoryStore', () => {
  beforeEach(() => {
    create();
  });

  describe('@action', () => {
    describe('add', () => {
      it('adds the url to the list (front)', () => {
        store.add(TEST_ENTRY_1);
        expect(store.history[0].entry).to.equal(TEST_ENTRY_1);
      });

      it('adds multiples to the list', () => {
        store.add(TEST_ENTRY_1);
        store.add(TEST_ENTRY_2);

        expect(store.history.length).to.equal(2);
        expect(store.history[0].entry).to.equal(TEST_ENTRY_2);
        expect(store.history[1].entry).to.equal(TEST_ENTRY_1);
      });

      it('does not add duplicates', () => {
        store.add(TEST_ENTRY_2);
        store.add(TEST_ENTRY_2);

        expect(store.history.length).to.equal(1);
        expect(store.history[0].entry).to.equal(TEST_ENTRY_2);
      });
    });

    describe('clear', () => {
      it('empties the list', () => {
        store.add(TEST_ENTRY_3);
        store.clear();

        expect(store.history.length).to.equal(0);
      });
    });
  });
});
