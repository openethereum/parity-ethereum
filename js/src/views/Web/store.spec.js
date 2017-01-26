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

import Store from './store';

const TEST_TOKEN = 'testing-123';
const TEST_URL1 = 'http://some.test.domain.com';
const TEST_URL2 = 'http://something.different.com';
const TEST_URL3 = 'https://world.wonders.xyz';

let api;
let store;

function createApi () {
  api = {
    dappsPort: 8080,
    signer: {
      generateWebProxyAccessToken: sinon.stub().resolves(TEST_TOKEN)
    }
  };

  return api;
}

function create () {
  store = new Store(createApi());

  return store;
}

describe('views/Web/Store', () => {
  beforeEach(() => {
    create();
    sinon.spy(store.historyStore, 'add');
  });

  afterEach(() => {
    store.historyStore.add.restore();
  });

  describe('@action', () => {
    describe('restoreUrl', () => {
      it('sets the nextUrl to the currentUrl', () => {
        store.setCurrentUrl(TEST_URL1);
        store.setNextUrl(TEST_URL2);
        store.restoreUrl();

        expect(store.nextUrl).to.equal(TEST_URL1);
      });
    });

    describe('setCurrentUrl', () => {
      beforeEach(() => {
        store.setCurrentUrl(TEST_URL1);
      });

      it('sets the url', () => {
        expect(store.currentUrl).to.equal(TEST_URL1);
      });

      it('saves the url in the history', () => {
        expect(store.historyStore.add).to.have.been.calledWith(TEST_URL1);
      });
    });

    describe('setLoading', () => {
      beforeEach(() => {
        store.setLoading(true);
      });

      it('sets the loading state (true)', () => {
        expect(store.isLoading).to.be.true;
      });

      it('sets the loading state (false)', () => {
        store.setLoading(false);

        expect(store.isLoading).to.be.false;
      });
    });

    describe('setNextUrl', () => {
      it('sets the url', () => {
        store.setNextUrl(TEST_URL1);

        expect(store.nextUrl).to.equal(TEST_URL1);
      });

      it('adds https when no protocol', () => {
        store.setNextUrl('google.com');

        expect(store.nextUrl).to.equal('https://google.com');
      });

      it('sets the currentUrl when none specified', () => {
        store.setCurrentUrl(TEST_URL3);
        store.setNextUrl();

        expect(store.nextUrl).to.equal(TEST_URL3);
      });
    });

    describe('setToken', () => {
      it('sets the token', () => {
        store.setToken(TEST_TOKEN);

        expect(store.token).to.equal(TEST_TOKEN);
      });
    });
  });

  describe('@computed', () => {
    describe('encodedUrl', () => {
      it('encodes current', () => {
        store.setCurrentUrl(TEST_URL1);
        expect(store.encodedUrl).to.match(/DSTPRV1BD1T78W1T5WQQ6VVDCMQ78SBKEGQ68VVDC5MPWBK3DXPG/);
      });
    });

    describe('frameId', () => {
      it('creates an id', () => {
        expect(store.frameId).to.be.ok;
      });
    });

    describe('isPristine', () => {
      it('is true when current === next', () => {
        store.setCurrentUrl(TEST_URL1);
        store.setNextUrl(TEST_URL1);

        expect(store.isPristine).to.be.true;
      });

      it('is false when current !== next', () => {
        store.setCurrentUrl(TEST_URL1);
        store.setNextUrl(TEST_URL2);

        expect(store.isPristine).to.be.false;
      });
    });
  });

  describe('operations', () => {
    describe('generateToken', () => {
      beforeEach(() => {
        return store.generateToken();
      });

      it('calls parity_generateWebProxyAccessToken', () => {
        expect(api.signer.generateWebProxyAccessToken).to.have.been.calledOnce;
      });

      it('sets the token as retrieved', () => {
        expect(store.token).to.equal(TEST_TOKEN);
      });
    });
  });
});
