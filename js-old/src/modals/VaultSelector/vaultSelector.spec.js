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

import VaultSelector from './';

const VAULTS_OPENED = [
  { name: 'A', isOpen: true },
  { name: 'B', isOpen: true }
];
const VAULTS_CLOSED = [
  { name: 'C' },
  { name: 'D' }
];
const VAULTS_ALL = VAULTS_OPENED.concat(VAULTS_CLOSED);

let component;
let instance;
let onClose;
let onSelect;
let vaultStore;

function createVaultStore () {
  vaultStore = {
    vaults: VAULTS_ALL,
    vaultsOpened: VAULTS_OPENED
  };

  return vaultStore;
}

function render () {
  onClose = sinon.stub();
  onSelect = sinon.stub();

  component = shallow(
    <VaultSelector
      onClose={ onClose }
      onSelect={ onSelect }
      selected='firstValue'
      vaultStore={ createVaultStore() }
    />
  );
  instance = component.instance();

  return component;
}

describe('ui/VaultSelector', () => {
  beforeEach(() => {
    render();
  });

  it('renders defaults', () => {
    expect(component).to.be.ok;
  });

  describe('components', () => {
    describe('Portal', () => {
      let portal;

      beforeEach(() => {
        portal = component.find('Portal');
      });

      it('renders', () => {
        expect(portal.get(0)).to.be.ok;
      });

      it('opens as a child modal', () => {
        expect(portal.props().isChildModal).to.be.true;
      });

      it('passes the instance onClose', () => {
        expect(portal.props().onClose).to.equal(instance.onClose);
      });
    });

    describe('SelectionList', () => {
      let list;

      beforeEach(() => {
        list = component.find('SelectionList');
      });

      it('renders', () => {
        expect(list.get(0)).to.be.ok;
      });

      it('passes the open vaults', () => {
        expect(list.props().items).to.deep.equal(VAULTS_OPENED);
      });

      it('passes internal renderItem', () => {
        expect(list.props().renderItem).to.equal(instance.renderVault);
      });

      it('passes internal isChecked', () => {
        expect(list.props().isChecked).to.equal(instance.isSelected);
      });

      it('passes internal onSelectClick', () => {
        expect(list.props().onSelectClick).to.equal(instance.onSelect);
      });
    });
  });

  describe('instance methods', () => {
    describe('renderVault', () => {
      let card;

      beforeEach(() => {
        card = instance.renderVault({ name: 'testVault' });
      });

      it('renders VaultCard', () => {
        expect(card).to.be.ok;
      });
    });

    describe('isSelected', () => {
      it('returns true when vault name matches', () => {
        expect(instance.isSelected({ name: 'firstValue' })).to.be.true;
      });

      it('returns false when vault name does not match', () => {
        expect(instance.isSelected({ name: 'testValue' })).to.be.false;
      });
    });

    describe('onSelect', () => {
      it('calls into props onSelect', () => {
        instance.onSelect({ name: 'testing' });
        expect(onSelect).to.have.been.called;
      });

      it('passes name when new selection made', () => {
        instance.onSelect({ name: 'newValue' });
        expect(onSelect).to.have.been.calledWith('newValue');
      });

      it('passes empty name when current selection made', () => {
        instance.onSelect({ name: 'firstValue' });
        expect(onSelect).to.have.been.calledWith('');
      });
    });

    describe('onClose', () => {
      it('calls props onClose', () => {
        instance.onClose();
        expect(onClose).to.have.been.called;
      });
    });
  });
});
