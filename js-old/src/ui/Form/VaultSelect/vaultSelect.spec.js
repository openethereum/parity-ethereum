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

import VaultSelect from './';

let component;
let instance;
let onSelect;
let vaultStore;

function createVaultStore () {
  vaultStore = {
    loadVaults: sinon.stub().resolves(true)
  };

  return vaultStore;
}

function render () {
  onSelect = sinon.stub();

  component = shallow(
    <VaultSelect
      onSelect={ onSelect }
      value='initialValue'
      vaultStore={ createVaultStore() }
    />
  );
  instance = component.instance();

  return component;
}

describe('ui/Form/VaultSelect', () => {
  beforeEach(() => {
    render();
  });

  it('renders defaults', () => {
    expect(component).to.be.ok;
  });

  describe('components', () => {
    describe('InputAddress', () => {
      let input;

      beforeEach(() => {
        input = component.find('Connect(InputAddress)');
      });

      it('renders', () => {
        expect(input.get(0)).to.be.ok;
      });

      it('passes value from props', () => {
        expect(input.props().value).to.equal('INITIALVALUE');
      });

      it('passes instance openSelector to onClick', () => {
        expect(input.props().onClick).to.equal(instance.openSelector);
      });
    });
  });

  describe('instance methods', () => {
    describe('onSelect', () => {
      it('calls into props', () => {
        instance.onSelect('testing');
        expect(onSelect).to.have.been.calledWith('testing');
      });
    });
  });
});
