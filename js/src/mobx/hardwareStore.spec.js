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

const ADDRESS = '0x1234567890123456789012345678901234567890';
const WALLET = {
  address: ADDRESS,
  name: 'testing'
};

let api;
let clock;
let ledger;
let store;

function createApi () {
  api = {
    parity: {
      hardwareAccountsInfo: sinon.stub().resolves({ ADDRESS: WALLET }),
      setAccountMeta: sinon.stub().resolves(true),
      setAccountName: sinon.stub().resolves(true)
    }
  };

  return api;
}

function createLedger () {
  ledger = {
    isSupported: true,
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

  describe('@computed', () => {
    describe('isConnected', () => {
      beforeEach(() => {
        store.setWallets({ [ADDRESS]: WALLET });
      });

      it('returns true for available', () => {
        expect(store.isConnected(ADDRESS)).to.be.true;
      });

      it('returns false for non-available', () => {
        expect(store.isConnected('nothing')).to.be.false;
      });
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

  describe('@action', () => {
    describe('setScanning', () => {
      it('sets the flag', () => {
        store.setScanning('testScanning');
        expect(store.isScanning).to.equal('testScanning');
      });
    });

    describe('setWallets', () => {
      it('sets the wallets', () => {
        store.setWallets('testWallet');
        expect(store.wallets).to.equal('testWallet');
      });
    });
  });

  describe('operations', () => {
    describe('createAccountInfo', () => {
      describe('when not existing', () => {
        beforeEach(() => {
          return store.createAccountInfo({
            address: 'testAddr',
            manufacturer: 'testMfg',
            name: 'testName'
          });
        });

        it('calls into parity_setAccountName', () => {
          expect(api.parity.setAccountName).to.have.been.calledWith('testAddr', 'testName');
        });

        it('calls into parity_setAccountMeta', () => {
          expect(api.parity.setAccountMeta).to.have.been.calledWith('testAddr', sinon.match({
            description: 'testMfg testName',
            hardware: {
              manufacturer: 'testMfg'
            },
            tags: ['hardware']
          }));
        });
      });

      describe('when already exists', () => {
        beforeEach(() => {
          return store.createAccountInfo({
            address: 'testAddr',
            manufacturer: 'testMfg',
            name: 'testName'
          }, {
            name: 'originalName',
            meta: {
              description: 'originalDescription',
              tags: ['tagA', 'tagB']
            }
          });
        });

        it('does not call into parity_setAccountName', () => {
          expect(api.parity.setAccountName).not.to.have.been.called;
        });

        it('calls into parity_setAccountMeta', () => {
          expect(api.parity.setAccountMeta).to.have.been.calledWith('testAddr', sinon.match({
            description: 'originalDescription',
            hardware: {
              manufacturer: 'testMfg'
            },
            tags: ['tagA', 'tagB']
          }));
        });
      });
    });

    describe('scanLedger', () => {
      beforeEach(() => {
        return store.scanLedger();
      });

      it('calls scan on the Ledger APIs', () => {
        expect(ledger.scan).to.have.been.called;
      });
    });

    describe('scanParity', () => {
      beforeEach(() => {
        return store.scanParity();
      });

      it('calls parity_hardwareAccountsInfo', () => {
        expect(api.parity.hardwareAccountsInfo).to.have.been.called;
      });
    });

    describe('scan', () => {
      beforeEach(() => {
        sinon.spy(store, 'setScanning');
        sinon.spy(store, 'setWallets');
        sinon.spy(store, 'scanLedger');
        sinon.spy(store, 'scanParity');

        return store.scan();
      });

      afterEach(() => {
        store.setScanning.restore();
        store.setWallets.restore();
        store.scanLedger.restore();
        store.scanParity.restore();
      });

      it('calls scanLedger', () => {
        expect(store.scanLedger).to.have.been.called;
      });

      it('calls scanParity', () => {
        expect(store.scanParity).to.have.been.called;
      });

      it('sets and resets the scanning state', () => {
        expect(store.setScanning).to.have.been.calledWith(true);
        expect(store.setScanning).to.have.been.calledWith(false);
      });

      it('sets the wallets', () => {
        expect(store.setWallets).to.have.been.called;
      });
    });

    describe('signLedger', () => {
      beforeEach(() => {
        return store.signLedger('testTx');
      });

      it('calls signTransaction on the ledger', () => {
        expect(ledger.signTransaction).to.have.been.calledWith('testTx');
      });
    });
  });
});
