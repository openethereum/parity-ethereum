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

let ledger;
let vendor;

function createLedger (error = null) {
  vendor = {
    getAddress: (path, callback) => {
      callback(error, {
        address: TEST_ADDRESS.toLowerCase()
      });
    },
    getAppConfiguration: (callback) => {
      callback(error, {});
    },
    signTransaction: (path, rawTransaction, callback) => {
      callback(error, {});
    }
  };
  ledger = new Ledger(vendor);

  return ledger;
}

describe('3rdparty/ledger', () => {
  beforeEach(() => {
    createLedger();

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
    let response;

    beforeEach(() => {
      return ledger
        .scan()
        .then((_response) => {
          response = _response;
        });
    });

    it('calls into getAddress', () => {
      expect(vendor.getAddress).to.have.been.called;
    });

    it('converts the address to checksum', () => {
      expect(response.address).to.equal(TEST_ADDRESS);
    });
  });

  describe('signTransaction', () => {
    beforeEach(() => {
      return ledger.signTransaction();
    });

    it('calls into signTransaction', () => {
      expect(vendor.signTransaction).to.have.been.called;
    });
  });
});
