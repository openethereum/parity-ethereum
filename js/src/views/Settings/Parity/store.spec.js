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
    sinon.spy(store, 'setChain');
  });

  afterEach(() => {
    store.setMode.restore();
    store.setChain.restore();
  });

  it('defaults to mode === active', () => {
    expect(store.mode).to.equal('active');
  });

  it('defaults to chain === foundation', () => {
    expect(store.chain).to.equal('foundation');
  });

  describe('@action', () => {
    describe('setMode', () => {
      it('sets the mode', () => {
        store.setMode('offline');
        expect(store.mode).to.equal('offline');
      });
    });

    describe('setChain', () => {
      it('sets the chain', () => {
        store.setChain('dev');
        expect(store.chain).to.equal('dev');
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

    describe('changeChain', () => {
      beforeEach(() => {
        return store.changeChain('dev');
      });

      it('calls parity.setChain', () => {
        expect(api.parity.setChain).to.have.been.calledWith('dev');
      });

      it('sets the chain as provided', () => {
        expect(store.setChain).to.have.been.calledWith('dev');
      });
    });

    describe('loadChain', () => {
      beforeEach(() => {
        return store.loadChain();
      });

      it('calls parity.chain', () => {
        expect(api.parity.chain).to.have.been.called;
      });

      it('sets the chain as retrieved', () => {
        expect(store.setChain).to.have.been.calledWith('foundation');
      });
    });
  });
});
