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

import Ledger from './';

const TEST_ADDRESS = '0x63Cf90D3f0410092FC0fca41846f596223979195';

let api;
let ledger;
let vendor;

function createApi () {
  api = {
    net: {
      version: sinon.stub().resolves('2')
    }
  };

  return api;
}

function createVendor (error = null) {
  vendor = {
    getAddress: (path, callback) => {
      callback({
        address: TEST_ADDRESS
      }, error);
    },
    getAppConfiguration: (callback) => {
      callback({}, error);
    },
    signTransaction: (path, rawTransaction, callback) => {
      callback({
        v: [39],
        r: [0],
        s: [0]
      }, error);
    }
  };

  return vendor;
}

function create (error) {
  ledger = new Ledger(createApi(), createVendor(error));

  return ledger;
}

describe('3rdparty/ledger', () => {
  beforeEach(() => {
    create();

    sinon.spy(vendor, 'getAddress');
    sinon.spy(vendor, 'getAppConfiguration');
    sinon.spy(vendor, 'signTransaction');
  });

  afterEach(() => {
    vendor.getAddress.restore();
    vendor.getAppConfiguration.restore();
    vendor.signTransaction.restore();
  });

  describe('getAppConfiguration', () => {
    beforeEach(() => {
      return ledger.getAppConfiguration();
    });

    it('calls into getAppConfiguration', () => {
      expect(vendor.getAppConfiguration).to.have.been.called;
    });
  });

  describe('scan', () => {
    beforeEach(() => {
      return ledger.scan();
    });

    it('calls into getAddress', () => {
      expect(vendor.getAddress).to.have.been.called;
    });
  });

  describe('signTransaction', () => {
    beforeEach(() => {
      return ledger.signTransaction({
        data: '0x0',
        gasPrice: 20000000,
        gasLimit: 1000000,
        nonce: 2,
        to: '0x63Cf90D3f0410092FC0fca41846f596223979195',
        value: 1
      });
    });

    it('retrieves chainId via API', () => {
      expect(api.net.version).to.have.been.called;
    });

    it('calls into signTransaction', () => {
      expect(vendor.signTransaction).to.have.been.called;
    });
  });
});
