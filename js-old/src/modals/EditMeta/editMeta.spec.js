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

import EditMeta from './';

import { ACCOUNT, createApi, createRedux } from './editMeta.test.js';

let api;
let component;
let instance;
let onClose;
let reduxStore;

function render (props) {
  api = createApi();
  onClose = sinon.stub();
  reduxStore = createRedux();

  component = shallow(
    <EditMeta
      { ...props }
      account={ ACCOUNT }
      onClose={ onClose }
    />,
    { context: { store: reduxStore } }
  ).find('EditMeta').shallow({ context: { api } });
  instance = component.instance();

  return component;
}

describe('modals/EditMeta', () => {
  describe('rendering', () => {
    it('renders defaults', () => {
      expect(render()).to.be.ok;
    });
  });

  describe('actions', () => {
    beforeEach(() => {
      render();
    });

    describe('onSave', () => {
      it('calls store.save', () => {
        sinon.spy(instance.store, 'save');

        return instance.onSave().then(() => {
          expect(instance.store.save).to.have.been.called;
          instance.store.save.restore();
        });
      });

      it('closes the dialog on success', () => {
        return instance.onSave().then(() => {
          expect(onClose).to.have.been.called;
        });
      });

      it('adds newError on failure', () => {
        sinon.stub(instance.store, 'save').rejects('test');

        return instance.onSave().then(() => {
          expect(reduxStore.dispatch).to.have.been.calledWith({ error: new Error('test'), type: 'newError' });
          instance.store.save.restore();
        });
      });
    });
  });
});
