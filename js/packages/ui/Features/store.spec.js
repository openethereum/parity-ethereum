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

import lstore from 'store';

import { LS_STORE_KEY } from './constants';
import defaults, { MODES } from './defaults';

import Store from './store';

let store;

function createStore () {
  store = new Store();

  return store;
}

describe('ui/Features/Store', () => {
  beforeEach(() => {
    lstore.set(LS_STORE_KEY, { 'testingFromStorage': true });
    createStore();
  });

  it('loads with values from localStorage', () => {
    expect(store.active.testingFromStorage).to.be.true;
  });

  describe('@action', () => {
    describe('setActiveFeatures', () => {
      it('sets the active features', () => {
        store.setActiveFeatures({ 'testing': true });
        expect(store.active.testing).to.be.true;
      });

      it('overrides the defaults', () => {
        store.setActiveFeatures({ '.PRODUCTION': false });
        expect(store.active['.PRODUCTION']).to.be.false;
      });
    });

    describe('toggleActive', () => {
      it('changes the state of a feature', () => {
        expect(store.active['.PRODUCTION']).to.be.true;
        store.toggleActive('.PRODUCTION');
        expect(store.active['.PRODUCTION']).to.be.false;
      });

      it('saves the updated state to localStorage', () => {
        store.toggleActive('.PRODUCTION');
        expect(lstore.get(LS_STORE_KEY)).to.deep.equal(store.active);
      });
    });
  });

  describe('operations', () => {
    describe('getDefaultActive', () => {
      it('returns features where mode === TESTING|PRODUCTION (non-production)', () => {
        const visibility = store.getDefaultActive(false);

        expect(
          Object
            .keys(visibility)
            .filter((key) => visibility[key])
            .filter((key) => ![MODES.TESTING, MODES.PRODUCTION].includes(defaults[key].mode))
            .length
        ).to.equal(0);
      });

      it('returns features where mode === PRODUCTION (production)', () => {
        const visibility = store.getDefaultActive(true);

        expect(
          Object
            .keys(visibility)
            .filter((key) => visibility[key])
            .filter((key) => ![MODES.PRODUCTION].includes(defaults[key].mode))
            .length
        ).to.equal(0);
      });
    });
  });
});
