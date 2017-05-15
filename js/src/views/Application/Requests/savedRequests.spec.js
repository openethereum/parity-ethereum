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
import store from 'store';

import SavedRequests, { LS_REQUESTS_KEY } from './savedRequests';

const DEFAULT_REQUEST = {
  requestId: '0x1',
  transaction: {}
};

const SIGNED_REQUEST = {
  requestId: '0x2',
  transactionHash: '0xabcdef',
  transaction: {}
};

const api = createApi();
const savedRequests = new SavedRequests();

function createApi () {
  return {
    parity: {
      checkRequest: sinon.stub().resolves(),
      localTransactions: sinon.stub().resolves([])
    }
  };
}

describe('views/Application/Requests/savedRequests', () => {
  beforeEach((done) => {
    store.set(LS_REQUESTS_KEY, {
      [DEFAULT_REQUEST.requestId]: DEFAULT_REQUEST,
      [SIGNED_REQUEST.requestId]: SIGNED_REQUEST
    });

    savedRequests.load(api)
      .then(() => done());
  });

  afterEach(() => {
    store.set(LS_REQUESTS_KEY, {});
  });

  it('gets requests from local storage', () => {
    const requests = savedRequests._get();

    expect(requests[DEFAULT_REQUEST.requestId]).to.deep.equal(DEFAULT_REQUEST);
  });

  it('sets requests to local storage', () => {
    savedRequests._set({});

    const requests = savedRequests._get();

    expect(requests).to.deep.equal({});
  });

  it('removes requests', () => {
    savedRequests.remove(DEFAULT_REQUEST.requestId);

    const requests = savedRequests._get();

    expect(requests[DEFAULT_REQUEST.requestId]).to.be.undefined;
  });

  it('saves new requests', () => {
    savedRequests.save(DEFAULT_REQUEST.requestId, { extraData: true });

    const requests = savedRequests._get();

    expect(requests[DEFAULT_REQUEST.requestId]).to.deep.equal({
      ...DEFAULT_REQUEST,
      extraData: true
    });
  });

  it('loads requests', () => {
    return savedRequests.load(api)
      .then((requests) => {
        expect(requests).to.deep.equal([ DEFAULT_REQUEST ]);
      });
  });
});
