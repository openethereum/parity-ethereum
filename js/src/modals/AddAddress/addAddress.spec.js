// Copyright 2015, 2016 Ethcore (UK) Ltd.
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

import { mount } from 'enzyme';
import React from 'react';
import sinon from 'sinon';

import AddAddress from './';

import { TEST_CONTACTS } from './store.test.js';

const api = {
  parity: {
    setAccountMeta: sinon.stub().resolves(true),
    setAccountName: sinon.stub().resolves(true)
  }
};

function render (props) {
  return mount(
    <AddAddress
      contacts={ TEST_CONTACTS }
      { ...props } />,
    { context: { api } }
  );
}

describe('modals/AddAddress', () => {
  it('renders', () => {
    expect(render()).to.be.ok;
  });

  describe('actions', () => {
    let onClose;
    let component;

    beforeEach(() => {
      onClose = sinon.stub();
      component = render({ onClose });
    });

    describe('close', () => {
      it('calls the props.onClose', () => {
        component.ref('closeButton').click();

        expect(onClose).to.have.been.called;
      });
    });

    describe('add', () => {
      it('adds the name & meta and calls the props.onClose', () => {
        component.ref('addButton').click();

        expect(onClose).to.have.been.called;
        expect(api.parity.setAccountMeta).to.have.been.called;
        expect(api.parity.setAccountName).to.have.been.called;
      });
    });
  });
});
