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

import AddContract from './';

import { CONTRACTS, createApi, createRedux } from './addContract.test.js';

let api;
let component;
let instance;
let onClose;
let reduxStore;

function render (props = {}) {
  api = createApi();
  onClose = sinon.stub();
  reduxStore = createRedux();

  component = shallow(
    <AddContract
      { ...props }
      contracts={ CONTRACTS }
      onClose={ onClose }
    />,
    { context: { store: reduxStore } }
  ).find('AddContract').shallow({ context: { api } });
  instance = component.instance();

  return component;
}

describe('modals/AddContract', () => {
  describe('rendering', () => {
    beforeEach(() => {
      render();
    });

    it('renders the defauls', () => {
      expect(component).to.be.ok;
    });
  });

  describe('onAdd', () => {
    it('calls store addContract', () => {
      sinon.stub(instance.store, 'addContract').resolves(true);
      return instance.onAdd().then(() => {
        expect(instance.store.addContract).to.have.been.called;
        instance.store.addContract.restore();
      });
    });

    it('calls closes dialog on success', () => {
      sinon.stub(instance.store, 'addContract').resolves(true);
      return instance.onAdd().then(() => {
        expect(onClose).to.have.been.called;
        instance.store.addContract.restore();
      });
    });

    it('adds newError on failure', () => {
      sinon.stub(instance.store, 'addContract').rejects('test');
      return instance.onAdd().then(() => {
        expect(reduxStore.dispatch).to.have.been.calledWith({ error: new Error('test'), type: 'newError' });
        instance.store.addContract.restore();
      });
    });
  });
});
