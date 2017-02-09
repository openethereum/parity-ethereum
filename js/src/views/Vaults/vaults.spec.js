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

import { shallow } from 'enzyme';
import React from 'react';
import sinon from 'sinon';

import { createApi, createReduxStore } from './vaults.test.js';

import Vaults from './';

let api;
let component;
let instance;
let store;

function render (props = {}) {
  api = createApi();
  store = createReduxStore();

  component = shallow(
    <Vaults />,
    {
      context: { store }
    }
  ).find('Vaults').shallow({ context: { api } });
  instance = component.instance();

  return component;
}

describe('modals/Vaults', () => {
  beforeEach(() => {
    render();
  });

  it('renders defaults', () => {
    expect(component).to.be.ok;
  });

  describe('instance methods', () => {
    describe('componentWillMount', () => {
      beforeEach(() => {
        sinon.spy(instance.vaultStore, 'loadVaults');

        return instance.componentWillMount();
      });

      afterEach(() => {
        instance.vaultStore.loadVaults.restore();
      });

      it('calls into vaultStore.loadVaults', () => {
        expect(instance.vaultStore.loadVaults).to.have.been.called;
      });
    });
  });

  describe('event methods', () => {
    describe('onCloseVault', () => {
      beforeEach(() => {
        sinon.spy(instance.vaultStore, 'openCloseModal');

        instance.onCloseVault('testing');
      });

      afterEach(() => {
        instance.vaultStore.openCloseModal.restore();
      });

      it('calls into vaultStore.openCloseModal', () => {
        expect(instance.vaultStore.openCloseModal).to.have.been.calledWith('testing');
      });
    });

    describe('onOpenAccounts', () => {
      beforeEach(() => {
        sinon.spy(instance.vaultStore, 'openAccountsModal');

        instance.onOpenAccounts('testing');
      });

      afterEach(() => {
        instance.vaultStore.openAccountsModal.restore();
      });

      it('calls into vaultStore.openAccountsModal', () => {
        expect(instance.vaultStore.openAccountsModal).to.have.been.calledWith('testing');
      });
    });

    describe('onOpenCreate', () => {
      beforeEach(() => {
        sinon.spy(instance.vaultStore, 'openCreateModal');

        instance.onOpenCreate();
      });

      afterEach(() => {
        instance.vaultStore.openCreateModal.restore();
      });

      it('calls into vaultStore.openCreateModal', () => {
        expect(instance.vaultStore.openCreateModal).to.have.been.called;
      });
    });

    describe('onOpenVault', () => {
      beforeEach(() => {
        sinon.spy(instance.vaultStore, 'openOpenModal');

        instance.onOpenVault('testing');
      });

      afterEach(() => {
        instance.vaultStore.openOpenModal.restore();
      });

      it('calls into vaultStore.openOpenModal', () => {
        expect(instance.vaultStore.openOpenModal).to.have.been.calledWith('testing');
      });
    });
  });
});
