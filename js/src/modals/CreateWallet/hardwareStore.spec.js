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

import HardwareStore from './hardwareStore';

let api;
let store;

function createApi () {
  api = {};

  return api;
}

function create () {
  store = new HardwareStore(createApi());

  return store;
}

describe('CreateWallet/HardwareStore', () => {
  beforeEach(() => {
    create();
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
});
