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

import VaultMeta from './';

const VAULT = {
  name: 'testVault'
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
    isBusyMeta: false,
    isModalMetaOpen: true,
    vault: VAULT,
    vaultDescription: '',
    vaultTags: [],
    vaultName: VAULT.name,
    vaults: [VAULT],
    closeMetaModal: sinon.stub(),
    editVault: sinon.stub().resolves(true),
    setVaultDescription: sinon.stub(),
    setVaultTags: sinon.stub()
  };

  return vaultStore;
}

function render (props = {}) {
  component = shallow(
    <VaultMeta vaultStore={ createVaultStore() } />,
    {
      context: {
        store: createReduxStore()
      }
    }
  ).find('VaultMeta').shallow();
  instance = component.instance();

  return component;
}

describe('modals/VaultMeta', () => {
  beforeEach(() => {
    render();
  });

  it('renders defaults', () => {
    expect(component).to.be.ok;
  });

  describe('event methods', () => {
    describe('onChangeDescription', () => {
      beforeEach(() => {
        instance.onChangeDescription(null, 'testing');
      });

      it('calls into setVaultDescription', () => {
        expect(vaultStore.setVaultDescription).to.have.been.calledWith('testing');
      });
    });

    describe('onChangeTags', () => {
      beforeEach(() => {
        instance.onChangeTags('testing');
      });

      it('calls into setVaultTags', () => {
        expect(vaultStore.setVaultTags).to.have.been.calledWith('testing');
      });
    });

    describe('onClose', () => {
      beforeEach(() => {
        instance.onClose();
      });

      it('calls into closeMetaModal', () => {
        expect(vaultStore.closeMetaModal).to.have.been.called;
      });
    });

    describe('onExecute', () => {
      beforeEach(() => {
        return instance.onExecute();
      });

      it('calls into editVault', () => {
        expect(vaultStore.editVault).to.have.been.called;
      });
    });
  });
});
