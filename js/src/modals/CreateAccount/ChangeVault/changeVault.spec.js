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

import ChangeVault from './';

let component;
let instance;
let store;
let vaultStore;

function createStore () {
  store = {
    setVaultName: sinon.stub(),
    vaultName: 'testing'
  };

  return store;
}

function createVaultStore () {
  vaultStore = {
    vaultsOpened: ['testing']
  };

  return vaultStore;
}

function render () {
  component = shallow(
    <ChangeVault
      createStore={ createStore() }
      vaultStore={ createVaultStore() }
    />
  );
  instance = component.instance();

  return component;
}

describe('modals/CreateAccount/ChangeVault', () => {
  beforeEach(() => {
    render();
  });

  it('renders defaults', () => {
    expect(component).to.be.ok;
  });

  describe('components', () => {
    describe('VaultSelect', () => {
      let select;

      beforeEach(() => {
        select = component.find('VaultSelect');
      });

      it('renders', () => {
        expect(select.get(0)).to.be.ok;
      });

      it('passes onSelect as instance method', () => {
        expect(select.props().onSelect).to.equal(instance.onSelect);
      });

      it('passes the value', () => {
        expect(select.props().value).to.equal('testing');
      });

      it('passes the vaultStore', () => {
        expect(select.props().vaultStore).to.equal(vaultStore);
      });
    });
  });

  describe('instance methods', () => {
    describe('onSelect', () => {
      it('calls into store setVaultName', () => {
        instance.onSelect('newName');
        expect(store.setVaultName).to.have.been.calledWith('newName');
      });
    });
  });
});
