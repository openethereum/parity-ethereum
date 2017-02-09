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

function createReduxStore () {
  reduxStore = {
    dispatch: sinon.stub(),
    subscribe: sinon.stub(),
    getState: sinon.stub()
  };

  return reduxStore;
}

function createVaultStore () {
  vaultStore = {
    isBusyCreate: false,
    isModalCreateOpen: true,
    createDescription: 'initialDesc',
    createName: 'initialName',
    createPassword: 'initialPassword',
    createPasswordRepeat: 'initialPassword',
    createPasswordHint: 'initialHint',
    closeCreateModal: sinon.stub(),
    createVault: sinon.stub().resolves(true),
    setCreateDescription: sinon.stub(),
    setCreateName: sinon.stub(),
    setCreatePassword: sinon.stub(),
    setCreatePasswordHint: sinon.stub(),
    setCreatePasswordRepeat: sinon.stub()
  };

  return vaultStore;
}

function render () {
  component = shallow(
    <VaultCreate vaultStore={ createVaultStore() } />,
    {
      context: {
        store: createReduxStore()
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

      it('calls setCreateDescription', () => {
        expect(vaultStore.setCreateDescription).to.have.been.calledWith('testDescription');
      });
    });

    describe('onEditName', () => {
      beforeEach(() => {
        instance.onEditName(null, 'testName');
      });

      it('calls setCreateName', () => {
        expect(vaultStore.setCreateName).to.have.been.calledWith('testName');
      });
    });

    describe('onEditPassword', () => {
      beforeEach(() => {
        instance.onEditPassword(null, 'testPassword');
      });

      it('calls setCreatePassword', () => {
        expect(vaultStore.setCreatePassword).to.have.been.calledWith('testPassword');
      });
    });

    describe('onEditPasswordHint', () => {
      beforeEach(() => {
        instance.onEditPasswordHint(null, 'testPasswordHint');
      });

      it('calls setCreatePasswordHint', () => {
        expect(vaultStore.setCreatePasswordHint).to.have.been.calledWith('testPasswordHint');
      });
    });

    describe('onEditPasswordRepeat', () => {
      beforeEach(() => {
        instance.onEditPasswordRepeat(null, 'testPassword');
      });

      it('calls setCreatePasswordRepeat', () => {
        expect(vaultStore.setCreatePasswordRepeat).to.have.been.calledWith('testPassword');
      });
    });
  });
});
