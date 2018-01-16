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

const TEST_HISTORY_URLA = 'http://testingA';
const TEST_HISTORY_URLB = 'http://testingB';
const TEST_HISTORY = {
  '': new Date(678),
  [TEST_HISTORY_URLA]: new Date(123),
  [TEST_HISTORY_URLB]: new Date(456)
};
const TEST_TOKEN = 'testing-123';
const TEST_URL1 = 'http://some.test.domain.com';
const TEST_URL2 = 'http://something.different.com';

let api;
let store;

function createApi () {
  api = {
    dappsPort: 8545,
    dappsUrl: 'http://home.web3.site:8545',
    parity: {
      listRecentDapps: sinon.stub().resolves(TEST_HISTORY)
    },
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
  });

  describe('@action', () => {
    describe('gotoUrl', () => {
      it('uses the nextUrl when none specified', () => {
        store.setNextUrl('https://parity.io');

        return store.gotoUrl().then(() => {
          expect(store.currentUrl).to.equal('https://parity.io');
        });
      });

      it('adds https when no protocol', () => {
        return store.gotoUrl('google.com').then(() => {
          expect(store.currentUrl).to.equal('https://google.com');
        });
      });
    });

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
    });

    describe('setHistory', () => {
      let history;

      beforeEach(() => {
        store.setHistory(TEST_HISTORY);
        history = store.history.peek();
      });

      it('sets the history', () => {
        expect(history.length).to.equal(2);
      });

      it('adds hostname to entries', () => {
        expect(history[1].hostname).to.be.ok;
      });

      it('removes hostname http prefixes', () => {
        expect(history[1].hostname.indexOf('http')).to.equal(-1);
      });

      it('sorts the entries according to recently accessed', () => {
        expect(history[0].url).to.equal(TEST_HISTORY_URLB);
        expect(history[1].url).to.equal(TEST_HISTORY_URLA);
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
      describe('encodedPath', () => {
        it('encodes current', () => {
          store.setCurrentUrl(TEST_URL1);
          expect(store.encodedPath).to.match(
            /http:\/\/home\.web3\.site:8545\/web\/DSTPRV1BD1T78W1T5WQQ6VVDCMQ78SBKEGQ68VVDC5MPWBK3DXPG\?t=[0-9]*$/
          );
        });
      });

      it('encodes current', () => {
        store.setCurrentUrl(TEST_URL1);
        expect(store.encodedUrl).to.match(
          /^http:\/\/DSTPRV1BD1T78W1T5WQQ6VVDCMQ78SBKEGQ68VVDC5MPWBK3DXPG\.web\.web3\.site:8545\?t=[0-9]*$/
        );
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

      it('calls signer_generateWebProxyAccessToken', () => {
        expect(api.signer.generateWebProxyAccessToken).to.have.been.calledOnce;
      });

      it('sets the token as retrieved', () => {
        expect(store.token).to.equal(TEST_TOKEN);
      });
    });

    describe('loadHistory', () => {
      beforeEach(() => {
        return store.loadHistory();
      });

      it('calls parity_listRecentDapps', () => {
        expect(api.parity.listRecentDapps).to.have.been.calledOnce;
      });

      it('sets the history as retrieved', () => {
        expect(store.history.peek().length).not.to.equal(0);
      });
    });
  });
});
