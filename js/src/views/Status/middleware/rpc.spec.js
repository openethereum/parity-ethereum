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
