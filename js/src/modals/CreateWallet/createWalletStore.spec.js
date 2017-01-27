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

import CreateWalletStore from './CreateWalletStore';

const ACCOUNTS = {
  '0x0123456789012345678901234567890123456789': {
    address: '0x0123456789012345678901234567890123456789'
  }
};

let api;
let store;

function createApi () {
  api = {};

  return api;
}

function create () {
  store = new CreateWalletStore(createApi(), ACCOUNTS);

  return store;
}

describe('modals/CreateWallet/Store', () => {
  beforeEach(() => {
    create();
  });

  describe('@action', () => {
    describe('onTypeChange', () => {
      it('changes the type', () => {
        expect(store.walletType).not.to.equal('HARDWARE_LEDGER');
        store.onTypeChange('HARDWARE_LEDGER');
        expect(store.walletType).to.equal('HARDWARE_LEDGER');
      });
    });
  });

  describe('@computed', () => {
    describe('steps', () => {
      it('returns non-deployment steps', () => {
        store.onTypeChange('WATCH');
        expect(store.steps).to.have.length(3);
      });

      it('returns deployment steps', () => {
        store.onTypeChange('MULTISIG');
        expect(store.steps).to.have.length(4);
      });
    });

    describe('waiting', () => {
      it('returns empty when non-waiting', () => {
        store.onTypeChange('WATCH');
        expect(store.waiting).to.have.length(0);
      });

      it('returns non-empty when waiting', () => {
        store.onTypeChange('MULTISIG');
        expect(store.waiting).to.have.length(1);
      });
    });
  });
});
