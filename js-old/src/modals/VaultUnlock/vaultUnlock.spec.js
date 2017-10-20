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

import VaultUnlock from './';

const VAULT = {
  name: 'testVault',
  meta: {
    passwordHint: 'some hint'
  }
};

let component;
let instance;
let reduxStore;
let vaultStore;

function createReduxStore () {
  reduxStore = {
    dispatch: sinon.stub(),
    subscribe: sinon.stub(),
    getState: () => {
      return {};
    }
  };

  return reduxStore;
}

function createVaultStore () {
  vaultStore = {
    isBusyUnlock: false,
    isModalUnlockOpen: true,
    vault: VAULT,
    vaultName: VAULT.name,
    vaultPassword: 'testPassword',
    vaults: [VAULT],
    closeUnlockModal: sinon.stub(),
    openVault: sinon.stub().resolves(true),
    setVaultPassword: sinon.stub()
  };

  return vaultStore;
}

function render () {
  component = shallow(
    <VaultUnlock vaultStore={ createVaultStore() } />,
    {
      context: {
        store: createReduxStore()
      }
    }
  ).find('VaultUnlock').shallow();
  instance = component.instance();

  return component;
}

describe('modals/VaultUnlock', () => {
  beforeEach(() => {
    render();
  });

  it('renders defaults', () => {
    expect(component).to.be.ok;
  });

  describe('ConfirmDialog', () => {
    let dialog;

    beforeEach(() => {
      dialog = component.find('ConfirmDialog');
    });

    it('renders the dialog', () => {
      expect(dialog.get(0)).to.be.ok;
    });

    it('passes onConfirm as onExecute', () => {
      expect(dialog.props().onConfirm).to.equal(instance.onExecute);
    });

    it('passes onDeny as onClose', () => {
      expect(dialog.props().onDeny).to.equal(instance.onClose);
    });
  });

  describe('event methods', () => {
    describe('onExecute', () => {
      beforeEach(() => {
        sinon.stub(instance, 'onClose');
        return instance.onExecute();
      });

      afterEach(() => {
        instance.onClose.restore();
      });

      it('closes the modal', () => {
        expect(instance.onClose).to.have.been.called;
      });

      it('calls into vaultStore.openVault', () => {
        expect(vaultStore.openVault).to.have.been.called;
      });
    });

    describe('onClose', () => {
      beforeEach(() => {
        instance.onClose();
      });

      it('calls into closeUnlockModal', () => {
        expect(vaultStore.closeUnlockModal).to.have.been.called;
      });
    });

    describe('onEditPassword', () => {
      beforeEach(() => {
        instance.onEditPassword(null, 'someVaultPassword');
      });

      it('calls into vaultStore.setVaultPassword', () => {
        expect(vaultStore.setVaultPassword).to.have.been.calledWith('someVaultPassword');
      });
    });
  });
});
