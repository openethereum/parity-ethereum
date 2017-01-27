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

import sinon from 'sinon';

import { createApi } from './parity.test.js';
import Store from './store';

let api;
let store;

function createStore () {
  api = createApi();
  store = new Store(api);

  return store;
}

describe('views/Settings/Parity/Store', () => {
  beforeEach(() => {
    createStore();
    sinon.spy(store, 'setMode');
  });

  afterEach(() => {
    store.setMode.restore();
  });

  it('defaults to mode === active', () => {
    expect(store.mode).to.equal('active');
  });

  describe('@action', () => {
    describe('setMode', () => {
      it('sets the mode', () => {
        store.setMode('offline');
        expect(store.mode).to.equal('offline');
      });
    });
  });

  describe('operations', () => {
    describe('changeMode', () => {
      beforeEach(() => {
        return store.changeMode('offline');
      });

      it('calls parity.setMode', () => {
        expect(api.parity.setMode).to.have.been.calledWith('offline');
      });

      it('sets the mode as provided', () => {
        expect(store.setMode).to.have.been.calledWith('offline');
      });
    });

    describe('loadMode', () => {
      beforeEach(() => {
        return store.loadMode();
      });

      it('calls parity.mode', () => {
        expect(api.parity.mode).to.have.been.called;
      });

      it('sets the mode as retrieved', () => {
        expect(store.setMode).to.have.been.calledWith('passive');
      });
    });
  });
});
