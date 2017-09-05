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
import localStore from 'store';

import { syncRpcStateFromLocalStorage } from '../actions/localstorage';
import rpcData from '../data/rpc.json';
import LocalStorageMiddleware from './localstorage';

describe('views/Status/middleware/localstorage', () => {
  let cut;
  let state;

  beforeEach('mock cut', () => {
    cut = new LocalStorageMiddleware();
    sinon.spy(cut, 'onAddRpcResponse');
    sinon.spy(cut, 'onResetRpcCalls');
    sinon.spy(cut, 'onInitApp');
    sinon.spy(cut, 'unshift');
    state = {
      rpc: {
        callNo: 1
      }
    };
  });

  it('should call onAddRpcResponse when respected action is dispatched', () => {
    // given
    const store = { getState: () => state };
    const next = sinon.spy();
    const middleware = cut.toMiddleware()(store)(next);
    const action = { type: 'add rpcResponse', payload: {} };

    expect(middleware).to.be.a('function');
    expect(action).to.be.an('object');

    // when
    middleware(action);

    // then
    expect(cut.onAddRpcResponse.calledWith(store, next, action)).to.be.true;
  });

  it('should call onResetRpcCalls when respected action is dispactched', () => {
    // given
    const store = {};
    const next = sinon.spy();
    const middleware = cut.toMiddleware()(store)(next);
    const action = { type: 'reset rpcPrevCalls', payload: {} };

    expect(middleware).to.be.a('function');
    expect(action).to.be.an('object');

    // when
    middleware(action);

    // then
    expect(cut.onResetRpcCalls.calledWith(store, next, action)).to.be.true;
  });

  it('should call onInitApp when respected action is dispatched', () => {
    // given
    const store = { dispatch: sinon.spy() };
    const next = sinon.spy();
    const middleware = cut.toMiddleware()(store)(next);
    const action = { type: 'init app' };

    cut.onInitApp = sinon.spy();
    expect(middleware).to.be.a('function');
    expect(action).to.be.an('object');

    // when
    middleware(action);

    // then
    expect(cut.onInitApp.calledWith(store, next, action)).to.be.true;
  });

  it('should not call onAddRpcResponse or onInitApp when a non-respected action is dispatched', () => {
    // given
    const store = {};
    const next = sinon.spy();
    const middleware = cut.toMiddleware()(store)(next);
    const action = { type: 'testAction' };

    expect(middleware).to.be.a('function');
    expect(action).to.be.an('object');

    // when
    middleware(action);

    // then
    expect(cut.onAddRpcResponse.called).to.be.false;
    expect(cut.onInitApp.called).to.be.false;
    expect(next.calledWith(action)).to.be.true;
  });

  describe('RPC', () => {
    it('should dispatch syncRpcStateFromLocalStorage when there are rpc calls in localStorage', () => {
      // given
      const store = { dispatch: sinon.spy() };
      const next = sinon.spy();
      const action = {};
      const key = 'rpcPrevCalls';
      const prevCalls = [rpcData.methods[0]];

      prevCalls[0].callNo = 1;
      localStore.remove(key);
      localStore.set(key, prevCalls);

      // when
      cut.onInitApp(store, next, action);

      // then
      expect(store.dispatch.calledWith(syncRpcStateFromLocalStorage({
        prevCalls: prevCalls,
        callNo: 2,
        selectedMethod: prevCalls[0]
      }))).to.be.true;
      expect(next.calledWith(action)).to.be.true;
    });

    it('should not dispatch syncRpcStateFromLocalStorage when there are no rpc calls in localStorage', () => {
      // given
      const store = { dispatch: sinon.spy() };
      const next = sinon.spy();
      const action = {};

      localStore.remove('rpcPrevCalls');

      // when
      cut.onInitApp(store, next, action);

      // then
      expect(store.dispatch.notCalled).to.be.true;
      expect(next.calledWith(action)).to.be.true;
    });
  });

  it('should call unshift and next', () => {
    // given
    const store = { getState: () => state };
    const next = sinon.spy();
    const action = { payload: {} };

    // when
    cut.onAddRpcResponse(store, next, action);

    // then
    expect(cut.unshift.calledWith('rpcPrevCalls', action.payload)).to.be.true;
    expect(action.payload.callNo).to.equal(1);
    expect(next.calledWith(action)).to.be.true;
  });

  describe('UNSHIFT', () => {
    // TODO [adgo] 20.04.2016 remove if/when PR is accepted: https://github.com/marcuswestin/store.js/pull/153
    it('should create array in local storage by key and unshift item to it', () => {
      // given
      const key = 'foo';
      const val = 'bar';

      localStore.remove(key);

      // when
      cut.unshift(key, val);

      // then
      expect(localStore.get(key)[0]).to.equal(val);
      expect(localStore.get(key).length).to.equal(1);
    });

    // TODO [adgo] 20.04.2016 remove if/when PR is accepted: https://github.com/marcuswestin/store.js/pull/153
    it('should unshift item to an existing array in local storage by key', () => {
      // given
      const key = 'foo';
      const val = 'bar';
      const newVal = 'bazz';

      localStore.remove(key);
      localStore.set(key, [val]);
      expect(localStore.get(key)).to.be.defined;

      // when
      cut.unshift(key, newVal);

      // then
      expect(localStore.get(key)[0]).to.equal(newVal);
      expect(localStore.get(key).length).to.equal(2);
    });
  });
});
