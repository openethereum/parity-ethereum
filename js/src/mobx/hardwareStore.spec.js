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

import HardwareStore, { HW_SCAN_INTERVAL } from './hardwareStore';

const WALLET = {
  name: 'testing'
};

let api;
let clock;
let ledger;
let store;

function createApi () {
  api = {
    parity: {
      setAccountMeta: sinon.stub().resolves(true),
      setAccountName: sinon.stub().resolves(true)
    }
  };

  return api;
}

function createLedger () {
  ledger = {
    getAppConfiguration: sinon.stub().resolves(),
    scan: sinon.stub().resolves(WALLET),
    signTransaction: sinon.stub().resolves()
  };

  return ledger;
}

function create () {
  clock = sinon.useFakeTimers();
  store = new HardwareStore(createApi());
  store._ledger = createLedger();

  return store;
}

function teardown () {
  clock.restore();
}

describe('mobx/HardwareStore', () => {
  beforeEach(() => {
    create();
  });

  afterEach(() => {
    teardown();
  });

  describe('@action', () => {
    describe('setScanning', () => {
      it('sets the flag', () => {
        store.setScanning('testScanning');
        expect(store.isScanning).to.equal('testScanning');
      });
    });

    describe('setWallet', () => {
      it('sets the wallet', () => {
        store.setWallet('testWallet');
        expect(store.wallet).to.equal('testWallet');
      });
    });
  });

  describe('operations', () => {
    describe('scanLedger', () => {
      beforeEach(() => {
        return store.scanLedger();
      });

      it('calls scan on the ledger', () => {
        expect(ledger.scan).to.have.been.called;
      });

      it('sets the wallet', () => {
        expect(store.wallet.name).to.equal(WALLET.name);
      });
    });

    describe('scan', () => {
      beforeEach(() => {
        sinon.spy(store, 'setScanning');
        sinon.spy(store, 'scanLedger');

        return store.scan();
      });

      afterEach(() => {
        store.setScanning.restore();
        store.scanLedger.restore();
      });

      it('calls scanLedger', () => {
        expect(store.scanLedger).to.have.been.called;
      });

      it('sets and resets the scanning state', () => {
        expect(store.setScanning).to.have.been.calledWith(true);
        expect(store.setScanning).to.have.been.calledWith(false);
      });
    });

    describe('background polling', () => {
      let pollId;

      beforeEach(() => {
        pollId = store._pollId;
        sinon.spy(store, 'scan');
      });

      afterEach(() => {
        store.scan.restore();
      });

      it('starts the polling at creation', () => {
        expect(pollId).not.to.be.null;
      });

      it('scans when timer elapsed', () => {
        expect(store.scan).not.to.have.been.called;
        clock.tick(HW_SCAN_INTERVAL + 1);
        expect(store.scan).to.have.been.called;
      });
    });
  });
});
