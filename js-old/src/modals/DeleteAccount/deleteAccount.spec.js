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

import DeleteAccount from './';

let api;
let component;
let instance;
let onClose;
let router;
let store;

const TEST_ADDRESS = '0x123456789012345678901234567890';
const TEST_PASSWORD = 'testPassword';

function createApi () {
  api = {
    parity: {
      killAccount: sinon.stub().resolves(true)
    }
  };

  return api;
}

function createRouter () {
  router = {
    push: sinon.stub()
  };

  return router;
}

function createStore () {
  store = {
    dispatch: sinon.stub(),
    subscribe: sinon.stub(),
    getState: () => {
      return {};
    }
  };

  return store;
}

function render () {
  onClose = sinon.stub();
  component = shallow(
    <DeleteAccount
      account={ {
        address: TEST_ADDRESS,
        meta: {
          description: 'testDescription'
        }
      } }
      onClose={ onClose }
    />,
    {
      context: {
        store: createStore()
      }
    }
  ).find('DeleteAccount').shallow({
    context: {
      api: createApi(),
      router: createRouter()
    }
  });
  instance = component.instance();

  return component;
}

describe('modals/DeleteAccount', () => {
  beforeEach(() => {
    render();
  });

  it('renders defaults', () => {
    expect(component).to.be.ok;
  });

  describe('event handlers', () => {
    describe('onChangePassword', () => {
      it('sets the state with the new password', () => {
        instance.onChangePassword(null, TEST_PASSWORD);
        expect(instance.state.password).to.equal(TEST_PASSWORD);
      });
    });

    describe('closeDeleteDialog', () => {
      it('calls onClose', () => {
        instance.closeDeleteDialog();
        expect(onClose).to.have.been.called;
      });
    });

    describe('onDeleteConfirmed', () => {
      beforeEach(() => {
        sinon.spy(instance, 'closeDeleteDialog');
        instance.onChangePassword(null, TEST_PASSWORD);
        return instance.onDeleteConfirmed();
      });

      afterEach(() => {
        instance.closeDeleteDialog.restore();
      });

      it('calls parity_killAccount', () => {
        expect(api.parity.killAccount).to.have.been.calledWith(TEST_ADDRESS, TEST_PASSWORD);
      });

      it('changes the route to /accounts', () => {
        expect(router.push).to.have.been.calledWith('/accounts');
      });

      it('closes the dialog', () => {
        expect(instance.closeDeleteDialog).to.have.been.called;
      });
    });
  });
});
