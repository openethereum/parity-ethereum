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

import VaultCreate from './';

let component;
let instance;
let reduxStore;
let vaultStore;

function vaultReduxStore () {
  reduxStore = {
    dispatch: sinon.stub(),
    subscribe: sinon.stub(),
    getState: sinon.stub()
  };

  return reduxStore;
}

function vaultVaultStore () {
  vaultStore = {
    isBusyCreate: false,
    isModalCreateOpen: true,
    vaultDescription: 'initialDesc',
    vaultName: 'initialName',
    vaultPassword: 'initialPassword',
    vaultPasswordRepeat: 'initialPassword',
    vaultPasswordHint: 'initialHint',
    closeCreateModal: sinon.stub(),
    createVault: sinon.stub().resolves(true),
    setVaultDescription: sinon.stub(),
    setVaultName: sinon.stub(),
    setVaultPassword: sinon.stub(),
    setVaultPasswordHint: sinon.stub(),
    setVaultPasswordRepeat: sinon.stub()
  };

  return vaultStore;
}

function render () {
  component = shallow(
    <VaultCreate vaultStore={ vaultVaultStore() } />,
    {
      context: {
        store: vaultReduxStore()
      }
    }
  ).find('VaultCreate').shallow();
  instance = component.instance();

  return component;
}

describe('modals/VaultCreate', () => {
  beforeEach(() => {
    render();
  });

  it('renders defaults', () => {
    expect(component).to.be.ok;
  });

  describe('event handlers', () => {
    describe('onClose', () => {
      beforeEach(() => {
        instance.onClose();
      });

      it('calls into closeCreateModal', () => {
        expect(vaultStore.closeCreateModal).to.have.been.called;
      });
    });

    describe('onCreate', () => {
      beforeEach(() => {
        sinon.spy(instance, 'onClose');
        return instance.onCreate();
      });

      afterEach(() => {
        instance.onClose.restore();
      });

      it('calls into createVault', () => {
        expect(vaultStore.createVault).to.have.been.called;
      });

      it('closes modal', () => {
        expect(instance.onClose).to.have.been.called;
      });
    });

    describe('onEditDescription', () => {
      beforeEach(() => {
        instance.onEditDescription(null, 'testDescription');
      });

      it('calls setVaultDescription', () => {
        expect(vaultStore.setVaultDescription).to.have.been.calledWith('testDescription');
      });
    });

    describe('onEditName', () => {
      beforeEach(() => {
        instance.onEditName(null, 'testName');
      });

      it('calls setVaultName', () => {
        expect(vaultStore.setVaultName).to.have.been.calledWith('testName');
      });
    });

    describe('onEditPassword', () => {
      beforeEach(() => {
        instance.onEditPassword(null, 'testPassword');
      });

      it('calls setVaultPassword', () => {
        expect(vaultStore.setVaultPassword).to.have.been.calledWith('testPassword');
      });
    });

    describe('onEditPasswordHint', () => {
      beforeEach(() => {
        instance.onEditPasswordHint(null, 'testPasswordHint');
      });

      it('calls setVaultPasswordHint', () => {
        expect(vaultStore.setVaultPasswordHint).to.have.been.calledWith('testPasswordHint');
      });
    });

    describe('onEditPasswordRepeat', () => {
      beforeEach(() => {
        instance.onEditPasswordRepeat(null, 'testPassword');
      });

      it('calls setVaultPasswordRepeat', () => {
        expect(vaultStore.setVaultPasswordRepeat).to.have.been.calledWith('testPassword');
      });
    });
  });
});
