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

import Store from './store';

let store;

describe('modals/UpgradeParity/store', () => {
  describe('@actions', () => {
    beforeEach(() => {
      store = new Store();
    });

    describe('openModal & closeModal', () => {
      it('toggles between the closed states', () => {
        expect(store.closed).to.be.true;
        store.openModal();
        expect(store.closed).to.be.false;
        store.closeModal();
        expect(store.closed).to.be.true;
      });

      it('resets the step state upon closing', () => {
        store.setStep(5, 'soem error');
        store.closeModal();
        expect(store.step).to.equal(0);
        expect(store.error).to.be.null;
      });
    });

    describe('setStep', () => {
      it('sets the step as provided', () => {
        expect(store.step).to.equal(0);
        store.setStep(3);
        expect(store.step).to.equal(3);
      });

      it('sets the error when provided', () => {
        expect(store.error).to.be.null;
        store.setStep(3, new Error('some error'));
        expect(store.error).to.match(/some error/);
      });
    });
  });
});
