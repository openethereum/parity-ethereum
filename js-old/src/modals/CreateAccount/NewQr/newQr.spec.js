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

import NewQr from './';

let component;
let instance;
let createStore;
let vaultStore;

function createStores () {
  createStore = {
    qrAddressValid: false,
    setDescription: sinon.stub(),
    setName: sinon.stub(),
    setQrAddress: sinon.stub()
  };

  vaultStore = {};
}

function render (props = {}) {
  createStores();

  component = shallow(
    <NewQr
      createStore={ createStore }
      vaultStore={ vaultStore }
    />
  );
  instance = component.instance();

  return component;
}

describe('modals/CreateAccount/NewQr', () => {
  beforeEach(() => {
    render();
  });

  it('renders defaults', () => {
    expect(component).to.be.ok;
  });

  describe('event methods', () => {
    describe('onEditAccountDescription', () => {
      beforeEach(() => {
        instance.onEditAccountDescription(null, 'testing');
      });

      it('calls into createStore.setDescription', () => {
        expect(createStore.setDescription).to.have.been.calledWith('testing');
      });
    });

    describe('onEditAccountName', () => {
      beforeEach(() => {
        instance.onEditAccountName(null, 'testing');
      });

      it('calls into createStore.setName', () => {
        expect(createStore.setName).to.have.been.calledWith('testing');
      });
    });

    describe('onScan', () => {
      beforeEach(() => {
        instance.onScan('testing');
      });

      it('calls into createStore.setQrAddress', () => {
        expect(createStore.setQrAddress).to.have.been.calledWith('testing');
      });
    });
  });
});
