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

import { hideRequest, trackRequest, watchRequest } from './requestsActions';

const TX_HASH = '0x123456';
const BASE_REQUEST = {
  requestId: '0x1',
  transaction: {
    from: '0x0',
    to: '0x1'
  }
};

let api;
let store;
let dispatcher;

function createApi () {
  return {
    pollMethod: (method, data) => {
      switch (method) {
        case 'parity_checkRequest':
          return Promise.resolve(TX_HASH);

        default:
          return Promise.resolve();
      }
    }
  };
}

function createRedux (dispatcher) {
  return {
    dispatch: (arg) => {
      if (typeof arg === 'function') {
        return arg(store.dispatch, store.getState);
      }

      return dispatcher(arg);
    },
    getState: () => {
      return {
        api,
        requests: {
          [BASE_REQUEST.requestId]: BASE_REQUEST
        }
      };
    }
  };
}

describe('redux/requests', () => {
  beforeEach(() => {
    api = createApi();
    dispatcher = sinon.spy();
    store = createRedux(dispatcher);
  });

  it('watches new requests', () => {
    store.dispatch(watchRequest(BASE_REQUEST));

    expect(dispatcher).to.be.calledWith({
      type: 'setRequest',
      requestId: BASE_REQUEST.requestId,
      requestData: BASE_REQUEST
    });
  });

  it('tracks requests', (done) => {
    store.dispatch(trackRequest(BASE_REQUEST.requestId));

    setTimeout(() => {
      expect(dispatcher).to.be.calledWith({
        type: 'setRequest',
        requestId: BASE_REQUEST.requestId,
        requestData: {
          transactionHash: TX_HASH,
          show: true
        }
      });

      done();
    }, 50);
  });

  it('hides requests', () => {
    store.dispatch(hideRequest(BASE_REQUEST.requestId));

    expect(dispatcher).to.be.calledWith({
      type: 'setRequest',
      requestId: BASE_REQUEST.requestId,
      requestData: { show: false }
    });
  });
});
