// Copyright 2015, 2016 Ethcore (UK) Ltd.
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
import _ from 'lodash';

import rpcData from '../data/rpc.json';
import RpcMiddleware from './rpc';
import * as RpcActions from '../actions/rpc';

describe('MIDDLEWARE: Rpc', () => {
  let cut;

  beforeEach('mock cut', () => {
    const request = sinon.spy();
    cut = new RpcMiddleware(request);
  });

  it('should not invoke request when a modify action is dispatched', () => {
    // given
    const store = null;
    const next = sinon.spy();
    const middleware = cut.toMiddleware()(store)(next);
    const action = { type: '_testAction' };
    expect(middleware).to.be.a('function');
    expect(action).to.be.an('object');

    // when
    middleware(action);

    // then
    expect(next.calledWith(action)).to.be.true;
    expect(cut._request.notCalled).to.be.true;
  });

  it('should invoke request when a modify action is dispatched', () => {
    // given
    const store = null;
    const next = sinon.spy();
    const middleware = cut.toMiddleware()(store)(next);
    const selectedMethod = _.find(rpcData.methods, { name: 'ethcore_minGasPrice' });
    const params = null;
    const action = RpcActions.fireRpc({
      method: selectedMethod.name,
      outputFormatter: selectedMethod.outputFormatter,
      inputFormatters: selectedMethod.inputFormatters,
      params: params
    });
    expect(middleware).to.be.a('function');
    expect(action).to.be.an('object');

    // when
    middleware(action);

    // then
    expect(next.calledWith(action)).to.be.true;
    expect(cut._request.calledWith({
      url: '/rpc/',
      method: 'POST',
      json: {
        id: 1000,
        method: selectedMethod.name,
        jsonrpc: '2.0',
        params: params // TODO :: add formatting
      }
    })).to.be.true;
  });

  it('should dispatch add rpc response on request CB', () => {
    // given
    const store = { dispatch: sinon.spy() };
    const method = 'testMethod';
    const params = [];
    const cb = (null, null, {});

    // when
    cut.responseHandler(store, method, params)(cb);

    // then
    expect(store.dispatch.called).to.be.true;
  });
});
