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

import PasswordManager from './';

import { ACCOUNT, createApi, createRedux } from './passwordManager.test.js';

let component;
let instance;
let onClose;
let reduxStore;

function render (props) {
  onClose = sinon.stub();
  reduxStore = createRedux();

  component = shallow(
    <PasswordManager
      { ...props }
      account={ ACCOUNT }
      onClose={ onClose }
    />,
    { context: { store: reduxStore } }
  ).find('PasswordManager').shallow({ context: { api: createApi() } });
  instance = component.instance();

  return component;
}

describe('modals/PasswordManager', () => {
  describe('rendering', () => {
    it('renders defaults', () => {
      expect(render()).to.be.ok;
    });
  });

  describe('actions', () => {
    beforeEach(() => {
      render();
    });

    describe('changePassword', () => {
      it('calls store.changePassword', () => {
        sinon.spy(instance.store, 'changePassword');

        return instance.changePassword().then(() => {
          expect(instance.store.changePassword).to.have.been.called;
          instance.store.changePassword.restore();
        });
      });

      it('closes the dialog on success', () => {
        return instance.changePassword().then(() => {
          expect(onClose).to.have.been.called;
        });
      });

      it('shows snackbar on success', () => {
        return instance.changePassword().then(() => {
          expect(reduxStore.dispatch).to.have.been.calledWithMatch({ type: 'openSnackbar' });
        });
      });

      it('adds newError on failure', () => {
        sinon.stub(instance.store, 'changePassword').rejects('test');

        return instance.changePassword().then(() => {
          expect(reduxStore.dispatch).to.have.been.calledWith({ error: new Error('test'), type: 'newError' });
          instance.store.changePassword.restore();
        });
      });
    });

    describe('testPassword', () => {
      it('calls store.testPassword', () => {
        sinon.spy(instance.store, 'testPassword');

        return instance.testPassword().then(() => {
          expect(instance.store.testPassword).to.have.been.called;
          instance.store.testPassword.restore();
        });
      });

      it('adds newError on failure', () => {
        sinon.stub(instance.store, 'testPassword').rejects('test');

        return instance.testPassword().then(() => {
          expect(reduxStore.dispatch).to.have.been.calledWith({ error: new Error('test'), type: 'newError' });
          instance.store.testPassword.restore();
        });
      });
    });
  });
});
