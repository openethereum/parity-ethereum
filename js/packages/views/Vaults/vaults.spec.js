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

import Vaults from './vaults';

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

describe('views/Vaults', () => {
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

    describe('renderList', () => {
      it('renders empty when no vaults', () => {
        instance.vaultStore.setVaults([], [], []);

        expect(
          shallow(instance.renderList()).find('FormattedMessage').props().id
        ).to.equal('vaults.empty');
      });

      describe('SectionList', () => {
        let list;

        beforeEach(() => {
          instance.vaultStore.setVaults(['testing'], [], ['meta']);
          list = instance.renderList();
        });

        it('renders', () => {
          expect(list).to.ok;
        });

        it('passes the vaults', () => {
          expect(list.props.items.peek()).to.deep.equal(instance.vaultStore.vaults.peek());
        });

        it('renders via renderItem', () => {
          expect(list.props.renderItem).to.deep.equal(instance.renderVault);
        });
      });
    });

    describe('renderVault', () => {
      const VAULT = { name: 'testing', isOpen: true, meta: 'meta' };
      let card;

      beforeEach(() => {
        card = instance.renderVault(VAULT);
      });

      it('renders', () => {
        expect(card).to.be.ok;
      });

      it('passes the vault', () => {
        expect(card.props.vault).to.deep.equal(VAULT);
      });
    });
  });

  describe('event methods', () => {
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

    describe('onOpenEdit', () => {
      beforeEach(() => {
        sinon.spy(instance.vaultStore, 'openMetaModal');

        instance.onOpenEdit('testing');
      });

      afterEach(() => {
        instance.vaultStore.openMetaModal.restore();
      });

      it('calls into vaultStore.openMetaModal', () => {
        expect(instance.vaultStore.openMetaModal).to.have.been.calledWith('testing');
      });
    });

    describe('onOpenLockVault', () => {
      beforeEach(() => {
        sinon.spy(instance.vaultStore, 'openLockModal');

        instance.onOpenLockVault('testing');
      });

      afterEach(() => {
        instance.vaultStore.openLockModal.restore();
      });

      it('calls into vaultStore.openLockModal', () => {
        expect(instance.vaultStore.openLockModal).to.have.been.calledWith('testing');
      });
    });

    describe('onOpenUnlockVault', () => {
      beforeEach(() => {
        sinon.spy(instance.vaultStore, 'openUnlockModal');

        instance.onOpenUnlockVault('testing');
      });

      afterEach(() => {
        instance.vaultStore.openUnlockModal.restore();
      });

      it('calls into vaultStore.openUnlockModal', () => {
        expect(instance.vaultStore.openUnlockModal).to.have.been.calledWith('testing');
      });
    });
  });
});
