// Copyright 2015, 2016 Parity Technologies (UK) Ltd.
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

import { ACCOUNT, createApi } from './passwordManager.test.js';

let component;
let onClose;

function render (props) {
  onClose = sinon.stub();

  component = shallow(
    <PasswordManager
      { ...props }
      account={ ACCOUNT }
      onClose={ onClose } />,
    {
      context: {
        api: createApi()
      }
    }
  );

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
      it('calls store.changePassword & props.onClose', () => {
        const instance = component.instance();
        sinon.spy(instance.store, 'changePassword');

        instance.changePassword().then(() => {
          expect(instance.store.changePassword).to.have.been.called;
          expect(onClose).to.have.been.called;
        });
      });
    });

    describe('testPassword', () => {
      it('calls store.testPassword', () => {
        const instance = component.instance();
        sinon.spy(instance.store, 'testPassword');

        instance.testPassword().then(() => {
          expect(instance.store.testPassword).to.have.been.called;
        });
      });
    });
  });
});
